#!/usr/bin/env bash
set -euo pipefail

# Reproduce the benchmark results from README.md.
# Runs Sonnet, Opus, and Haiku in parallel.
#
# WARNING: This invokes claude -p ~200 times. Run outside of Claude Code.

cd "$(dirname "$0")"

BENCH="./target/release/bench"

echo "Building benchmark runner..."
cargo build --release

# Set up repos if needed
$BENCH setup --repos

# All 26 real-repo tasks (non-synthetic)
ALL_TASKS="gin_radix_tree,gin_client_ip,gin_middleware_chain,gin_context_next,gin_servehttp_flow,gin_binding_tag,rg_trait_implementors,rg_flag_definition,rg_search_dispatch,rg_walker_parallel,rg_lineiter_definition,rg_lineiter_usage,rg_binary_detection_default,af_session_config,af_request_chain,af_response_validation,af_interceptor_protocol,af_upload_multipart,af_acceptable_status,zod_string_schema,zod_parse_flow,zod_error_handling,zod_discriminated_union,zod_transform_pipe,zod_optional_nullable,zod_error_fallback"

# 6 hard tasks used for Opus
HARD_TASKS="gin_middleware_chain,gin_servehttp_flow,rg_search_dispatch,rg_walker_parallel,rg_binary_detection_default,zod_error_fallback"

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

echo "  sonnet (pid $PID_SONNET) — 156 runs"
echo "  opus   (pid $PID_OPUS) — 36 runs"
echo "  haiku  (pid $PID_HAIKU) — 52 runs"
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
