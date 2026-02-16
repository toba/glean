---
# jz9-coo
title: Investigate correctness regressions in glean mode
status: ready
type: task
priority: normal
created_at: 2026-02-16T01:08:07Z
updated_at: 2026-02-16T01:08:07Z
parent: 4q3-09c
---

## Problem

Several tasks where baseline gets 100% correctness drop when using glean:

| Task | B✓ | G✓ | Failure reason |
|------|-----|-----|---------------|
| rg_binary_detection | 3/3 | 1/3 | Unknown — need to check |
| rg_lineiter_definition | 3/3 | 0/3 | 0% glean correctness — complete failure |
| af_interceptor_protocol | 3/3 | 2/3 | Missing: RequestInterceptor (1-turn flakes) |
| af_request_chain | 3/3 | 2/3 | Unknown |
| rg_trait_implementors | 3/3 | 2/3 | Unknown |

## Investigation

- [ ] rg_lineiter_definition: 100% baseline → 0% glean is the worst regression. Check tool sequences — what is glean doing differently?
- [ ] rg_binary_detection: 100% → 33%. Check the 2 failing glean runs — did glean_search miss key files?
- [ ] Check if glean_search results are displacing information the model needs. Hypothesis: glean's expanded definitions fill context with tangential code, crowding out the specific details the correctness check requires
- [ ] For 1-turn failures (af_interceptor_protocol): likely model flakes, not glean issues
