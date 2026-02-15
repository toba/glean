---
# yas-bde
title: Replace benchmark fixtures
status: completed
type: task
priority: normal
created_at: 2026-02-15T20:32:12Z
updated_at: 2026-02-15T20:35:59Z
---

Replace Python/JS benchmark fixtures with Swift/TypeScript. Remove synthetic fixture. Add Alamofire (Swift) and Zod (TypeScript) repos. Add edit tasks to ripgrep and gin.

## Tasks
- [x] Remove synthetic fixture (files, setup, config, run logic)
- [x] Remove fastapi and express (files, config, task registry)
- [x] Add Alamofire and Zod to config
- [x] Create alamofire.rs task file
- [x] Create zod.rs task file
- [x] Add edit tasks to ripgrep.rs and gin.rs
- [x] Generalize repo reset in run.rs
- [x] Update mod.rs registry and main.rs
- [x] Build and verify with cargo build, clippy, test


## Summary of Changes

Replaced Python/JS benchmark fixtures with Swift/TypeScript:
- Removed synthetic fixture (synthetic_content/, synthetic.rs, setup_synthetic)
- Removed fastapi and express repos and tasks
- Added Alamofire (Swift, pinned to 5.11.1) with 6 tasks: session config, request chain, response validation, interceptor protocol, multipart upload, edit encoding threshold
- Added Zod (TypeScript, pinned to v4.3.6) with 6 tasks: string schema, parse flow, error handling, discriminated union, transform/pipe, optional/nullable
- Added edit tasks to ripgrep (edit buffer capacity) and gin (edit multipart memory)
- Generalized repo reset in run.rs to work with any repo (not just synthetic)
- Removed --synthetic flag from Setup command
- All 4 repos: ripgrep (Rust), gin (Go), alamofire (Swift), zod (TypeScript)
- Total: 26 tasks across 4 repos
