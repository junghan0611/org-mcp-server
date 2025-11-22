# org-mcp-server 검색 전략 총괄

**작성일**: 2025-11-22
**대상**: 3051개 org 파일 (~/org/)

## 현재 구현 분석

### search 함수 상세 (org-core/src/org_mode.rs:245-292)

```rust
pub fn search(
    &self,
    query: &str,
    limit: Option<usize>,
    snippet_max_size: Option<usize>,
) -> Result<Vec<SearchResult>, OrgModeError> {
    // 1. nucleo-matcher 초기화
    let mut matcher = Matcher::new(NucleoConfig::DEFAULT);
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,      // 대소문자 무시
        Normalization::Smart,       // 스마트 정규화
        AtomKind::Fuzzy,           // Fuzzy 매칭
    );

    // 2. 모든 파일 목록 가져오기 (37ms)
    let files = self.list_files(None, None)?;
    let mut all_results = Vec::new();

    // 3. 순차 파일 처리 (병목!)
    for file in files {  // ← 3051번 반복
        // 3-1. 파일 읽기 (I/O 병목)
        let content = match self.read_file(&file) {  // ← 순차 I/O!
            Ok(content) => content,
            Err(_) => continue,
        };

        // 3-2. 줄 단위 Vec 변환 (메모리 할당)
        let matches = pattern.match_list(
            content.lines()
                .map(|s| s.to_owned())      // ← String 복사!
                .collect::<Vec<_>>(),        // ← Vec 할당!
            &mut matcher,
        );

        // 3-3. 결과 수집
        for (snippet, score) in matches {
            let snippet = Self::snippet(&snippet, snippet_max_size.unwrap_or(100));
            all_results.push(SearchResult {
                file_path: file.clone(),
                snippet,
                score,
                tags: self.tags_in_file(&file).unwrap_or_default(),  // ← 추가 파싱!
            });
        }
    }

    // 4. 전체 결과 정렬 후 truncate
    all_results.sort_by(|a, b| b.score.cmp(&a.score));
    all_results.truncate(limit.unwrap_or(all_results.len()));

    Ok(all_results)
}
```

---

## 성능 병목 상세 분석

### 병목 1: 순차 파일 I/O (가장 큰 문제!)

**코드 위치**: 266-270번째 줄

```rust
for file in files {  // 3051개 파일
    let content = match self.read_file(&file) {  // ← 순차 I/O!
        Ok(content) => content,
        Err(_) => continue,
    };
    // ...
}
```

**문제점:**
1. **순차 처리**: 한 번에 1개 파일만 읽기
2. **I/O 대기**: CPU가 디스크 I/O 대기하며 idle
3. **단일 스레드**: 8 CPU 코어 중 1개만 사용

**시간 계산:**
```
파일 1개 읽기: ~30ms (평균)
3051개 × 30ms = 91.5초
실제 측정: 96초 ✅ 일치!
```

**왜 느린가:**
- 각 파일을 열고 (`open`)
- 전체 내용을 읽고 (`read_to_string`)
- 닫고 (`close`)
- 다음 파일... 반복

**병렬화 가능 이유:**
- ✅ 파일 간 의존성 없음
- ✅ 각 파일 독립적으로 처리 가능
- ✅ 결과만 합치면 됨

---

### 병목 2: 메모리 할당 (중간 문제)

**코드 위치**: 272-275번째 줄

```rust
let matches = pattern.match_list(
    content.lines()
        .map(|s| s.to_owned())           // ← String 복사!
        .collect::<Vec<_>>(),            // ← Vec 할당!
    &mut matcher,
);
```

**문제점:**
1. **String 복사**: 각 줄을 새로운 String으로 복사
2. **Vec 할당**: 전체 줄을 Vec에 담기
3. **메모리 오버헤드**: 대용량 파일 시 메모리 스파이크

**예시 (100KB 파일):**
```
원본 content: 100KB
lines() → Vec<String>: ~200KB (복사 + 메타데이터)
메모리 사용: 2배 증가
```

