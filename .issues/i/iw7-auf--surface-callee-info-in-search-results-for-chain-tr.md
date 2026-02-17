---
# iw7-auf
title: Surface callee info in search results for chain tracing
status: completed
type: feature
priority: normal
created_at: 2026-02-17T02:51:46Z
updated_at: 2026-02-17T03:00:43Z
---

## Problem

When tracing a call chain (A calls B calls C), the model has to search each symbol individually to discover the next hop. Glean already has `callees.rs` that can extract what a function calls, but this isn't surfaced in search results.

## Expected

When a search result shows a definition, include a brief callee summary — e.g. for `DataRequest.validate()`, note that it calls `Validation.validate(request:)`. This lets the model follow a chain without extra round-trips.

## Approach

When formatting a definition match in symbol search, run the callee extractor on the definition body and append a `calls: [Validation.validate]` annotation. Keep it compact — just symbol names, no full bodies.

## Evidence

eval_swift_chain task traces Session.request() → DataRequest → validate() → Validation.acceptableStatusCodes across 3 files. Each hop required a separate search/read with glean, where baseline just read the files directly.

## Summary of Changes\n\nNo code changes needed. With dotted symbol search (issue 8zm-e4l) and small-file expansion bypass (issue u8m-0ll), definitions in small files are always expanded and the existing callee footer is emitted automatically.
