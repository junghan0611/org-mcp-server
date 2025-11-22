# org-mcp-server 성능 분석

**테스트 날짜**: 2025-11-22
**환경**: NixOS, ~/org/ 디렉토리
**파일 개수**: 3051개 org 파일

## 벤치마크 결과

### 파일 목록 (list)

```bash
$ time org-cli list
```

**결과:**
- 파일 개수: 3051개
- 실행 시간: **37ms**
- 평가: ⚡ **매우 빠름**

**분석:**
- `ignore` 크레이트로 효율적인 파일 탐색
- .gitignore 패턴 자동 존중
- 3000+ 파일도 100ms 이내 처리

---

### Heading 추출 (outline)

```bash
$ time org-cli outline ~/org/20240906T154822--home-notesjunghanacscom__homepage.org --format json
```

**결과:**
- Heading 개수: 16개
- 실행 시간: **4ms**
- Line number 포함: ✅

**JSON 출력 예시:**
```json
{
  "label": "Introduction",
  "level": 1,
  "line_number": 6,
  "line_end": 15,
  "tags": []
}
```

**분석:**
- orgize 파서 매우 빠름
- line_number 추출 오버헤드 미미
- 단일 파일 파싱은 10ms 이내

---

### 전문 검색 (search)

```bash
$ time org-cli search "emacs" --limit 10
```

**결과:**
- 검색 파일: 3051개
- 실행 시간: **96초 (1분 36초)**
- 평가: ❌ **개선 필요**

**병목 분석:**

1. **순차 파일 I/O**
```rust
for file in files {
    let content = self.read_file(&file)?;  // ← 병목!
    // 검색...
}
```

2. **캐싱 없음**
- 매번 파일시스템에서 읽기
- 반복 검색 시 동일한 I/O 반복

3. **단일 스레드 처리**
- CPU 코어 1개만 사용
- 병렬화 가능 지점

---

## 성능 목표

### 현재 vs 목표

| 작업 | 현재 | 목표 | 개선 방법 |
|------|------|------|-----------|
| **list** | 37ms | ✅ | 이미 최적 |
| **outline** | 4ms | ✅ | 이미 최적 |
| **search** | 96초 | **< 10초** | rayon + cache |

### 예상 개선 효과

**rayon 병렬화 (8 코어):**
```rust
files.par_iter()  // 병렬 처리
    .filter_map(|f| search_file(f, query))
    .collect()
```
- 96초 → **12-15초** (6-8배 개선)

**DashMap 캐싱:**
```rust
cache: DashMap<PathBuf, (SystemTime, String)>
```
- 반복 검색: 96초 → **< 1초** (100배+ 개선)

**조합 (병렬 + 캐시):**
- 첫 검색: **10-15초**
- 반복 검색: **< 1초**

---

## 병목 지점 상세 분석

### 1. search 함수 (org-core/src/org_mode.rs)

**현재 구현:**
```rust
pub fn search(&self, query: &str, limit: Option<usize>, tags: Option<Vec<String>>)
    -> Result<Vec<SearchResult>, OrgModeError>
{
    let files = self.list_files(tags)?;

    for file in files {
        let content = self.read_file(&file)?;  // ← I/O 병목!
        let matches = pattern.match_list(
            content.lines().map(|s| s.to_owned()).collect::<Vec<_>>(),
            &mut matcher,
        );
        // ...
    }
}
```

**문제점:**
1. 순차 파일 읽기 (3051번 I/O)
2. 매번 `Vec<String>` 할당
3. 캐시 없음

### 2. Arc<Mutex> 동시성 제한

**현재:**
```rust
// org-mcp-server/src/core.rs
pub struct OrgModeRouter {
    pub(crate) org_mode: Arc<Mutex<OrgMode>>,
}
```

**문제:**
- 읽기도 Mutex로 직렬화
- 동시 검색 불가

**개선:**
```rust
pub(crate) org_mode: Arc<RwLock<OrgMode>>,
```

---