**개선 방안:**
```rust
// 복사 없이 &str slice 사용
content.lines()  // Iterator<Item = &str>
    .enumerate()  // 줄 번호 포함
    .filter_map(|(line_num, line)| {
        matcher.fuzzy_match(line, query)
            .map(|score| (line, line_num, score))
    })
```

---

### 병목 3: 추가 파싱 (작은 문제)

**코드 위치**: 283번째 줄

```rust
tags: self.tags_in_file(&file).unwrap_or_default(),  // ← 추가 파싱!
```

**문제점:**
- 매칭된 파일마다 다시 파싱
- `tags_in_file()`이 `Org::parse()` 호출

**개선 방안:**
- 태그를 캐싱
- 또는 search 결과에서 태그 제외 (필요 시에만)

---

## rayon 병렬화 상세 설계

### rayon이란?

**Rust의 데이터 병렬 처리 라이브러리:**
```toml
[dependencies]
rayon = "1.10"
```

**특징:**
- Work-stealing 알고리즘
- 자동 스레드 풀 관리
- 안전한 병렬 iterator

### 병렬화 전략

#### Before (순차 처리)

```
Thread 1: [File1] → [File2] → [File3] → ... → [File3051]
Thread 2: (idle)
Thread 3: (idle)
...
Thread 8: (idle)

총 시간: 96초
CPU 사용률: 12.5% (1/8)
```

#### After (rayon 병렬 처리)

```
Thread 1: [File1]   [File9]   [File17]  ...
Thread 2: [File2]   [File10]  [File18]  ...
Thread 3: [File3]   [File11]  [File19]  ...
Thread 4: [File4]   [File12]  [File20]  ...
Thread 5: [File5]   [File13]  [File21]  ...
Thread 6: [File6]   [File14]  [File22]  ...
Thread 7: [File7]   [File15]  [File23]  ...
Thread 8: [File8]   [File16]  [File24]  ...

총 시간: 12-15초 (6-8배 개선)
CPU 사용률: 90%+ (거의 모든 코어)
```

### 구체적 구현

```rust
use rayon::prelude::*;

pub fn search_parallel(
    &self,
    query: &str,
    limit: Option<usize>,
    snippet_max_size: Option<usize>,
) -> Result<Vec<SearchResult>, OrgModeError> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let files = self.list_files(None, None)?;
    let snippet_max = snippet_max_size.unwrap_or(100);

    // 병렬 검색 (핵심!)
    let all_results: Vec<SearchResult> = files
        .par_iter()  // ← 병렬 iterator!
        .filter_map(|file| {
            // 각 스레드가 독립적으로 처리
            let content = self.read_file(file).ok()?;

            // 스레드 로컬 matcher
            let mut matcher = Matcher::new(NucleoConfig::DEFAULT);
            let pattern = Pattern::new(
                query,
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
            );

            let matches = pattern.match_list(
                content.lines().map(|s| s.to_owned()).collect::<Vec<_>>(),
                &mut matcher,
            );

            // 이 파일의 결과만 수집
            let file_results: Vec<SearchResult> = matches
                .into_iter()
                .map(|(snippet, score)| {
                    let snippet = Self::snippet(&snippet, snippet_max);
                    SearchResult {
                        file_path: file.clone(),
                        snippet,
                        score,
                        tags: self.tags_in_file(file).unwrap_or_default(),
                    }
                })
                .collect();

            Some(file_results)
        })
        .flatten()  // Vec<Vec<SearchResult>> → Iterator<SearchResult>
        .collect();  // → Vec<SearchResult>

    // 정렬 및 제한
    let mut all_results = all_results;
    all_results.sort_by(|a, b| b.score.cmp(&a.score));
    all_results.truncate(limit.unwrap_or(all_results.len()));

    Ok(all_results)
}
```

### 개선 효과

**시간 복잡도:**
- Before: O(n) - 순차 처리
- After: O(n/cores) - 병렬 처리

**예상 성능 (8 코어):**
```
이론적: 96초 / 8 = 12초
실제 (오버헤드 고려): 12-15초
개선 비율: 6-8배
```

