# org-mcp-server 설계 철학 비교

> 작성일: 2025-11-25  
> 목적: 원저자(Sebastian)의 전략과 우리(Junghan)의 전략 명확화

## 목차
1. [원저자 전략: 고성능 백엔드](#원저자-전략-고성능-백엔드)
2. [우리 전략: 메타데이터 기반 탐색](#우리-전략-메타데이터-기반-탐색)
3. [전략 비교 매트릭스](#전략-비교-매트릭스)
4. [통합 아키텍처](#통합-아키텍처)

---

## 원저자 전략: 고성능 백엔드

### 배경
- **작성자**: Sebastian Zaffarano (Elastic 소속)
- **에디터**: Neovim + nvim-orgmode + org-roam.nvim
- **문제 인식**: nvim-orgmode (Lua)의 성능 한계 (3000+ 파일)

### 핵심 목표
> **"MCP server for org-mode and org-roam knowledge base management"**

1. **고성능 Rust 백엔드**
   - nvim-orgmode (Lua) 대체
   - 대용량 파일 처리 (3000+)
   - nucleo-matcher 검색 엔진 (Neovim telescope와 동일)

2. **에디터 독립적 도구**
   - Neovim에서도 사용 가능
   - Emacs에서도 사용 가능
   - CLI로도 사용 가능
   - MCP 프로토콜 표준화

3. **org-roam 백링크 시스템**
   - 노드 간 연결 관리
   - ID 기반 참조
   - 지식 그래프 탐색

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
│ • 전체 파일 읽기         │
└──────────────────────────┘
         ↓
┌──────────────────────────┐
│ org files (plain text)   │
│ • ~/Documents/org.new/   │
│ • DB 없이 파일명으로     │
└──────────────────────────┘
```

### 접근 방식
- **전체 파일 읽기**: `fs::read_to_string()` - 전체 내용 파싱
- **orgize 파서**: 완전한 org-mode AST 생성
- **검색**: nucleo-matcher로 전문 검색
- **성능**: rayon 병렬화, 메모리 캐싱

### 제공 기능
1. **파일 전체 읽기** (`org://{file}`)
2. **아웃라인 추출** (`org-outline://{file}`)
3. **헤딩 내용 추출** (`org-heading://{file}#{heading}`)
4. **전문 검색** (`org-search`)
5. **아젠다 뷰** (`org-agenda`)

---

## 우리 전략: 메타데이터 기반 탐색

### 배경
- **작성자**: Junghan (Emacs + Denote 사용자)
- **에디터**: Emacs + Doom Emacs + Denote + memacs
- **문제 인식**: AI 에이전트의 비효율적 파일 읽기

### 핵심 목표
> **"Denote 메타데이터로 파일 내용 접근 최소화"**

1. **파일명 기반 정보 추출**
   - Denote 형식: `YYYYMMDDTHHMMSS--title__tags.org`
   - 파일 읽기 없이 메타데이터 확보
   - 타임스탬프, 제목, 태그, 시그니처

2. **헤딩 정보만으로 탐색**
   - `TreeNode` 구조: label + level + line_number
   - 파일 내용 읽지 않고 구조 파악
   - AI 에이전트가 필요한 부분만 접근

3. **DenoteID 전역 참조 시스템**
   - 파일 포맷 독립적 (org, md, txt 무관)
   - ID 기반 크로스 레퍼런스
   - 에이전트가 자연스럽게 이해

### 아키텍처 전략

```
┌──────────────────────────┐
│ AI Agent (Claude, etc)   │
│ • 파일명으로 선택        │
│ • 헤딩으로 범위 좁히기   │
│ • 필요시에만 내용 읽기   │
└──────────────────────────┘
         ↓ MCP
┌──────────────────────────┐
│ org-mcp-server (Rust)    │
│ • Denote 파싱            │
│ • TreeNode (line_number) │
│ • 부분 읽기 (100줄?)     │
└──────────────────────────┘
         ↓
┌──────────────────────────┐
│ Denote files             │
│ • 20251125T120000--...   │
│ • 파일명 = 메타데이터    │
└──────────────────────────┘
```

### 접근 방식
- **메타데이터 우선**: 파일명 + frontmatter + TreeNode
- **필요시 부분 읽기**: 앞 100줄 또는 특정 헤딩만
- **DenoteID 링크**: 파일 간 연결 추적
- **전역 ID 공간**: 모든 파일 포맷에서 일관된 ID

### 제공 기능 (필요)
1. **Denote 파일 목록** (`denote-list`) - 메타데이터 포함
2. **DenoteID 참조** (`denote://{id}`) - 파일 포맷 무관
3. **Heading 탐색** (`org-outline://{file}`) - 내용 읽기 전 구조 파악
4. **부분 읽기** (`org://{file}?lines=0-100`) - 선택적 접근

---

## 전략 비교 매트릭스

| 측면 | 원저자 (Sebastian) | 우리 (Junghan) |
|------|-------------------|---------------|
| **목표** | 고성능 백엔드 | AI 에이전트 효율성 |
| **에디터** | Neovim | Emacs + Denote |
| **파일 읽기** | 전체 읽기 (전문 검색) | 메타데이터 우선 (부분 읽기) |
| **파일명** | 자유 형식 | Denote 형식 (메타데이터) |
| **검색** | 내용 기반 (nucleo) | 메타+구조 기반 |
| **링크** | org-roam ID | DenoteID (범용) |
| **성능 포인트** | 병렬화, 캐싱 | 읽기 최소화 |
| **타겟 유저** | 에디터 사용자 | AI 에이전트 |

### 읽기 패턴 비교

#### 원저자: 전체 읽기
```rust
// org-core/src/org_mode.rs
pub fn read_file(&self, path: &str) -> Result<String, OrgModeError> {
    fs::read_to_string(full_path)  // 전체 파일 읽기
        .map_err(OrgModeError::IoError)
}
```

**사용 사례:**
- 전문 검색 (nucleo-matcher)
- 완전한 AST 파싱 (orgize)
- 에디터가 전체 내용 표시

#### 우리: 메타데이터 + 부분 읽기

**1단계 - 파일명 파싱** (파일 안 읽음):
```rust
// org-core/src/denote.rs
pub fn parse_filename(filename: &str) -> Option<DenoteFile> {
    // YYYYMMDDTHHMMSS--title__tags.org
    // → identifier, title, tags 추출
}
```

**2단계 - 헤딩 구조** (내용 최소 읽기):
```rust
// org-core/src/org_mode.rs
pub fn get_outline(&self, path: &str) -> Result<TreeNode, OrgModeError> {
    // TreeNode { label, level, line_number, line_end }
    // → 헤딩 구조만 추출 (내용은 안 읽음)
}
```

**3단계 - 필요시 부분 읽기** (TODO):
```rust
// 제안: org-core/src/org_mode.rs
pub fn read_file_lines(&self, path: &str, start: usize, end: usize) 
    -> Result<String, OrgModeError> {
    // 특정 줄 범위만 읽기
}
```

### AI 에이전트 워크플로우 비교

#### 원저자 방식 (현재):
```
1. org-search "keyword"     → 전체 파일 읽기 + 검색
2. org://file.org           → 전체 파일 읽기 (2000+ 줄도 전부)
3. 내용 분석                → AI 토큰 소비 큼
```

#### 우리 방식 (목표):
```
1. denote-list --tags ai    → 파일명만 파싱 (읽기 없음)
   → 20251125T120000--ai-strategy__ai_planning.org
   
2. org-outline://file.org   → 헤딩만 추출 (내용 읽기 없음)
   → * AI Strategy (1-10)
     ** Background (10-20)
     ** Goals (20-30)
   
3. org://file.org?lines=1-100  → 필요한 부분만 읽기
   → 앞 100줄만 (frontmatter + 첫 섹션)
   
4. denote://20251125T120000    → DenoteID로 직접 접근
   → 파일 포맷 무관 (org, md, txt)
```

---

## 통합 아키텍처

### 핵심 아이디어: 레이어드 접근

```
┌─────────────────────────────────────────────┐
│ Layer 0: Metadata (파일 읽기 없음)         │
│ • Denote filename parsing                   │
│ • File tags (#+filetags:)                  │
│ • DenoteID registry                         │
│ → denote-list, denote://{id}               │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ Layer 1: Structure (최소 읽기)             │
│ • TreeNode outline (heading only)           │
│ • Line numbers (line_number, line_end)     │
│ • Heading tags                              │
│ → org-outline://{file}                     │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ Layer 2: Partial Content (선택적 읽기)     │
│ • Specific heading content                  │
│ • Line range (1-100, 50-150)               │
│ • Property drawer only                      │
│ → org-heading://{file}#{heading}           │
│ → org://{file}?lines=1-100                 │
└─────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────┐
│ Layer 3: Full Content (전체 읽기)          │
│ • Complete file content                     │
│ • Full-text search                          │
│ • Orgize AST parsing                        │
│ → org://{file} (현재 구현)                 │
│ → org-search (현재 구현)                   │
└─────────────────────────────────────────────┘
```

### 레이어별 사용 시나리오

| Layer | 사용 시기 | 비용 | 정보량 |
|-------|----------|------|--------|
| 0: Metadata | 파일 선택 | 최소 | 제목, 태그, ID |
| 1: Structure | 범위 좁히기 | 낮음 | 헤딩 구조 |
| 2: Partial | 특정 섹션 읽기 | 중간 | 필요한 부분 |
| 3: Full | 전문 검색/분석 | 높음 | 전체 내용 |

### AI 에이전트 최적화 패턴

**좋은 패턴 (레이어드):**
```python
# 1. Metadata로 후보 추리기
files = denote_list(tags=["ai", "strategy"], limit=10)
# → 10개 파일명만 (파일 안 읽음)

# 2. Structure로 범위 좁히기
outline = org_outline(files[0])
# → 헤딩 구조만 (내용 안 읽음)

# 3. Partial로 필요한 부분만
content = org_read_lines(files[0], lines=(1, 100))
# → 앞 100줄만

# 4. Full은 꼭 필요할 때만
full = org_read_file(files[0])
# → 전체 내용 (최후 수단)
```

**나쁜 패턴 (현재 상황):**
```python
# 1. Search로 시작
results = org_search("ai strategy")
# → 모든 파일 읽음 + 검색 (비쌈!)

# 2. 각 파일 전체 읽기
for result in results:
    content = org_read_file(result.file)
    # → 또 전체 읽음 (중복!)
```

---

## 구현 로드맵

### Phase 1: Denote 메타데이터 (완료)
- [x] `denote.rs` 모듈 추가
- [x] `DenoteFile` 구조체 (identifier, title, tags)
- [x] `parse_filename()` 함수
- [x] `#+filetags:` frontmatter 파싱

### Phase 2: MCP 도구 추가 (진행중)
- [x] Multi-Silo 지원 (`list_files_all_silos()`)
- [ ] `denote-list` MCP 도구
  - `list_denote_files(tags, limit, all_silos)`
  - 파일명 파싱만, 내용 읽기 없음
  
- [ ] `denote://{id}` MCP 리소스
  - DenoteID로 파일 찾기 (모든 silo 검색)
  - 파일 포맷 무관 (org, md, txt)

### Phase 3: 부분 읽기 지원 (미래)
- [ ] `read_file_lines(path, start, end)` 함수
- [ ] `org://{file}?lines=1-100` 리소스
- [ ] OpenCode Read tool 통합 (기본 2000줄 제한)

### Phase 4: DenoteID 링크 추적 (미래)
- [ ] DenoteID 역색인 구축
- [ ] 백링크 탐색 (`denote-backlinks://{id}`)
- [ ] 링크 그래프 시각화

---

## 철학적 차이

### 원저자: "고성능 = 빠른 전체 처리"
- **관점**: 사용자가 파일 내용을 보고 싶어 한다
- **도구**: 에디터 (Neovim, Emacs) - 전체 내용 표시
- **최적화**: 병렬화, 캐싱으로 전체 읽기 속도 개선

### 우리: "효율성 = 필요한 만큼만 읽기"
- **관점**: AI 에이전트는 구조를 먼저 이해하고 싶어 한다
- **도구**: AI Agent (Claude, etc) - 토큰 비용 의식
- **최적화**: 메타데이터로 읽기 자체를 회피

### 통합 비전
**두 전략은 상호 보완적입니다:**

1. **에디터 사용자** → Layer 3 (Full Content) 선호
   - "파일 열어서 전부 보여줘"
   - 원저자 구현이 완벽히 대응

2. **AI 에이전트** → Layer 0-2 (Metadata → Partial) 선호
   - "먼저 구조 보고, 필요한 부분만 읽을게"
   - 우리 구현이 최적화 제공

3. **검색** → Layer 0 (Denote tags) + Layer 3 (Full-text)
   - Denote 메타 검색: 빠르고 정확 (파일명만)
   - 전문 검색: 느리지만 완전함 (현재 구현)

---

## 다음 단계

### 즉시 구현 (P1)
1. **denote-list MCP 도구**
   - 파일: `org-mcp-server/src/tools/denote_list.rs`
   - 기능: `list_denote_files(tags, limit, all_silos)`
   - 결과: `Vec<DenoteFile>` (파일명 파싱만)

2. **denote://{id} MCP 리소스**
   - 파일: `org-mcp-server/src/resources/denote_id.rs`
   - 기능: DenoteID로 파일 찾기 (모든 silo)
   - 리턴: 파일 경로 + 메타데이터

### 중기 구현 (P2)
3. **org://{file}?lines=start-end 확장**
   - 파일: `org-core/src/org_mode.rs`
   - 함수: `read_file_lines(path, start, end)`
   - URI: `org://file.org?lines=1-100`

4. **CLI 통합**
   - `org denote-list --tags ai`
   - `org denote-get 20251125T120000`

### 장기 구현 (P3)
5. **DenoteID 역색인**
   - 파일: `org-core/src/denote_index.rs`
   - 구조: `HashMap<DenoteID, PathBuf>`
   - 백링크 추적

6. **성능 벤치마크**
   - Metadata (Layer 0) vs Full (Layer 3)
   - 2140+ 파일 환경 측정
   - PERFORMANCE-ko.md 업데이트

---

## 관련 문서
- **STRATEGY-ko.md**: 원저자 분석 및 기여 전략
- **CLI-MCP-COMPARISON.md**: 기능 동등성 검증
- **PERFORMANCE-ko.md**: 성능 벤치마크 (예정)
- **AGENTS.md**: Claude 가이드라인

---

## 변경 이력
- 2025-11-25: 초안 작성 (전략 차이 명확화)
