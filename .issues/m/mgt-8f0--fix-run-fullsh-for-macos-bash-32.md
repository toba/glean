---
# mgt-8f0
title: Fix run_full.sh for macOS bash 3.2
status: completed
type: bug
priority: normal
created_at: 2026-02-15T22:56:03Z
updated_at: 2026-02-15T22:57:32Z
sync:
    github:
        issue_number: "4"
        synced_at: "2026-02-17T00:08:58Z"
---

run_full.sh uses declare -A (associative arrays) which requires bash 4+. macOS ships with bash 3.2. Rewrite to use parallel arrays instead.

## Summary of Changes\n\nReplaced bash 4+ features (declare -A associative arrays, array +=, ${!array[@]}) with bash 3.2-compatible alternatives:\n- Named variables + eval-based repo_tasks() helper instead of associative array\n- Space-separated PID string instead of array+=\n- Simple for-in loop over PIDs instead of indexed array iteration
