---
name: upstream
description: |
  Check upstream repo for new changes that may be worth incorporating. Use when:
  (1) User says /upstream
  (2) User asks to "check upstream" or "what changed upstream"
  (3) User wants to know if upstream repos have new commits
  (4) User asks about syncing with or pulling from upstream sources
---

# Upstream Change Tracker

Check the upstream repo that glean derives from for new commits, classify changes by relevance, and present a summary.

## Upstream Repo

| Repo | Default Branch | Relationship |
|------|---------------|-------------|
| `jahala/tilth` | `main` | Direct fork — glean's entire Rust codebase derives from tilth |

## Workflow

### Step 1: Read Marker File

Read `.claude/skills/upstream/references/last-checked.json`.

- **If the file is empty or does not exist** -> this is a first run. Set `FIRST_RUN=true`.
- **If the file exists with content** -> parse the JSON to get `last_checked_sha` and `last_checked_date`.

### Step 2: Fetch Changes

#### First Run (no marker file)

Fetch the last 30 commits:

```bash
gh api "repos/jahala/tilth/commits?per_page=30&sha=main" --jq '[.[] | {sha: .sha, date: .commit.committer.date, message: (.commit.message | split("\n") | .[0]), author: .commit.author.name}]'
```

Also fetch the changed files for each commit to classify relevance:

```bash
gh api "repos/jahala/tilth/commits?per_page=30&sha=main" --jq '[.[].sha]' | jq -r '.[]' | head -30 | while read sha; do gh api "repos/jahala/tilth/commits/$sha" --jq '{sha: .sha, files: [.files[].filename]}'; done
```

#### Subsequent Runs (marker file exists)

Use the compare API:

```bash
gh api "repos/jahala/tilth/compare/{LAST_SHA}...main" --jq '{total_commits: .total_commits, commits: [.commits[] | {sha: .sha, date: .commit.committer.date, message: (.commit.message | split("\n") | .[0]), author: .commit.author.name}], files: [.files[].filename]}'
```

**Fallback:** If the compare API returns 404 (e.g. force-push rewrote history), fall back to date-based query:

```bash
gh api "repos/jahala/tilth/commits?since={LAST_DATE}&sha=main&per_page=100" --jq '[.[] | {sha: .sha, date: .commit.committer.date, message: (.commit.message | split("\n") | .[0]), author: .commit.author.name}]'
```

### Step 3: Classify Changed Files by Relevance

Use these mappings to assign HIGH / MEDIUM / LOW relevance to each changed file:

| Relevance | Path Patterns |
|-----------|--------------|
| **HIGH** | `src/**/*.rs` (all Rust source — direct counterparts in glean) |
| **MEDIUM** | `Cargo.toml`, `Cargo.lock`, `benchmark/**`, `flake.nix` |
| **LOW** | `.github/**`, `README.md`, `AGENTS.md`, `LICENSE`, `scripts/**` |

Files not matching any pattern -> **MEDIUM** (unknown = worth reviewing).

### Step 4: Present Summary

Format the output as follows:

```
# Upstream Changes

## jahala/tilth (N new commits since YYYY-MM-DD)

### Commits
- `abc1234` Fix tree-sitter parsing for Go — @author (2025-05-01)
- `def5678` Add support for Zig language — @author (2025-04-28)

### Changed Files

**HIGH relevance** (Rust source — direct counterparts in glean):
- src/search/symbol.rs
- src/read/mod.rs

**MEDIUM relevance** (may affect behavior):
- Cargo.toml

**LOW relevance** (infrastructure/docs):
- README.md

### Assessment
2 high-relevance changes to Rust source — worth reviewing for potential incorporation.
```

If there are **no new commits**, show:

```
## jahala/tilth — No new commits since last check (YYYY-MM-DD)
```

### Step 5: Update Marker File

Build the new marker JSON with the HEAD SHA and current date.

- **First run:** Write the marker file automatically (tell the user it was created).
- **Subsequent runs:** Ask the user "Update the last-checked marker to current HEAD?" before writing.

Write to `.claude/skills/upstream/references/last-checked.json`:

```json
{
  "jahala/tilth": {
    "last_checked_sha": "<HEAD_SHA>",
    "last_checked_date": "<ISO_DATE>"
  }
}
```
