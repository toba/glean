---
# 5eu-sr3
title: Rename Matcher trait to PatternMatcher
status: scrapped
type: task
priority: normal
created_at: 2026-02-17T02:24:33Z
updated_at: 2026-02-17T03:05:52Z
---

Rename the Matcher trait to PatternMatcher throughout the codebase:
- Trait definition in src/lib.rs
- impl block for RegexMatcher in src/lib.rs  
- Generic parameter in Searcher in src/searcher.rs
- impl block for Searcher in src/searcher.rs
- use statement in src/searcher.rs

## Reasons for Scrapping\n\nDuplicate of m9k-kf2.
