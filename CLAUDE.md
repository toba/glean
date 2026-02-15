# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is glean?

glean is a Rust CLI and MCP server for smart code reading. It combines tree-sitter AST parsing, ripgrep search, and token-aware file viewing into a single tool for AI agents. ~6,000 lines of Rust.

## Build & Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (LTO, stripped)
cargo build --profile fast           # Optimized but fast compile (for dev/benchmarking)
cargo test                           # Run all tests (inline #[cfg(test)] modules)
cargo fmt --check                    # Check formatting
cargo clippy -- -D warnings          # Lint (CI enforces zero warnings)
```

## Architecture

### Query Flow

All CLI queries go through `lib.rs:run()`:

```
classify(query) → QueryType → dispatch
  FilePath  → read::read_file()
  Glob      → search::search_glob()
  Symbol    → search::search_symbol()
  Content   → search::search_content()
  Fallthrough → try symbol, then content, then NotFound
```

### Classification (`classify.rs`)

Deterministic byte-pattern matching (no regex). Checks for glob metacharacters (`* ? { [`), path separators, dotfiles, numeric strings, and valid identifiers — in that order.

### Read Pipeline (`read/`)

Decision tree: directory → section param → empty → binary → generated → token estimate.
- **≤3500 tokens**: full content with line numbers
- **>3500 tokens**: language-specific smart outline via `read/outline/`

Outline strategies by file type:
- `code.rs` — tree-sitter AST extraction (functions, classes, imports with line ranges)
- `markdown.rs` — heading hierarchy
- `structured.rs` — JSON/TOML/YAML top-level keys
- `tabular.rs` — CSV/TSV column headers
- `fallback.rs` — head + tail for logs and other text

### Search Pipeline (`search/`)

- **Symbol search** (`symbol.rs`): tree-sitter definition detection + ripgrep usage search, run in parallel via `rayon::join`. Results merged and deduped.
- **Content search** (`content.rs`): ripgrep regex, supports `/regex/` syntax.
- **Callers** (`callers.rs`): structural tree-sitter reverse matching with `memchr` SIMD pre-filtering.
- **Callees** (`callees.rs`): extracted at expand time from definition bodies.
- **Ranking** (`rank.rs`): definitions first, then by distance to context file and file age.

### MCP Server (`mcp.rs`)

JSON-RPC 2.0 over stdio. Tools: `glean_read`, `glean_search`, `glean_files`, `glean_edit` (optional), `glean_session`. Stateful — maintains `Session` (tracks expanded definitions for dedup) and `OutlineCache` (mtime-invalidated, `DashMap`-backed).

### Edit Mode (`edit.rs`, `format.rs`)

Hash-anchored editing. `glean_read` emits `line:hash|` format where hash = FNV-1a truncated to 12 bits (3 hex chars). `glean_edit` validates hashes before applying edits — rejects if file changed since last read.

### Session Dedup (`session.rs`)

MCP mode tracks which definitions have been expanded. Re-expanding shows `[shown earlier]` instead of full body to save tokens.

### Cache (`cache.rs`)

Outline cache keyed by `(path, mtime)`. Uses `DashMap` entry API to avoid TOCTOU races. Stale entries (mtime changed) are never hit.

## Key Types (`types.rs`)

- `QueryType` — classified query variant (FilePath, Glob, Symbol, Content, Fallthrough)
- `Lang` — supported programming languages (compiler enforces exhaustive matching)
- `FileType` — determines outline strategy (Code, Markdown, StructuredData, Tabular, Log, Other)
- `ViewMode` — what kind of output was produced (Full, Outline, Keys, Section, etc.)
- `Match` / `SearchResult` — search results with definition ranges and ranking metadata
- `OutlineEntry` / `OutlineKind` — structured outline tree

## Clippy Configuration

The project uses `clippy::pedantic` with specific allows listed in `lib.rs`. CI runs `cargo clippy -- -D warnings`.

## Adding a New Language

Add an arm to `Lang` in `types.rs` — the compiler will flag every match that needs updating (classification, tree-sitter grammar init, outline extraction, definition detection).

## Benchmarks

Python scripts in `benchmark/` test against real repos (Express, FastAPI, Gin, ripgrep). Run with `python benchmark/run.py`. See `benchmark/README.md` for methodology and results.
