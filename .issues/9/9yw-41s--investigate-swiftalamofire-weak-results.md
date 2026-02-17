---
# 9yw-41s
title: Investigate Swift/Alamofire weak results
status: completed
type: task
priority: high
created_at: 2026-02-16T01:07:56Z
updated_at: 2026-02-16T01:12:17Z
parent: 4q3-09c
sync:
    github:
        issue_number: "23"
        synced_at: "2026-02-17T00:09:01Z"
---

## Problem

Glean shows minimal benefit on Swift (Alamofire) tasks. Context/cost improvements are modest and correctness drops on 3 of 6 tasks:

| Task | Ctx Δ | Cost Δ | B✓ | G✓ |
|------|-------|--------|-----|-----|
| af_acceptable_status | -8% | -13% | 3/3 | 3/3 |
| af_interceptor_protocol | -14% | -25% | 3/3 | 2/3 |
| af_request_chain | +30% | -17% | 3/3 | 2/3 |
| af_response_validation | -6% | +5% | 3/3 | 3/3 |
| af_session_config | -25% | -26% | 1/3 | 0/3 |
| af_upload_multipart | +10% | +5% | 3/3 | 3/3 |

## Investigation

- [ ] Check glean outline quality for Alamofire files — run `glean Session.swift` and compare outline to actual file structure
- [ ] Check if Swift tree-sitter definitions are being detected correctly for Alamofire patterns (protocols, extensions, generics)
- [ ] Look at af_session_config failure: what is 'Missing: Session.swift'? Is this the same filename-not-echoed problem as the zod tasks we fixed?
- [ ] Look at af_request_chain +30% context regression — what is the model reading that baseline doesn't?
- [ ] Compare glean_search results for Swift symbols vs TypeScript symbols — is Swift symbol detection less precise?

## Findings

### .build directory polluting Alamofire search results (FIXED)

Swift Package Manager \`.build/\` directory contained build artifacts (plugin-tools.yaml with every source file listed). This was being searched because:
- \`.build\` was not in SKIP_DIRS
- glean intentionally ignores .gitignore (\`git_ignore(false)\`)

### .xcodeproj/project.pbxproj (218KB) in search results (FIXED)

The pbxproj file mentions every source file by name, polluting content search results. Added \`.xcodeproj\` and \`.xcworkspace\` to SKIP_DIRS.

### Documentation ranked above source code (NEW ISSUE)

After fixing the above, \`glean "Session"\` returns Documentation/AdvancedUsage.md examples as top "definitions" instead of the actual Session class in Source/Core/Session.swift. Markdown code block examples are being classified as definitions. This is a search ranking issue that needs investigation.

### Fixes applied
- [x] Added \`.build\` to SKIP_DIRS
- [x] Added \`.xcodeproj\` and \`.xcworkspace\` to SKIP_DIRS  
- [x] Removed stale \`.build\` directory from alamofire fixture
