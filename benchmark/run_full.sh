#!/usr/bin/env bash
set -euo pipefail

# Opus benchmark: all tasks × 2 modes × 3 reps, parallelized by repo.
#
# WARNING: This invokes claude -p many times. Run outside of Claude Code.

cd "$(dirname "$0")"

BENCH="./target/release/bench"
REPS=3
MODES="baseline,glean"

echo "Building benchmark runner..."
cargo build --release

# Set up repos if needed
"${BENCH}" setup --repos

# Tasks grouped by repo — each group runs in parallel.
REPOS=(gin rg zod af)

declare -A TASKS
TASKS[gin]="gin_middleware_chain,gin_servehttp_flow,gin_radix_tree,gin_client_ip,gin_context_next,gin_binding_tag"
TASKS[rg]="rg_search_dispatch,rg_walker_parallel,rg_binary_detection_default,rg_trait_implementors,rg_flag_definition,rg_lineiter_definition,rg_lineiter_usage"
TASKS[zod]="zod_error_fallback,zod_string_schema,zod_parse_flow,zod_error_handling,zod_discriminated_union,zod_transform_pipe,zod_optional_nullable"
TASKS[af]="af_session_config,af_request_chain,af_response_validation,af_interceptor_protocol,af_upload_multipart,af_acceptable_status"

# Count total tasks across all repos.
TOTAL_TASKS=0
for repo in "${REPOS[@]}"; do
    # shellcheck disable=SC2312 # tr won't fail on a literal string
    mapfile -t items < <(tr ',' '\n' <<< "${TASKS[${repo}]}")
    TOTAL_TASKS=$((TOTAL_TASKS + ${#items[@]}))
done
TOTAL_RUNS=$((TOTAL_TASKS * 2 * REPS))

echo "Running Opus on ${TOTAL_TASKS} tasks, ${REPS} reps (${TOTAL_RUNS} runs total)..."
echo "Parallelizing by repo: ${REPOS[*]}"
echo ""

# Each repo group writes to its own temp file to avoid interleaved writes.
TMPDIR_RESULTS=$(mktemp -d)
# shellcheck disable=SC2064 # expand TMPDIR_RESULTS now so cleanup targets the right dir
trap "rm -rf '${TMPDIR_RESULTS}'" EXIT

# Launch each repo group in parallel.
PIDS=()
for repo in "${REPOS[@]}"; do
    tag=$(printf '[%-3s]' "${repo}")
    tmpfile="${TMPDIR_RESULTS}/${repo}.jsonl"
    "${BENCH}" run --models opus --tasks "${TASKS[${repo}]}" --modes "${MODES}" --reps "${REPS}" --output "${tmpfile}" 2>&1 | sed "s/^/${tag} /" &
    PIDS+=($!)
done

echo "Waiting for all repo groups to finish..."
echo "${PIDS[*]}"
echo ""

FAILED=0
for pid in "${PIDS[@]}"; do
    wait "${pid}" || { echo "ERROR: repo group (PID ${pid}) failed"; FAILED=1; }
done

if [[ ${FAILED} -ne 0 ]]; then
    echo "Some repo groups failed. Check output above."
    exit 1
fi

# Merge per-repo temp files into a single timestamped result file.
TIMESTAMP="${EPOCHSECONDS}"
MERGED="results/benchmark_$(date -r "${TIMESTAMP}" +%Y%m%d_%H%M%S)_opus.jsonl"
mkdir -p results
cat "${TMPDIR_RESULTS}"/*.jsonl > "${MERGED}"

echo ""
echo "======================================================================"
echo "Benchmark complete! (${TOTAL_RUNS} runs)"
echo "Merged results: ${MERGED}"
echo "======================================================================"
echo ""
echo "To analyze: ${BENCH} analyze ${MERGED}"
