---
# js8-zqh
title: Rename Context.Next() to Context.Continue()
status: completed
type: task
priority: normal
created_at: 2026-02-17T02:22:48Z
updated_at: 2026-02-17T03:05:46Z
---

Rename the Next() method on Context to Continue() in the mini-go test fixture. Update:
- context.go: method definition
- middleware.go: c.Next() call site
- router.go: c.Next() call site
- Update test files in symbol.rs, callers.rs that reference 'Next'

## Summary of Changes\n\nRenamed `Next()` to `Continue()` in mini-go fixture files:\n- context.go: method definition\n- middleware.go: `c.Continue()` call site\n- router.go: `c.Continue()` call site\n\nUpdated tests:\n- symbol.rs: `results_deduped_and_balanced` test\n- content.rs: `regex_search_finds_method_signature` test\n- callers.rs: both caller tests (pre-existing change)\n- rank.rs: comment update
