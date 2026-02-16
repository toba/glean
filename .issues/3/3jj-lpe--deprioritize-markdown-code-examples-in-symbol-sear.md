---
# 3jj-lpe
title: Deprioritize Markdown code examples in symbol search ranking
status: ready
type: bug
priority: high
created_at: 2026-02-16T01:14:09Z
updated_at: 2026-02-16T01:14:09Z
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
