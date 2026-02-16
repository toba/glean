---
# 92m-xvs
title: Add file locking or per-process temp files for JSONL output
status: completed
type: bug
priority: normal
created_at: 2026-02-15T23:46:34Z
updated_at: 2026-02-15T23:49:54Z
---

When run_full.sh launches multiple bench run processes in parallel, they all append to the same JSONL file concurrently. This causes interleaved writes that corrupt JSON lines (3 corrupted lines observed in benchmark_20260215_155751_opus.jsonl). Either use file locking (flock) when appending, or write to per-process temp files and merge after all groups finish.

## Summary of Changes\n\nAdded --output/-o flag to bench run so callers can specify the output file path. Updated run_full.sh to write each parallel repo group to its own temp file (in a mktemp -d directory), then merge them into a single JSONL after all groups finish. This eliminates the concurrent-write corruption that occurred when multiple processes appended to the same file simultaneously.