**추가 개선 (메모리 최적화):**
```rust
// String 복사 제거
content.lines()
    .filter_map(|line| {
        matcher.fuzzy_match(line, query)
            .map(|score| (line.to_string(), score))
    })
```

예상: 15초 → **10초**

---

## 검색 전략 총괄

### 검색 유형별 비교

| 검색 유형 | 현재 구현 | 속도 | Use Case | 정확도 |
|----------|----------|------|----------|--------|
| **Fuzzy** | ✅ nucleo | 96초 → 10초 | 오타 허용 검색 | 중간 |
| **Regex** | ❌ | - | 패턴 매칭 | 높음 |
| **Semantic (RAG)** | ❌ | ~1-2초 | 의미 기반 검색 | 매우 높음 |
| **Structured** | 부분적 | < 1초 | heading/tag 검색 | 매우 높음 |

### 1. Fuzzy Search (현재)

**nucleo-matcher:**
- Neovim telescope에서 사용하는 동일 엔진
- 오타 허용 (typo tolerance)
- 순위 점수 (scoring)

**장점:**
- ✅ 빠른 대화형 검색
- ✅ 오타 허용
- ✅ 직관적

**단점:**
- ❌ 의미 기반 검색 불가
- ❌ 정확한 패턴 매칭 어려움
- ❌ 현재 병렬화 안 됨

**Use Case:**
```bash
# "emcs" 입력 → "emacs" 찾기
org-cli search "emcs"  # fuzzy match!

# "knowlege" → "knowledge"
org-cli search "knowlege"
```

---

### 2. Regex Search (미구현, 추가 권장)

**구현 예시:**
```rust
use regex::Regex;

pub fn search_regex(
    &self,
    pattern: &str,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>, OrgModeError> {
    let re = Regex::new(pattern)?;

    files.par_iter()
        .filter_map(|file| {
            let content = self.read_file(file).ok()?;
            let matches: Vec<_> = content
                .lines()
                .enumerate()
                .filter(|(_, line)| re.is_match(line))
                .collect();
            // ...
        })
        .collect()
}
```

**장점:**
- ✅ 정확한 패턴 매칭
- ✅ 복잡한 쿼리 가능
- ✅ 병렬화 가능

**Use Case:**
```bash
# TODO 키워드로 시작하는 줄
org-cli search-regex "^\* TODO"

# 날짜 패턴
org-cli search-regex "\d{4}-\d{2}-\d{2}"

# 특정 property
org-cli search-regex ":CUSTOM_ID:.*"
```

---

### 3. Semantic Search (RAG) - 상호 보완 관계

**임베딩 기반 검색:**

```
[org 파일들]
    ↓ 임베딩 생성 (사전)
[Vector DB]
    ↓ 쿼리 임베딩
[Semantic 검색]
```

#### RAG와의 상호 보완 구조

```
┌─────────────────────────────────────┐
│ 사용자 쿼리                          │
└─────────────────────────────────────┘
         ↓
┌─────────────────────────────────────┐
│ 쿼리 분석 (AI Agent)                 │
└─────────────────────────────────────┘
         ↓
    ┌────┴────┐
    ↓         ↓
┌────────┐ ┌──────────┐
│ Fuzzy  │ │ Semantic │
│ Search │ │ (RAG)    │
└────────┘ └──────────┘
org-mcp      Vector DB
(실시간)      (인덱스)
    ↓         ↓
┌─────────────────────────────────────┐
│ 결과 통합 & 랭킹                     │
└─────────────────────────────────────┘
```

#### 각 검색의 역할

**1. org-mcp-server (Fuzzy/Regex) - 실시간**

**강점:**
- ✅ **최신 정보**: 파일 수정 즉시 반영
- ✅ **정확한 매칭**: 키워드 기반
- ✅ **구조 활용**: heading, tag, property
- ✅ **DB 불필요**: 파일명만으로 메타데이터

**약점:**
- ❌ 의미 이해 불가
- ❌ 동의어 처리 어려움
- ❌ 개념 기반 검색 불가

**Use Case:**
```bash
# 특정 태그 파일
org-cli list --tags active

# 특정 키워드 (정확한 매칭)
org-cli search "TODO cleanup"

# Heading 구조 탐색
org-cli outline file.org  # line_number로 정확한 위치
```

