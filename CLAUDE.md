# Claude Guidelines for org-mcp-server

## Project Context

**Upstream**: https://github.com/szaffarano/org-mcp-server/ (Sebastian Zaffarano)
**Fork**: https://github.com/junghan0611/org-mcp-server/ (ko branch)
**Local**: ~/repos/3rd/org-mcp-server

MCP server for org-mode/roam knowledge base management in Rust. Multi-crate
workspace with:

- `org-core` — Business logic and org-mode functionality
- `org-mcp-server` — MCP protocol implementation
- `org-cli` — CLI tool for testing and direct usage

**Goal**: Provide search, content creation, and note linking with media
references for org-mode files.

### Original Author Background

**Sebastian Zaffarano** (Elastic employee)
- **Editor**: Neovim (not Emacs!)
- **Stack**: nvim-orgmode + org-roam.nvim + nvim-mcp + telescope
- **NixOS user**: https://github.com/szaffarano/nix-dotfiles
- **Why org-mode**: Editor-independent knowledge base + AI agent integration

### Our Fork (ko branch)

**Purpose**: Korean localization + performance improvements + Denote support

**Test Environment**:
- 2140+ org files in ~/org/ (notes/, meta/, bib/, llmlog/)
- Denote naming: `YYYYMMDDTHHMMSS--title__tags.org`
- Multiple silos: ~/org/, ~/claude-memory/, ~/repos/gh/*/docs

## Build Commands

### NixOS Build (Recommended)

```bash
./build-nixos.sh           # Build and install to ~/.local/bin
nix build .#default        # Build only
```

**Installed binaries:**
- `~/.local/bin/org-mcp-server`
- `~/.local/bin/org-cli`

### Cargo Build

- `cargo build` — Build all crates
- `cargo test` — Run all tests
- `cargo test <test_name>` — Run specific test
- `cargo test -p <crate_name>` — Test specific crate
- `cargo clippy` — Run linter
- `cargo fmt` — Format code
- `cargo run --example <name>` — Run playground examples
- `cargo run --bin org-cli` — Run CLI tool
- `cargo run --bin org-mcp-server` — Run MCP server

## Just Commands

Development tasks are managed with [just](https://github.com/casey/just) (available in nix devShell).

**Common commands:**
- `just` — Show all available commands
- `just build` — Build all crates
- `just test` — Run all tests
- `just dev` — Development workflow (format, lint, test, coverage)
- `just coverage-html` — Generate HTML coverage report
- `just lint` — Run clippy linter
- `just fmt` — Format code

**Coverage targets:**
- `just coverage` — Generate all coverage formats
- `just coverage-html` — HTML report in `coverage/html/`
- `just coverage-summary` — Terminal summary
- `just coverage-ci` — LCOV for CI
- `just coverage-json` — JSON format

**Quality checks:**
- `just check` — Run all quality checks (format check, lint, test, coverage)
- `just fmt-check` — Check code formatting without modifying

Run `just` to see all available commands with descriptions.

## Code Style & Preferences

- **Formatting**: Always use `just fmt` or `cargo fmt` before commits
- **Error handling**: Prefer explicit `Result<T, E>` over panics
- **String formatting**: Use `"string {var}"` over `"string {}", var`
- **Imports**: Standard library before external crates
- **Testing**: Use `assert_eq!` over `assert!`, add `#[cfg(test)]` modules
- **Functions**: Keep focused and well-documented

## Architecture

- **Rust 2024 edition** with async-first design using `tokio`
- **Examples** in `org-core/examples/` for dependency experimentation
- **Test fixtures** in `tests/fixtures/` for org/roam files
- **Key deps**: `orgize` (parsing), `walkdir` (file traversal), `clap` (CLI)

## Development Workflow

1. **Multi-crate changes**: Update workspace dependencies in root Cargo.toml
1. **New functionality**: Add to `org-core`, expose via `org-mcp-server` and `org-cli`
1. **Error handling**: Use custom error types, implement proper chaining
1. **File operations**: Validate paths at construction, not runtime
1. **Testing**: Create fixtures for complex org-mode files

## Behavioral Guidelines

- **Concise responses**: Be direct, avoid unnecessary explanations
- **File creation**: NEVER create files unless absolutely necessary
- **Commits**: Always sign with -S, never include Claude Code references
- **Code quality**: Run clippy and fmt before suggesting changes
- **Documentation**: Only create when explicitly requested

## Current Implementation Status

### Upstream Features (v0.0.4)
- ✅ Basic file listing with recursive directory traversal
- ✅ Error handling with custom types and proper chaining
- ✅ CLI tool with `list`, `outline`, `search`, `agenda` commands
- ✅ MCP server with JSON-RPC protocol
- ✅ Org-mode parsing (orgize 0.10.0-alpha.10)
- ✅ Full-text search with nucleo-matcher
- ✅ Tag-based filtering
- ✅ Agenda functionality

### Our Improvements (ko branch)

**Phase 1: Line Number Support** ✅ DONE (2025-11-22)
- ✅ TreeNode에 `line_number`, `line_end` 추가
- ✅ `get_outline()` 함수에서 headline position 추출
- ✅ `byte_offset_to_line_number()` 헬퍼 함수
- ✅ 테스트 검증 완료 (2140+ files)

**Key Changes:**
- File: `org-core/src/org_mode.rs`
- Lines: 58-70 (TreeNode struct), 336-343 (helper), 349-405 (get_outline)
- Commit: `16213e8`

**Phase 1.5: Performance Testing** 🚧 NEXT
- [ ] Benchmark with 2140+ org files
- [ ] PERFORMANCE-ko.md documentation
- [ ] rayon parallel search
- [ ] Upstream PR preparation

**Phase 2: Denote Support** 📝 PLANNED
- [ ] Filename parsing: `YYYYMMDDTHHMMSS--title__tags.org`
- [ ] Frontmatter parsing: `#+identifier:`, `#+filetags:`
- [ ] denote-list MCP tool
- [ ] denote://{id} resource

**Phase 3: Multi-Silo Support** 📝 PLANNED
- [ ] Multiple directory configuration
- [ ] Auto-discovery: `~/repos/*/docs`
- [ ] Unified search across silos

### Performance Goals

**Target (2140+ files):**
- File list: < 100ms
- Outline extraction: < 3s
- Full-text search: < 1s

**Planned Optimizations:**
- rayon parallel processing (3-5x improvement)
- DashMap caching (10x+ for repeated searches)
- RwLock for concurrent reads

## Key Documentation

- **STRATEGY-ko.md** — Project analysis and contribution strategy
- **inbox__human.org** — TODO tracking and next steps
- **README.md** — Original project documentation
