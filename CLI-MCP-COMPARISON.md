# org-cli vs org-mcp-server 기능 비교

> 작성일: 2025-11-25
> 목적: CLI 도구와 MCP 서버 간 기능 동등성 검증 및 개선사항 파악

## 개요

`org-mcp-server` 프로젝트는 두 가지 인터페이스를 제공합니다:
- **org-cli**: 터미널에서 직접 사용하는 CLI 도구
- **org-mcp-server**: MCP 프로토콜로 LLM과 연동하는 서버
  - **Tools**: 능동적 작업 (검색, 필터링)
  - **Resources**: 읽기 전용 접근 (URI 기반)

## 기능 매트릭스

| 기능 | org-cli | MCP Tools | MCP Resources |
|------|---------|-----------|---------------|
| **파일 목록** | `list` | `org-file-list` | `org://` |
| **파일 읽기** | `read <file>` | - | `org://{file}` |
| **아웃라인** | `outline <file>` | - | `org-outline://{file}` |
| **헤딩 추출** | `heading <file> <heading>` | - | `org-heading://{file}#{heading}` |
| **ID로 접근** | `element-by-id <id>` | - | `org-id://{id}` |
| **검색** | `search <query>` | `org-search` | - |
| **아젠다** | `agenda list/today/week/range` | `org-agenda` | `org-agenda://` 시리즈 |
| **Multi-Silo** | ❌ (부분) | ✅ (file-list만) | ❌ |
| **설정 보기** | `config show` | - | - |

## 세부 비교

### 1. 파일 목록

#### org-cli: `org list [OPTIONS]`
```bash
org list                    # 전체 목록
org list -t tag1,tag2       # 태그 필터
org list -f json            # JSON 출력
```

**옵션**:
- `-t, --tags <TAGS>`: 태그 필터 (콤마 구분)
- `-f, --format <FORMAT>`: 출력 포맷 (plain/json)

#### MCP Tool: `org-file-list`
```json
{
  "tags": ["tag1", "tag2"],
  "limit": 100,
  "all_silos": true
}
```

**파라미터**:
- `tags`: 태그 필터 (배열)
- `limit`: 결과 개수 제한
- `all_silos`: **Multi-Silo 검색** (primary + extra + discovered repos)

#### MCP Resource: `org://`
```
org://
```

**동작**: 기본 디렉토리의 모든 org 파일 나열

#### 차이점
- ✅ MCP Tool은 `all_silos` 지원 (v0.0.4+)
- ❌ org-cli는 `all_silos` 옵션 없음 → **개선 필요**
- ❌ MCP Resource는 multi-silo 미지원

---

### 2. 검색

#### org-cli: `org search [OPTIONS] <QUERY>`
```bash
org search "keyword"                    # 기본 검색
org search -t tag1,tag2 "keyword"       # 태그 필터
org search -l 10 -s 200 "keyword"       # 제한/스니펫
org search -f json "keyword"            # JSON 출력
```

**옵션**:
- `<QUERY>`: 검색어
- `-l, --limit <LIMIT>`: 결과 개수 제한
- `-s, --snippet-size <SIZE>`: 스니펫 크기 (기본 100)
- `-t, --tags <TAGS>`: 태그 필터
- `-f, --format <FORMAT>`: 출력 포맷

#### MCP Tool: `org-search`
```json
{
  "query": "keyword",
  "limit": 10,
  "snippet_max_size": 200,
  "tags": ["tag1", "tag2"]
}
```

**파라미터**:
- `query`: 검색어 (필수)
- `limit`: 결과 개수 제한
- `snippet_max_size`: 스니펫 크기
- `tags`: 태그 필터

#### 차이점
- ❌ org-cli는 **단일 silo**만 검색 (org_directory)
- ❌ MCP Tool도 **단일 silo**만 검색 → **개선 필요**
- ⚠️ Multi-Silo 검색을 위해 `all_silos` 옵션 추가 필요

---

### 3. 아젠다 (TODO/Task 관리)