**2. RAG (Semantic) - 의미 기반**

**강점:**
- ✅ **의미 이해**: "인공지능" ≈ "AI" ≈ "머신러닝"
- ✅ **개념 검색**: "성능 개선 방법" → 관련 문서들
- ✅ **문맥 이해**: 문장/문단 수준

**약점:**
- ❌ **인덱싱 지연**: 파일 수정 후 재임베딩 필요
- ❌ **리소스 소모**: GPU/메모리
- ❌ **정확성**: 키워드 정확 매칭 어려움

**Use Case:**
```
# 의미 기반 질문
"성능을 개선하는 방법은?"
→ 벡터 검색 → 관련 문서들

"Rust 병렬 처리 예제"
→ rayon, tokio 관련 문서
```

---

### 하이브리드 검색 전략 (권장!)

#### Tier 1: 구조적 필터링 (org-mcp-server)

```bash
# 1단계: 빠른 필터링 (< 100ms)
org-cli list --tags rust,performance

# 결과: 50개 파일로 축소
```

**장점:**
- 즉시 범위 축소
- 정확한 태그 매칭
- DB 업데이트 불필요

#### Tier 2: 키워드 검색 (org-mcp-server)

```bash
# 2단계: Fuzzy 검색 (50개 파일 대상, ~2초)
org-cli search "rayon parallel" --limit 20
```

**장점:**
- 축소된 범위에서 빠른 검색
- 키워드 정확도

#### Tier 3: Semantic 확장 (RAG)

```python
# 3단계: 의미 확장 (20개 문서 대상)
rag.expand_search(
    results=cli_results,
    query="성능 개선 방법",
    top_k=5
)
```

**장점:**
- 이미 필터링된 결과에서 semantic 랭킹
- 적은 문서 → 빠른 처리
- 의미 기반 정렬

---

### 실제 워크플로우 예시

**사용자 쿼리: "Rust 병렬 처리로 성능 개선한 사례"**

```
Step 1: 태그 필터 (org-mcp-server)
  org-cli list --tags rust,performance
  → 50개 파일 (37ms)

Step 2: Heading 스캔 (org-mcp-server)
  for file in 50_files:
      org-cli outline $file
  → 각 4ms × 50 = 200ms
  → "병렬", "rayon", "개선" 키워드 포함 heading만 선택
  → 15개 heading

Step 3: 정확한 위치 읽기 (Read tool)
  for heading in 15_headings:
      Read file offset=heading.line_number limit=50
  → 각 < 1ms
  → 15개 섹션 내용

Step 4 (선택): Semantic 랭킹 (RAG)
  embedding_search(sections, query)
  → Top 5 가장 관련있는 섹션

총 시간: ~500ms (RAG 제외)
정확도: 매우 높음
```

**vs 기존 방식:**
```
grep -r "병렬 처리" ~/org/
→ 몇 분 소요
→ 구조 무시
→ 불필요한 매칭 많음
```

---

## 검색 타입별 권장 사항

### 언제 Fuzzy Search를 사용하는가?

**적합한 경우:**
- ✅ 키워드 기반 검색
- ✅ 오타 허용 필요
- ✅ 파일명/태그 검색
- ✅ 실시간 반영 필요

**예시:**
```bash
# "emmacs" → "emacs" 찾기
org-cli search "emmacs"

# 태그 조합
org-cli list --tags active,rust
```

### 언제 Regex Search를 사용하는가?

**적합한 경우:**
- ✅ 정확한 패턴 매칭
- ✅ 구조화된 데이터 (날짜, ID 등)
- ✅ 복잡한 조건

**예시:**
```bash
# TODO 키워드
org-cli search-regex "^\* TODO.*:urgent:"

# 날짜 범위
org-cli search-regex "SCHEDULED: <2025-11-\d{2}"

# CUSTOM_ID 검색
org-cli search-regex ":CUSTOM_ID: [a-z0-9-]+"
```

### 언제 Semantic Search (RAG)를 사용하는가?

