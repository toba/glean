# Glean

Glean is derived from [tilth](https://github.com/jahala/tilth) which was in turn inspired by Can Bölük's [Harness Problem](https://blog.can.ac/2026/02/12/the-harness-problem/) article.

Changes from `tilth`:
- Support for Swift and Zig
- Impl/trait detection — searching for a trait or interface surfaces all its implementors as definitions, not just string matches
- Faceted search grouping — results with more than 5 matches get sorted into Definitions, Implementations, Tests, and Usages sections
- Relative paths in search output (because nobody needs to see your home directory)
- Rewrote the benchmark tool in Rust
- Various code optimizations of dubious value
- Fixed reference to some guy's personal home directory
- Changed benchmark projects to Go, Rust, TypeScript and Swift
- More complex benchmarks (like expecting call tree navigation)
- Available via Homebrew

Like `tilth`, Glean combines tree-sitters and fast file searching so LLM agents spend less time (and less token dollars!) bumbling around your code like fools. I know non-software people think these thing are amazing—and they are (em-dash!)—but the rest of us watch with some horror as these tools consistently do the same wrong thing five times before getting it right, again and again, all day long, no matter how many times and ways we elucidate the path of righteousness.

Sure, props for persistence, and *maybe* getting it right eventually, but I'd rather fritter away tokens on *my own* foolish ideas, not unholy agent ineptitude. *Glean* may help.

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

Inspect a range or heading with `--section`:

```bash
$ glean src/auth.ts --section 44-89
$ glean docs/guide.md --section "## Installation"
```

### Search

Tree-sitters (language awareness) for *Rust*, *TypeScript*, *JavaScript*, *Python*, *Go*, *Java*, *C*, *C++*, *Ruby*, *Zig* and *Swift* find where symbols are **defined**, not just where strings appear. They also list the file, range and signature of callers and callees so agents can follow call chains without more searching.

Searching for a trait or interface name surfaces all its implementors — `impl Display for MyType`, `class Foo implements Serializable`, etc. — as first-class definitions. No more hoping ripgrep lands on the right line.

When results exceed 5 matches, they're grouped into **Definitions**, **Implementations**, **Tests**, and **Usages** sections so agents (and humans squinting at tool output) can orient quickly.

```bash
$ glean handleAuth --scope src/
# Search: "handleAuth" in src/ — 8 matches (2 definitions, 6 usages)

### Definitions

## auth.ts:44-89 [definition]
  [24-42]  fn validateToken(token: string)
→ [44-89]  export fn handleAuth(req, res, next)
  [91-120] fn refreshSession(req, res)

  44 │ export function handleAuth(req, res, next) {
  45 │   const token = req.headers.authorization?.split(' ')[1];
  ...
  88 │   next();
  89 │ }

── calls ──
  validateToken  auth.ts:24-42  fn validateToken(token: string): Claims | null
  refreshSession  auth.ts:91-120  fn refreshSession(req, res)

### Usages

## routes/api.ts:34 [usage]
→ [34]   router.use('/api/protected/*', handleAuth);
```

### Edit mode

Pass `--edit` when running `glean install` (e.g. `glean install claude-code --edit`) to add `glean_edit` and switch `glean_read` to hashline output:

```
42:a3f|  let x = compute();
43:f1b|  return x;
```

This allows edits to be anchored to content rather than ephemeral line numbers. If a hash doesn't match, the lines are reread, preventing code corruption when multiple agents are active.

```json
{
  "path": "src/auth.ts",
  "edits": [
    { "start": "42:a3f", "content": "  let x = recompute();" },
    { "start": "44:b2c", "end": "46:e1d", "content": "" }
  ]
}
```

Edit mode is valuable when the agent's built-in file editing is imprecise — large files, repetitive code, or multi-site edits. For small files and edits, the hashing overhead is unjustified.

### DRY

In MCP mode, previously expanded definitions show `[shown earlier]` instead of the full body on subsequent searches. This saves tokens when the agent revisits symbols it already saw.

## Benchmarks

26 tasks across 4 real repos (Gin, ripgrep, Alamofire, Zod), 3 reps each, Opus 4.6. Full methodology and per-task results in [benchmark/README.md](./benchmark/README.md).

| Model | Tasks | Baseline $/correct | glean $/correct | Change | Baseline acc | glean acc |
|---|---|---|---|---|---|---|
| Opus 4.6 | 26 (78 runs/mode) | $0.61 | $0.66 | **+9%** | 85% | 76% |
| Haiku 4.5 | 7 forced* (7 runs) | $0.22 | $0.04 | **-82%** | 69% | 100% |

Glean reduces raw cost by 3% on Opus but accuracy drops from 85% to 76%, so cost per correct answer — the metric that matters — is 9% worse. The value proposition is unproven on frontier models.

Haiku forced mode is the one clear win: removing built-in search tools and forcing glean adoption produces 100% accuracy at dramatically lower cost. But that's 7 runs, so take it with appropriate salt.

### Tilth results

[Tilth](https://github.com/jahala/tilth) benchmarked on different tasks (Express, FastAPI, Gin, ripgrep). Their results are more favorable to MCP-assisted search:

| Model | Tasks | Baseline $/correct | tilth $/correct | Change | Baseline acc | tilth acc |
|---|---|---|---|---|---|---|
| Sonnet 4.5 | 21 (126 runs) | $0.31 | $0.23 | **-26%** | 79% | 86% |
| Opus 4.6 | 6 hard (36 runs) | $0.49 | $0.42 | **-14%** | 83% | 78% |

### Smaller models

Smaller models (e.g. Haiku) may ignore glean tools in favor of built-in Bash/Grep. To force glean adoption, disable the overlapping built-in tools:

```bash
claude --disallowedTools "Bash,Grep,Glob"
```

This improved Haiku accuracy from 69% to 100% and reduced cost per correct answer by 82% in our (small) benchmark.

## What's inside

Quantum processing imported from `futures`, roughly 2038.
