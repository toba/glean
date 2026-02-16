---
# 3jj-lpe
title: Deprioritize Markdown code examples in symbol search ranking
status: completed
type: bug
priority: high
created_at: 2026-02-16T01:14:09Z
updated_at: 2026-02-16T02:15:33Z
parent: 4q3-09c
---

## Problem

When searching for symbols like `Session` in the Alamofire repo, Documentation/AdvancedUsage.md code examples are ranked as top "definitions" above the actual `Session` class in Source/Core/Session.swift.

Markdown fenced code blocks containing usage examples (e.g. `let session = Session.default`) are being classified as definitions by the symbol search.

## Expected behavior

Actual source code definitions (class/struct/protocol declarations in .swift files) should rank above documentation examples.

## Possible fixes

- Deprioritize Markdown file matches in symbol ranking (rank.rs)
- Only classify definitions from Code file types, not Markdown
- Weight matches by FileType (Code > Markdown > Other)


## Summary of Changes

**Root cause**: `find_definitions` in `symbol.rs` ran a keyword heuristic fallback (`is_definition_line`) on ALL files without tree-sitter grammars, including Markdown. Lines like `class Session` inside fenced code blocks were classified as definitions.

**Fix**: Gate the heuristic fallback on `is_code` — only files with `FileType::Code(_)` get the heuristic. Markdown, StructuredData, Tabular, Log, and Other file types are now excluded.

**Files changed**:
- `src/search/symbol.rs` — added `is_code` check before heuristic fallback; added `markdown_code_examples_not_classified_as_definitions` test
- `tests/fixtures/mini-rust/README.md` — new fixture with code examples mentioning `Matcher`/`RegexMatcher`