**적합한 경우:**
- ✅ 개념/의미 기반 질문
- ✅ 동의어/유사어 처리
- ✅ 문맥 이해 필요
- ✅ 탐색적 검색

**예시:**
```
# 질문 형태
"Rust로 성능을 개선하는 방법은?"
→ rayon, async, 캐싱 관련 문서

# 개념 검색
"지식 관리 시스템"
→ org-roam, denote, zettelkasten 관련
```

**단점:**
- ❌ 인덱싱 시간 (초기 1-2분)
- ❌ 파일 수정 후 재임베딩 필요
- ❌ GPU/메모리 리소스

---

## 통합 검색 아키텍처 (최종 제안)

### 3-Tier 검색 시스템

```
┌─────────────────────────────────────────────────┐
│ Tier 1: 구조적 필터 (org-mcp-server)             │
│ • 태그 필터: list --tags                        │
│ • 속도: < 100ms                                 │
│ • 용도: 범위 축소 (3000 → 100)                  │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│ Tier 2: 키워드 검색 (org-mcp-server parallel)    │
│ • Fuzzy/Regex 검색                              │
│ • 속도: 10초 (병렬화)                           │
│ • 용도: 키워드 매칭 (100 → 20)                  │
└─────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────┐
│ Tier 3: Semantic 랭킹 (RAG, 선택적)              │
│ • 의미 기반 정렬                                │
│ • 속도: 1-2초                                   │
│ • 용도: 최종 랭킹 (20 → 5)                      │
└─────────────────────────────────────────────────┘
```

### 각 Tier의 역할

**Tier 1 (필수):**
- 빠른 메타데이터 기반 필터
- DB 불필요
- 실시간 반영

**Tier 2 (필수):**
- 내용 기반 검색
- 병렬화로 빠른 처리
- 정확한 키워드 매칭

**Tier 3 (선택):**
- 의미 기반 정렬
- 탐색적 검색
- LLM 컨텍스트 최적화

---

## 상호 보완 시나리오

### Scenario A: 빠른 참조 검색

```
사용자: "active 태그가 붙은 Rust 관련 파일"

1. org-cli list --tags active,rust  (37ms)
   → 15개 파일

2. org-cli search "rayon" (15개 파일, ~1초)
   → 3개 매칭

3. org-cli outline file.org
   → heading + line_number

결과: 즉시 정확한 위치
RAG: 불필요
```

### Scenario B: 탐색적 연구

```
사용자: "성능 최적화 관련 아이디어"

1. org-cli list --tags performance  (37ms)
   → 100개 파일

2. RAG semantic search (100개 파일)
   "성능 최적화 기법"
   → 20개 관련 문서

3. org-cli outline (20개 파일)
   → 구조 파악

4. Read 정확한 섹션

결과: 의미적으로 관련있는 문서 발견
org-mcp: 구조 제공
RAG: 의미 랭킹
```

### Scenario C: 하이브리드 (최적)

```
사용자: "Rust async 사용한 파일에서 tokio 에러 처리 방법"

1. org-cli list --tags rust  (37ms)
   → 80개 파일

2. org-cli search "tokio" --parallel (80개, ~3초)
   → 25개 파일

3. RAG "에러 처리 패턴" (25개 파일, ~500ms)
   → Top 5 관련 문서

4. org-cli outline (5개 파일)
   → 정확한 섹션 위치

5. Read 필요한 부분만

총 시간: ~4초
정확도: 매우 높음
효율성: 최적
```

---

## org-mcp-server만으로 충분한가?

### org-mcp-server만으로 가능한 것

✅ **구조적 탐색**
```
tags → files → outlines → line_number → read
```

✅ **키워드 검색**
```
fuzzy/regex → snippets → 정확한 매칭
```

✅ **실시간 반영**
```
파일 수정 → 즉시 검색 가능 (DB 업데이트 불필요)
```

### RAG가 필요한 경우

❌ **의미 기반 질문**
```
"성능 개선 방법은?" → 키워드 없이 개념으로 검색
```

❌ **동의어/유사어**
```
"AI" ≈ "인공지능" ≈ "머신러닝" ≈ "딥러닝"
```

