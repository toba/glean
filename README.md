# Glean

Glean is derived from [tilth](https://github.com/jahala/tilth) which was in turn inspired by Can Bölük's [Harness Problem](https://blog.can.ac/2026/02/12/the-harness-problem/) article.

Changes from `tilth`:
- Support for Swift
- Rewrote the benchmark tool in Rust
- Various code optimizations of dubious value
- Available via Homebrew

Like `tilth`, Glean combines tree-sitters and fast file searching so LLM agents spend less time (and less token dollars!) bumbling around your code like fools. I know non-software people think these thing are amazing—and they are (em-dash!)—but the rest of us watch with some horror as these tools consistently do the same wrong thing five times before getting it right, again and again, all day long, no matter how many times and  ways we elucidate the path of righteousness.

Sure, props for persistence, and *maybe* getting it right eventually, but I'd rather fritter away my tokens on *my own* foolish ideas, not unholy agent ineptitude. *Glean* may help.

## Usage

### Install

```bash
brew install toba/tap/glean
```

### MCP server

```bash
glean install claude-code      # ~/.claude.json
glean install cursor           # ~/.cursor/mcp.json
glean install windsurf         # ~/.codeium/windsurf/mcp_config.json
glean install vscode           # .vscode/mcp.json (project scope)
glean install claude-desktop
```

Add `--edit` to enable hash-anchored file editing (see [Edit mode](#edit-mode)):

```bash
glean install claude-code --edit
```

### CLI

Hopefully it's your agent typing this for you.

```bash
glean <path>                      # read file (outline if large)
glean <path> --section 45-89      # exact line range
glean <path> --section "## Foo"   # markdown heading
glean <path> --full               # force full content
glean <symbol> --scope <dir>      # definitions + usages
glean "TODO: fix" --scope <dir>   # content search
glean "/<regex>/" --scope <dir>   # regex search
glean "*.test.ts" --scope <dir>   # glob files
glean --map --scope <dir>         # codebase skeleton (CLI only)
```

### Example
```bash
$ glean src/auth.ts
# src/auth.ts (258 lines, ~3.4k tokens) [outline]

[1-12]   imports: express(2), jsonwebtoken, @/config
[14-22]  interface AuthConfig
[24-42]  fn validateToken(token: string): Claims | null
[44-89]  export fn handleAuth(req, res, next)
[91-258] export class AuthManager
  [99-130]  fn authenticate(credentials)
  [132-180] fn authorize(user, resource)
```

## How it works

### Reading files

Small files come back whole. Large files get a structural outline — token-based, not line-based, so a 1-line minified bundle gets outlined while a 120-line focused module prints whole.

| Input | Behaviour |
|-------|-----------|
| 0 bytes | `[empty]` |
| Binary | `[skipped]` with mime type |
| Generated (lockfiles, .min.js) | `[generated]` |
| < ~3500 tokens | Full content with line numbers |
| > ~3500 tokens | Structural outline with line ranges |

Drill into any range or heading with `--section`:

```bash
$ glean src/auth.ts --section 44-89
$ glean docs/guide.md --section "## Installation"
```

### Search

Tree-sitter finds where symbols are **defined**, not just where strings appear. Definitions sort first; each match shows its surrounding file structure for immediate context.

```bash
$ glean handleAuth --scope src/
# Search: "handleAuth" in src/ — 6 matches (2 definitions, 4 usages)

## src/auth.ts:44-89 [definition]
  [24-42]  fn validateToken(token: string)
→ [44-89]  export fn handleAuth(req, res, next)
  [91-120] fn refreshSession(req, res)

  44 │ export function handleAuth(req, res, next) {
  45 │   const token = req.headers.authorization?.split(' ')[1];
  ...
  88 │   next();
  89 │ }

── calls ──
  validateToken  src/auth.ts:24-42  fn validateToken(token: string): Claims | null
  refreshSession  src/auth.ts:91-120  fn refreshSession(req, res)

## src/routes/api.ts:34 [usage]
→ [34]   router.use('/api/protected/*', handleAuth);
```

Expanded definitions include a **callee footer** (`── calls ──`) with resolved file, line range, and signature — the agent follows call chains without separate searches.

**Multi-symbol**: `glean "ServeHTTP, HandlersChain, Next" --scope .` — each symbol gets its own result block; the expand budget is shared and deduped across files.

**Callers**: `glean isTrustedProxy --kind callers --scope .` — structural tree-sitter matching finds call sites (not text matches).

### Edit mode

Install with `--edit` to add `glean_edit` and switch `glean_read` to hashline output:

```
42:a3f|  let x = compute();
43:f1b|  return x;
```

`glean_edit` uses these hashes as anchors. If the file changed since the last read, hashes won't match and the edit is rejected with current content shown:

```json
{
  "path": "src/auth.ts",
  "edits": [
    { "start": "42:a3f", "content": "  let x = recompute();" },
    { "start": "44:b2c", "end": "46:e1d", "content": "" }
  ]
}
```

### Session dedup

In MCP mode, previously expanded definitions show `[shown earlier]` instead of the full body on subsequent searches. Saves tokens when the agent revisits symbols it already saw.

## Speed

CLI times on Apple Silicon Mac, 26–1060 file codebases. Includes ~17ms process startup (MCP mode pays this once).

| Operation | ~30 files | ~1000 files |
|-----------|-----------|-------------|
| File read + type detect | ~18ms | ~18ms |
| Code outline (400 lines) | ~18ms | ~18ms |
| Symbol search | ~27ms | — |
| Content search | ~26ms | — |
| Glob | ~24ms | — |
| Map (codebase skeleton) | ~21ms | ~240ms |

Search, content search, and glob use early termination — time is roughly constant regardless of codebase size. `--map` is CLI-only — benchmarks showed agents overused it, hurting accuracy.

## Benchmarks

Code navigation tasks across 4 real-world repos (Express, FastAPI, Gin, ripgrep). Baseline = Claude Code built-in tools. glean = built-in tools + glean MCP server. We report **cost per correct answer** (`total_spend / correct_answers`) — the expected cost under retry. See [benchmark/](benchmark/) for full methodology.

| Model | Tasks | Baseline $/correct | glean $/correct | Change | Baseline acc | glean acc |
|---|---|---|---|---|---|---|
| Sonnet 4.5 | 21 (126 runs) | $0.31 | $0.23 | **-26%** | 79% | 86% |
| Opus 4.6 | 6 hard (36 runs) | $0.49 | $0.42 | **-14%** | 83% | 78% |
| Haiku 4.5 | 7 forced* (7 runs) | $0.22 | $0.04 | **-82%** | 69% | 100% |

\*Haiku ignores glean tools when offered alongside built-in tools (9% adoption rate). In **forced mode** (`--disallowedTools "Bash,Grep,Glob"`), it adopts glean and results improve dramatically. See [Smaller models](#smaller-models).

See [benchmark/](benchmark/) for per-task results, by-language breakdowns, and model comparison.

### Smaller models

Smaller models (e.g. Haiku) may ignore glean tools in favor of built-in Bash/Grep. To force glean adoption, disable the overlapping built-in tools:

```bash
claude --disallowedTools "Bash,Grep,Glob"
```

Benchmarks show this improves Haiku accuracy from 69% to 100% and reduces cost per correct answer by 82% on code navigation tasks.


## What's inside

Quantum processing imported from `futures`, roughly 2038.