#### org-cli: `org agenda <SUBCOMMAND> [OPTIONS]`

**List 모드**: 전체 태스크 목록
```bash
org agenda list                         # 모든 태스크
org agenda list -s TODO,DONE            # 상태 필터
org agenda list -t project,urgent       # 태그 필터
org agenda list -p A                    # 우선순위 필터
org agenda list -l 10                   # 개수 제한
org agenda list -f json                 # JSON 출력
```

**View 모드**: 날짜 기반 뷰
```bash
org agenda today                        # 오늘
org agenda week                         # 이번 주
org agenda range -s 2025-01-01 -e 2025-01-31  # 날짜 범위
```

**옵션**:
- `-s, --states <STATES>`: TODO 상태 필터
- `-t, --tags <TAGS>`: 태그 필터
- `-p, --priority <A|B|C>`: 우선순위
- `-l, --limit <LIMIT>`: 결과 제한
- `-f, --format <FORMAT>`: 출력 포맷

#### MCP Tool: `org-agenda`
```json
{
  "mode": "list",
  "todo_states": ["TODO", "DONE"],
  "tags": ["project", "urgent"],
  "priority": "A",
  "limit": 10
}
```

```json
{
  "mode": "view",
  "start_date": "2025-01-01",
  "end_date": "2025-01-31",
  "tags": ["project"]
}
```

**파라미터**:
- `mode`: "list" (전체) / "view" (날짜 뷰)
- `start_date`: 시작 날짜 (YYYY-MM-DD)
- `end_date`: 종료 날짜 (YYYY-MM-DD)
- `todo_states`: 상태 필터
- `tags`: 태그 필터
- `priority`: 우선순위 (A/B/C)
- `limit`: 결과 제한

#### MCP Resources: `org-agenda://`

**고정 뷰**:
```
org-agenda://           # 전체 목록
org-agenda://today      # 오늘
org-agenda://week       # 이번 주
```

**날짜 기반 뷰**:
```
org-agenda://day/2025-01-15
org-agenda://week/3
org-agenda://month/1
org-agenda://query/from/2025-01-01/to/2025-01-31
```

#### 차이점
- ✅ 기능 동등 (CLI와 MCP 모두 동일한 기능 제공)
- CLI는 서브커맨드 방식, MCP는 `mode` 파라미터 방식

---

### 4. CLI 전용 기능

#### 파일 읽기: `org read <FILE>`
```bash
org read notes/task.org
```

**MCP 대응**: Resource `org://{file}`

---

#### 아웃라인: `org outline [OPTIONS] <FILE>`
```bash
org outline notes/task.org          # Plain 출력
org outline -f json notes/task.org  # JSON 출력
```

**MCP 대응**: Resource `org-outline://{file}`

---

#### 헤딩 추출: `org heading <FILE> <HEADING>`
```bash
org heading notes/task.org "Project Planning"
```

**MCP 대응**: Resource `org-heading://{file}#{heading}`

---

#### ID로 접근: `org element-by-id <ID>`
```bash
org element-by-id a1b2c3d4-e5f6-4a5b-8c7d-9e0f1a2b3c4d
```

**MCP 대응**: Resource `org-id://{id}`

---

#### 설정 보기: `org config show`
```bash
org config show
```

**MCP 대응**: ❌ 없음 (필요시 추가)

---

## Multi-Silo 지원 현황

### 구현 완료 (v0.0.4+)

| 위치 | 파일 | 상태 |
|------|------|------|
| Core | `org-core/src/config.rs` | ✅ `org_extra_directories`, `org_silo_roots` |
| Core | `org-core/src/config.rs` | ✅ `discover_repo_docs()` (Git 리포 자동 발견) |
| Core | `org-core/src/org_mode.rs` | ✅ `list_files_all_silos()` |
| Core | `org-core/src/denote.rs` | ✅ Denote 파일명 파싱 |
| MCP Tool | `org-mcp-server/src/tools/org_file_list.rs` | ✅ `all_silos` 옵션 |