## 개선 계획

### Phase 1.5: 병렬 검색 (우선순위: 높음)

**목표:** 96초 → 10-15초

**구현:**
```rust
use rayon::prelude::*;

pub fn search_parallel(&self, query: &str, ...) -> Result<Vec<SearchResult>, OrgModeError> {
    let files = self.list_files(tags)?;

    let results: Vec<_> = files.par_iter()  // 병렬화!
        .filter_map(|file| {
            let content = self.read_file(file).ok()?;
            // 검색 로직...
            Some(results)
        })
        .flatten()
        .take(limit.unwrap_or(usize::MAX))
        .collect();

    Ok(results)
}
```

**예상 효과:** 6-8배 개선 (8 CPU 코어 기준)

### Phase 2: 파일 캐싱 (우선순위: 중간)

**목표:** 반복 검색 < 1초

**구현:**
```rust
use dashmap::DashMap;
use std::time::SystemTime;

pub struct CachedOrgMode {
    inner: OrgMode,
    file_cache: DashMap<PathBuf, (SystemTime, String)>,
}

impl CachedOrgMode {
    fn read_file_cached(&self, path: &Path) -> Result<String, OrgModeError> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;

        if let Some((cached_time, content)) = self.file_cache.get(path) {
            if *cached_time == modified {
                return Ok(content.clone());
            }
        }

        let content = fs::read_to_string(path)?;
        self.file_cache.insert(path.to_path_buf(), (modified, content.clone()));
        Ok(content)
    }
}
```

**예상 효과:** 100배+ 개선 (반복 검색 시)

### Phase 3: RwLock 전환 (우선순위: 낮음)

**목표:** 동시 읽기 허용

```rust
Arc<RwLock<OrgMode>>
```

---

## 비교: Elisp vs Rust

### 예상 성능 (3000개 파일)

| 작업 | Elisp (추정) | Rust (현재) | Rust (최적화) |
|------|-------------|------------|--------------|
| 파일 목록 | 5-10초 | **37ms** | 37ms |
| Heading 추출 | 100ms-1s | **4ms** | 4ms |
| 전문 검색 | 5-10분 | 96초 | **10초** |

**Rust 우위:**
- list: **100-200배 빠름**
- outline: **25-250배 빠름**
- search: **현재도 3-6배**, 최적화 시 **30-60배 빠름**

---

## nvim-orgmode 통합 시나리오

### Neovim에서 활용

```lua
-- Neovim에서
:Telescope orgmode search_headings

-- 내부적으로
vim.fn.system('org-cli outline ' .. file .. ' --format json')

-- 결과
{
  "label": "Section",
  "line_number": 42  -- ← Neovim이 42번 줄로 점프!
}
```

**장점:**
- Lua I/O 대신 Rust 속도
- Line number로 정확한 위치
- 3000+ 파일도 빠른 검색

---

## 결론

### 현재 성능

✅ **파일 탐색/파싱**: 이미 매우 빠름
❌ **전문 검색**: 개선 필요 (96초 → 10초)

### 개선 우선순위

1. **rayon 병렬 검색** (즉각 6-8배 개선)
2. **파일 캐싱** (반복 검색 100배+ 개선)
3. **RwLock 전환** (동시 읽기 허용)

### AI Agent 활용 시나리오

**Before (grep/100줄 읽기):**
```
3051개 파일 grep → 몇 분 소요
```

**After (org-cli outline):**
```
1. org-cli list --tags active        → 37ms (필터링된 파일)
2. org-cli outline file.org          → 4ms (heading + line number)
3. Read file.org offset=42 limit=50  → Agent가 정확한 위치만 읽기
```

**총 시간: < 100ms** (1000배+ 개선!)

---

## 다음 단계

1. **rayon 병렬 검색 구현**
2. **성능 벤치마크 CI 추가**
3. **Upstream PR 준비**
4. **캐싱 레이어 설계**

---

**Rust + org-mode = AI 시대의 완벽한 지식베이스!**
