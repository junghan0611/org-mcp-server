# org-mcp-server 기여 전략

## 프로젝트 이해

### 원저자: Sebastian Zaffarano (Elastic)

**배경:**
- Elastic 소속 (검색/분석 엔진 전문가)
- **Neovim 사용자** (Vim이 아님!)
- NixOS 사용자
- org-mode 애호가

**에디터 스택:**
```
Neovim + nvim-orgmode + org-roam.nvim + telescope + nvim-mcp
```

**설정:**
- dotfiles: https://github.com/szaffarano/nix-dotfiles
- org 경로: `~/Documents/org.new/`
- org-roam 백링크 시스템 사용

### 왜 Vim 사용자가 org-mode를?

**org-mode 파일 포맷의 범용성:**
1. 에디터 독립적 (plain text)
2. Git 친화적
3. 구조화된 데이터 (heading, properties, links)
4. Zettelkasten/org-roam 백링크 시스템

**nvim-orgmode 생태계:**
- [nvim-orgmode/orgmode](https://github.com/nvim-orgmode/orgmode) - Lua 구현
- [chipsenkbeil/org-roam.nvim](https://github.com/chipsenkbeil/org-roam.nvim) - 백링크
- [telescope-orgmode](https://github.com/nvim-orgmode/telescope-orgmode.nvim) - fuzzy search

## 프로젝트 비전

### 목표
> "MCP server for org-mode and org-roam knowledge base management"

**해결하려는 문제:**
1. **nvim-orgmode (Lua)의 성능 한계**
   - 대용량 파일 (3000+) 처리 느림
   - 복잡한 쿼리 성능 제한

2. **AI 에이전트 연동**
   - MCP 프로토콜로 표준화
   - Emacs/Neovim 독립적
   - 백그라운드 고성능 처리

3. **에디터 독립적 org-mode 도구**
   - CLI로도 사용 가능
   - MCP 서버로도 사용 가능
   - 다양한 에디터에서 활용

### 아키텍처 전략

```
┌──────────────────────────┐
│ Frontend (UI)            │
│ • Neovim (nvim-orgmode)  │
│ • Emacs                  │
│ • CLI (org-cli)          │
└──────────────────────────┘
         ↓ MCP
┌──────────────────────────┐
│ org-mcp-server (Rust)    │
│ • 고성능 검색/파싱       │
│ • 백그라운드 처리        │
└──────────────────────────┘
         ↓
┌──────────────────────────┐
│ org files (plain text)   │
│ • ~/Documents/org.new/   │
│ • DB 없이 파일명으로     │
└──────────────────────────┘
```

## 성능 현황

### nvim-orgmode (Lua) 한계

**테스트 환경:**
- 3000+ org 파일
- treesitter 파싱
- Lua VM 성능

**예상 병목:**
- 파일 스캔: Lua I/O
- 파싱: treesitter (C) vs Lua 처리
- 검색: 순차 처리

### org-mcp-server (Rust) 잠재력

**현재 구현:**
- nucleo-matcher (Neovim telescope와 동일 엔진!)
- orgize 파서 (Rust)
- ignore 크레이트 (gitignore 지원)

**문제점:**
- ❌ 캐싱 없음
- ❌ 병렬 처리 미흡
- ❌ 인덱싱 없음

**개선 가능성:**
- ✅ rayon 병렬화
- ✅ DashMap 캐싱
- ✅ RwLock 동시 읽기

### 예상 성능 (추정)

| 작업 | nvim-orgmode (Lua) | org-mcp-server (현재) | org-mcp-server (최적화) |
|------|-------------------|----------------------|------------------------|
| 3000개 파일 스캔 | 5-10초 | 1-3초 | 100-500ms |
| Heading 추출 | 느림 | 빠름 | 매우 빠름 |
| 전문 검색 | 10-30초 | 3-5초 | 500ms-1초 |

## 우리의 위치

### junghanacs vs szaffarano

| 항목 | szaffarano | junghanacs |
|------|-----------|-----------|
| **에디터** | Neovim | Emacs (Doomemacs) |
| **org 경로** | ~/Documents/org.new/ | ~/org/ (3000+) |
| **시스템** | org-roam.nvim | Denote |
| **목적** | AI + Neovim 통합 | AI + Emacs + Life 통합 |
| **공통점** | **Rust 성능 + MCP + org 파일** | |

### 우리의 독특한 가치

**1. Denote 시스템**
```
YYYYMMDDTHHMMSS--title__tags.org
```
- DB 없는 파일명 기반 관리
- 여러 Silo 통합
- memacs-config의 life-integrated 접근

**2. 대용량 지식베이스**
- ~/org/ 3000+ 파일
- ~/claude-memory/
- ~/repos/gh/*/docs

**3. 실시간 Agent 협업**
- Agent 대량 생성 문서
- 빠른 정리/편집 필요
- DB 업데이트 지연 불가

## 기여 전략

### 원저자 로드맵 정렬

**Phase 2 (진행 중):**
- [x] Configuration support
- [x] Tag-based filtering
- [x] Agenda functionality
- [ ] **Link following (org-roam)** ← 원저자 관심!
- [ ] **Metadata caching** ← 원저자 계획!

### 우리의 기여 (ko 브랜치)

**완료:**
- ✅ Phase 1: Line Number 추가 (363-367번째 줄)
  - AI Agent 효율성 극대화
  - Neovim 점프 기능 지원 가능

**계획 (PR 가능성 높음):**

**1. 성능 최적화 (Phase 2 정렬)**
```rust
// 병렬 검색
files.par_iter()
    .filter_map(|f| search_file(f, query))
    .collect()

// 캐싱
DashMap<PathBuf, CachedFile>

// RwLock
Arc<RwLock<OrgMode>>
```

**2. 성능 벤치마크**
```rust
// benches/search_benchmark.rs
criterion::benchmark_group!(
    name = benches;
    config = Criterion::default();
    targets = search_1000_files, search_3000_files
);
```

**계획 (Fork 전용):**

**3. Denote 지원**
- 파일명 파싱
- frontmatter (#+identifier:, #+filetags:)
- denote 기반 필터링

**4. Silo 관리**
- 여러 디렉토리 통합
- Git repos/*/docs 자동 발견

### PR vs Fork 전략

**Upstream PR 제안:**
1. ✅ Line Number (Phase 1) - 새로운 가치!
2. ✅ 병렬 검색 (rayon)
3. ✅ 성능 벤치마크
4. ✅ 캐싱 레이어 (Phase 2)
5. ✅ RwLock 개선

**ko 브랜치 유지:**
1. ✅ Denote 특화 기능
2. ✅ Silo 관리
3. ✅ 한글화
4. ✅ memacs 통합

## 실행 계획

### 단기 (이번 주)

1. **문서화**
   - [x] STRATEGY-ko.md (이 문서)
   - [ ] PERFORMANCE-ko.md
   - [ ] CHANGELOG-ko.md

2. **성능 테스트**
   - [ ] ~/org/ 3000개 파일 벤치마크
   - [ ] outline 성능 (line number 포함)
   - [ ] search 성능

3. **커밋 정리**
   - [ ] Phase 1 완료 커밋
   - [ ] ko 브랜치 push

### 중기 (1-2주)

1. **성능 최적화 PR**
   - [ ] rayon 병렬 검색 구현
   - [ ] 벤치마크 추가
   - [ ] upstream PR 제출

2. **Phase 2 시작**
   - [ ] Denote 파일명 파싱
   - [ ] frontmatter 파싱
   - [ ] ko 브랜치에서 개발

### 장기 (1개월+)

1. **캐싱 레이어**
   - [ ] DashMap 기반 파일 캐시
   - [ ] 메타데이터 인덱스
   - [ ] 파일 워처 연동

2. **Silo 관리**
   - [ ] 다중 디렉토리 지원
   - [ ] 자동 발견

3. **Upstream 협업**
   - [ ] 이슈 참여
   - [ ] PR 리뷰
   - [ ] 커뮤니티 기여

## 참고 자료

**원저자 프로젝트:**
- org-mcp-server: https://github.com/szaffarano/org-mcp-server
- nix-dotfiles: https://github.com/szaffarano/nix-dotfiles
- top.nvim: https://github.com/szaffarano/top.nvim

**Neovim org 생태계:**
- nvim-orgmode: https://github.com/nvim-orgmode/orgmode
- org-roam.nvim: https://github.com/chipsenkbeil/org-roam.nvim
- nvim-mcp: https://github.com/linw1995/nvim-mcp

**우리 프로젝트:**
- memacs-config: ~/repos/gh/memacs-config
- doomemacs-config: ~/repos/gh/doomemacs-config
- denote-silo-dynamic.el: ~/repos/gh/doomemacs-config/+denote-silo-dynamic.el

## 성능 목표

### 벤치마크 타겟

**3000개 org 파일 기준:**

| 작업 | 목표 시간 | 현재 (추정) |
|------|----------|------------|
| 파일 목록 | < 100ms | ~1s |
| Heading 추출 (1개) | < 10ms | ~50ms |
| Heading 추출 (전체) | < 3s | ~10s |
| 전문 검색 | < 1s | ~5s |
| Tag 필터링 | < 500ms | ~2s |

### 최적화 우선순위

1. **병렬 검색** (rayon) - 즉각적인 3-5배 개선
2. **파일 캐싱** (DashMap) - 반복 검색 10배+ 개선
3. **RwLock** - 동시 읽기 허용
4. **메타데이터 인덱스** - 태그/날짜 필터링 개선

## 결론

**org-mcp-server는 Neovim 사용자를 위한 고성능 org-mode 백엔드입니다.**

우리의 기여:
1. ✅ Line Number - AI 에이전트와 Neovim 모두에게 유용
2. 🎯 성능 최적화 - 원저자의 핵심 관심사
3. 🎯 Denote 지원 - 범용성 확대

**Win-Win 협업 가능성이 매우 높습니다!**