❌ **문맥 이해**
```
"이 문제의 해결책" → 주변 문맥 필요
```

---

## 권장 통합 전략

### 단계별 구현

**Phase 1: org-mcp-server 최적화 (현재)**
1. ✅ Line number 추가
2. ⏳ rayon 병렬 검색
3. ⏳ Regex search 추가
4. ⏳ 파일 캐싱

**Phase 2: RAG 통합 (향후)**
1. 임베딩 생성 파이프라인
2. Vector DB (qdrant/chromadb)
3. Hybrid search API
4. 결과 통합 랭커

**Phase 3: 지능형 라우팅 (미래)**
```rust
pub enum SearchStrategy {
    Structural,  // tags, heading
    Keyword,     // fuzzy, regex
    Semantic,    // RAG
    Hybrid,      // 조합
}

pub fn search_smart(query: &str) -> SearchStrategy {
    // AI가 쿼리 분석하여 최적 전략 선택
}
```

---

## 현실적인 접근

### 우리의 Use Case (junghanacs)

**80% 케이스: org-mcp-server로 충분**
```
- 태그 기반 파일 찾기
- Heading 구조 탐색
- 키워드 검색
- 최신 파일 우선
```

**20% 케이스: RAG 필요**
```
- 개념적 질문
- 탐색적 연구
- 동의어 처리
```

### 권장 구성

**기본 (org-mcp-server):**
```bash
# 빠르고 정확한 일상 작업
org-cli list --tags active
org-cli outline file.org
org-cli search "keyword"
```

**고급 (org-mcp + RAG):**
```python
# 복잡한 탐색
def hybrid_search(query):
    # 1. 태그 필터
    files = org_cli_list(tags=['research'])

    # 2. Semantic 검색
    docs = rag_search(query, files)

    # 3. 구조 확인
    for doc in docs:
        outline = org_cli_outline(doc)
        # line_number로 정확한 위치
```

---

## 성능 개선 로드맵

### 즉시 (1주 이내)

1. **rayon 병렬 검색**
   - search 함수 수정
   - 96초 → 10-15초
   - PR 가능

2. **Regex search 추가**
   - 새 함수 `search_regex`
   - 정확한 패턴 매칭
   - PR 가능

### 단기 (2-4주)

3. **파일 캐싱**
   - DashMap 기반
   - mtime 체크
   - 반복 검색 100배+ 개선

4. **RwLock 전환**
   - Arc<Mutex> → Arc<RwLock>
   - 동시 읽기 허용

### 중기 (1-2개월)

5. **메타데이터 인덱스**
   - 태그, 날짜, property 사전 추출
   - 필터링 속도 개선

6. **파일 워처**
   - notify 크레이트
   - 실시간 캐시 무효화

### 장기 (3개월+)

7. **RAG 통합 (선택)**
   - 임베딩 파이프라인
   - Hybrid search API
   - 의미 기반 랭킹

---

## 결론

### org-mcp-server의 역할

**핵심 가치:**
1. ✅ **실시간 정확성** - DB 불필요
2. ✅ **구조적 탐색** - heading + line_number
3. ✅ **빠른 필터링** - tags, properties
4. ✅ **키워드 정확도** - fuzzy + regex

**한계:**
1. ❌ 의미 이해 불가
2. ❌ 개념 기반 검색 제한

### RAG와의 관계

**상호 보완적:**
- org-mcp: **정확성 + 실시간**
- RAG: **의미 + 탐색**

**둘 다 필요하지만:**
- 80% 작업: org-mcp만으로 충분
- 20% 작업: RAG 추가로 완성

### 다음 단계

1. ✅ **rayon 병렬 검색 구현** (즉시 6-8배 개선)
2. ✅ **성능 테스트 CI 추가**
3. ⏳ **Upstream PR** (병렬 검색 + 벤치마크)
4. ⏳ **RAG 통합 설계** (별도 프로젝트)

---

**org-mcp-server = 빠르고 정확한 베이스레이어**
**RAG = 의미 기반 확장 레이어**

둘의 조합 = **완벽한 지식베이스 검색!**