### 미구현 (개선 필요)

| 위치 | 기능 | 우선순위 |
|------|------|----------|
| org-cli | `list --all-silos` | P1 (높음) |
| org-cli | `search --all-silos` | P1 (높음) |
| MCP Tool | `org-search` `all_silos` | P1 (높음) |
| MCP Resource | `org://` silo 파라미터 | P3 (낮음) |
| org-cli | `agenda --all-silos` | P2 (중간) |
| MCP Tool | `org-agenda` `all_silos` | P2 (중간) |

---

## 권장 개선 로드맵

### Phase 1: 필수 기능 (P1)
1. **org-cli `list --all-silos`** 추가
   - 파일: `org-cli/src/commands/list.rs`
   - 변경: `--all-silos` 플래그 추가, `list_files_all_silos()` 호출

2. **org-cli `search --all-silos`** 추가
   - 파일: `org-cli/src/commands/search.rs`
   - 변경: `--all-silos` 플래그 추가
   - Core: `org-core/src/org_mode.rs`에 `search_all_silos()` 구현 필요

3. **MCP Tool `org-search` all_silos** 추가
   - 파일: `org-mcp-server/src/tools/org_search.rs`
   - 변경: `all_silos: Option<bool>` 파라미터 추가
   - Core 의존: 위 #2 완료 후

### Phase 2: 부가 기능 (P2)
4. **org-cli `agenda --all-silos`** 추가
   - 파일: `org-cli/src/commands/agenda.rs`
   - Core: `list_tasks_all_silos()`, `get_agenda_view_all_silos()` 필요

5. **MCP Tool `org-agenda` all_silos** 추가
   - 파일: `org-mcp-server/src/tools/org_agenda.rs`

### Phase 3: 선택적 (P3)
6. **MCP Resource `org://` silo 지원**
   - 현재: `org://` (기본 디렉토리만)
   - 개선: `org://?silo=all` or `org-all://`

---

## 설계 원칙

### 1. 기능 동등성
- CLI와 MCP는 **동일한 핵심 기능** 제공
- 차이는 인터페이스 방식뿐 (터미널 vs JSON-RPC)

### 2. 단일 소스 진실 (Single Source of Truth)
- 모든 비즈니스 로직은 **org-core**에 구현
- CLI/MCP는 단순 래퍼 (Thin Wrapper)

### 3. Multi-Silo 전략
- **Primary Directory**: 기본 `org_directory` (필수)
- **Extra Directories**: 명시적 추가 디렉토리 (옵션)
- **Auto-Discovered Repos**: `org_silo_roots`에서 자동 발견 (옵션)
- **기본 동작 유지**: `all_silos=false` (하위 호환성)

### 4. 성능 고려사항
- Multi-Silo 검색은 **비용이 큼** (2000+ 파일)
- 사용자가 **명시적으로 요청**할 때만 활성화
- 향후 개선: rayon 병렬화, 캐싱

---

## 테스트 전략

### 기능 테스트
- [ ] CLI `list --all-silos` 동작 검증
- [ ] CLI `search --all-silos` 동작 검증
- [ ] MCP `org-search` all_silos 동작 검증
- [ ] Multi-Silo 결과 포맷 일관성 검증

### 성능 테스트
- [ ] 2140+ 파일 환경에서 벤치마크
- [ ] all_silos vs 단일 silo 성능 비교
- [ ] PERFORMANCE-ko.md 업데이트

### 통합 테스트
- [ ] CLI ↔ MCP 결과 동등성 검증
- [ ] Denote 메타데이터 일관성 검증

---

## 관련 문서
- **AGENTS.md**: 프로젝트 아키텍처 및 가이드라인
- **PERFORMANCE-ko.md**: 성능 벤치마크 결과
- **SEARCH-STRATEGY-ko.md**: 검색 전략 및 최적화
- **STRATEGY-ko.md**: 전체 프로젝트 전략

---

## 변경 이력
- 2025-11-25: 초안 작성 (Denote + Multi-Silo 구현 후)
