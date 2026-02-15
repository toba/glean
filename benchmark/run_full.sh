#!/usr/bin/env bash
set -euo pipefail

# Reproduce the benchmark results from README.md (v0.3.2).
# Runs Sonnet, Opus, and Haiku in parallel.
#
# WARNING: This invokes claude -p ~200 times. Run outside of Claude Code.

cd "$(dirname "$0")"

BENCH="./target/release/bench"

if [[ ! -x "$BENCH" ]]; then
  echo "Building benchmark runner..."
  cargo build --release
fi

# Set up repos and synthetic fixture if needed
$BENCH setup --repos
$BENCH setup --synthetic

# All 21 real-repo tasks (non-synthetic)
ALL_TASKS="express_app_init,express_res_send,express_json_send,express_render_chain,express_app_render,fastapi_dependency_resolution,fastapi_request_validation,fastapi_depends_internals,fastapi_depends_function,fastapi_depends_processing,gin_radix_tree,gin_client_ip,gin_middleware_chain,gin_context_next,gin_servehttp_flow,rg_trait_implementors,rg_flag_definition,rg_search_dispatch,rg_walker_parallel,rg_lineiter_definition,rg_lineiter_usage"

# 6 hard tasks used for Opus
HARD_TASKS="fastapi_dependency_resolution,fastapi_depends_processing,gin_middleware_chain,gin_servehttp_flow,rg_search_dispatch,rg_walker_parallel"

LOGDIR="$(mktemp -d)"
echo "Logs: $LOGDIR/{sonnet,opus,haiku}.log"
echo ""

echo "Starting 3 model runs in parallel..."

$BENCH run \
  --models sonnet \
  --tasks "$ALL_TASKS" \
  --modes baseline,glean \
  --reps 3 \
  > "$LOGDIR/sonnet.log" 2>&1 &
PID_SONNET=$!

$BENCH run \
  --models opus \
  --tasks "$HARD_TASKS" \
  --modes baseline,glean \
  --reps 3 \
  > "$LOGDIR/opus.log" 2>&1 &
PID_OPUS=$!

$BENCH run \
  --models haiku \
  --tasks "$ALL_TASKS" \
  --modes baseline,glean_forced \
  --reps 1 \
  > "$LOGDIR/haiku.log" 2>&1 &
PID_HAIKU=$!

echo "  sonnet (pid $PID_SONNET) — 126 runs"
echo "  opus   (pid $PID_OPUS) — 36 runs"
echo "  haiku  (pid $PID_HAIKU) — 42 runs"
echo ""
echo "Tail progress: tail -f $LOGDIR/*.log"
echo ""

FAILED=0

wait $PID_SONNET || { echo "FAIL: sonnet (see $LOGDIR/sonnet.log)"; FAILED=1; }
wait $PID_OPUS   || { echo "FAIL: opus (see $LOGDIR/opus.log)"; FAILED=1; }
wait $PID_HAIKU  || { echo "FAIL: haiku (see $LOGDIR/haiku.log)"; FAILED=1; }

echo ""
echo "Done. Results in benchmark/results/"
echo "Logs in $LOGDIR/"
echo ""
echo "To analyze: bench analyze results/<file>.jsonl"

exit $FAILED
