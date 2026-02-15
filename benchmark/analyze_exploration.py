#!/usr/bin/env python3
"""
Analyze exploration patterns from benchmark results.
Extract tool call sequences and exploration efficiency metrics.
"""

import json
import os
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple

def parse_jsonl(file_path: str) -> List[Dict]:
    """Parse JSONL file and return list of run results."""
    results = []
    with open(file_path, 'r') as f:
        for line in f:
            line = line.strip()
            if line:
                results.append(json.loads(line))
    return results

def calculate_ratios(run: Dict) -> Dict:
    """Calculate exploration efficiency metrics."""
    tool_calls = run.get('tool_calls', {})
    num_turns = run.get('num_turns', 0)
    num_tool_calls = run.get('num_tool_calls', 0)

    # Extract counts for different tool categories
    reads = 0
    searches = 0
    greps = 0
    maps = 0
    globs = 0

    for tool, count in tool_calls.items():
        if 'read' in tool.lower():
            reads += count
        if 'search' in tool.lower():
            searches += count
        if 'grep' in tool.lower():
            greps += count
        if 'map' in tool.lower():
            maps += count
        if 'glob' in tool.lower():
            globs += count

    # Calculate ratios
    ratios = {
        'reads_per_turn': reads / num_turns if num_turns > 0 else 0,
        'searches_per_turn': searches / num_turns if num_turns > 0 else 0,
        'greps_per_turn': greps / num_turns if num_turns > 0 else 0,
        'tools_per_turn': num_tool_calls / num_turns if num_turns > 0 else 0,
        'total_reads': reads,
        'total_searches': searches,
        'total_greps': greps,
        'total_maps': maps,
        'total_globs': globs,
    }

    # Infer likely first tool (if map=1 in glean mode, it's likely first)
    first_tool = None
    if maps == 1 and run.get('mode') == 'glean':
        first_tool = 'glean_map'
    elif globs >= 1 and run.get('mode') == 'baseline':
        first_tool = 'Glob (likely)'

    ratios['likely_first_tool'] = first_tool

    return ratios

def analyze_file(file_path: str) -> Tuple[str, List[Dict]]:
    """Analyze a single benchmark file."""
    runs = parse_jsonl(file_path)
    filename = os.path.basename(file_path)

    analyzed_runs = []
    for run in runs:
        if 'error' in run:
            continue  # Skip errors/timeouts

        analysis = {
            'task': run.get('task'),
            'repo': run.get('repo'),
            'mode': run.get('mode'),
            'model': run.get('model'),
            'num_turns': run.get('num_turns'),
            'num_tool_calls': run.get('num_tool_calls'),
            'tool_calls': run.get('tool_calls', {}),
            'correct': run.get('correct'),
            'glean_version': run.get('glean_version'),
        }

        analysis.update(calculate_ratios(run))
        analyzed_runs.append(analysis)

    return filename, analyzed_runs

def compare_modes(runs: List[Dict]):
    """Compare baseline vs glean exploration patterns."""
    baseline_runs = [r for r in runs if r['mode'] == 'baseline' and r['model'] == 'sonnet']
    glean_runs = [r for r in runs if r['mode'] == 'glean' and r['model'] == 'sonnet']

    def calc_avg(runs, key):
        values = [r[key] for r in runs if key in r]
        return sum(values) / len(values) if values else 0

    print("\n" + "="*80)
    print("BASELINE vs GLEAN COMPARISON (Sonnet only)")
    print("="*80)

    print(f"\nBaseline runs: {len(baseline_runs)}")
    print(f"Glean runs: {len(glean_runs)}")

    print("\n--- Average Exploration Metrics ---")
    print(f"{'Metric':<30} {'Baseline':>15} {'Glean':>15} {'Delta':>15}")
    print("-" * 80)

    metrics = [
        'tools_per_turn',
        'reads_per_turn',
        'searches_per_turn',
        'greps_per_turn',
        'num_turns',
        'num_tool_calls',
    ]

    for metric in metrics:
        baseline_avg = calc_avg(baseline_runs, metric)
        glean_avg = calc_avg(glean_runs, metric)
        delta = glean_avg - baseline_avg
        print(f"{metric:<30} {baseline_avg:>15.2f} {glean_avg:>15.2f} {delta:>15.2f}")

    # Tool preference analysis
    print("\n--- Tool Preference (Average per run) ---")
    print(f"{'Tool':<30} {'Baseline':>15} {'Glean':>15}")
    print("-" * 80)

    baseline_reads = calc_avg(baseline_runs, 'total_reads')
    baseline_greps = calc_avg(baseline_runs, 'total_greps')
    baseline_globs = calc_avg(baseline_runs, 'total_globs')

    glean_reads = calc_avg(glean_runs, 'total_reads')
    glean_searches = calc_avg(glean_runs, 'total_searches')
    glean_maps = calc_avg(glean_runs, 'total_maps')

    print(f"{'Read operations':<30} {baseline_reads:>15.2f} {glean_reads:>15.2f}")
    print(f"{'Search operations (Grep/Search)':<30} {baseline_greps:>15.2f} {glean_searches:>15.2f}")
    print(f"{'Discovery (Glob/Map)':<30} {baseline_globs:>15.2f} {glean_maps:>15.2f}")

    # Glean map usage
    print("\n--- Glean Map Usage ---")
    glean_with_map = sum(1 for r in glean_runs if r['total_maps'] > 0)
    print(f"Glean runs starting with map: {glean_with_map}/{len(glean_runs)} ({100*glean_with_map/len(glean_runs):.1f}%)")

    # Search to read ratio
    print("\n--- Exploration Patterns ---")
    baseline_search_read_ratio = baseline_greps / baseline_reads if baseline_reads > 0 else 0
    glean_search_read_ratio = glean_searches / glean_reads if glean_reads > 0 else 0

    print(f"Baseline Grep:Read ratio: {baseline_search_read_ratio:.2f}:1")
    print(f"Glean Search:Read ratio: {glean_search_read_ratio:.2f}:1")

    return baseline_runs, glean_runs

def print_detailed_runs(runs: List[Dict], title: str, limit: int = 5):
    """Print detailed information for individual runs."""
    print(f"\n{title}")
    print("="*80)

    for i, run in enumerate(runs[:limit], 1):
        print(f"\n{i}. {run['task']} ({run['repo']}) - {'✓' if run['correct'] else '✗'}")
        print(f"   Turns: {run['num_turns']}, Tool calls: {run['num_tool_calls']}")
        print(f"   Tools: {run['tool_calls']}")
        print(f"   Ratios: tools/turn={run['tools_per_turn']:.2f}, reads/turn={run['reads_per_turn']:.2f}, searches/turn={run['searches_per_turn']:.2f}")
        if run['likely_first_tool']:
            print(f"   Likely first tool: {run['likely_first_tool']}")

def main():
    """Main analysis function."""
    results_dir = Path("/Users/flysikring/conductor/workspaces/glean/almaty/benchmark/results")

    # Focus on the two newest files
    target_files = [
        "benchmark_20260213_131246.jsonl",  # Previous glean run
        "benchmark_20260213_135039.jsonl",  # New glean run (MCP-only)
    ]

    all_runs = []

    for filename in target_files:
        file_path = results_dir / filename
        if file_path.exists():
            print(f"\nProcessing: {filename}")
            _, runs = analyze_file(str(file_path))
            all_runs.extend(runs)
            print(f"  Found {len(runs)} valid runs")

    # Compare modes
    baseline_runs, glean_runs = compare_modes(all_runs)

    # Show detailed examples
    print_detailed_runs(baseline_runs, "\nDETAILED BASELINE EXAMPLES", limit=3)
    print_detailed_runs(glean_runs, "\nDETAILED GLEAN EXAMPLES", limit=3)

    # Task-specific comparison
    print("\n" + "="*80)
    print("TASK-SPECIFIC COMPARISON")
    print("="*80)

    tasks = set(r['task'] for r in all_runs)
    for task in sorted(tasks):
        task_baseline = [r for r in baseline_runs if r['task'] == task]
        task_glean = [r for r in glean_runs if r['task'] == task]

        if task_baseline and task_glean:
            print(f"\n{task}:")

            # Compare both files for this task
            for b_run, t_run in zip(task_baseline, task_glean):
                file_marker = "OLD" if "131246" in str(b_run.get('glean_version', '')) else "NEW"
                print(f"  Baseline: turns={b_run['num_turns']}, tools={b_run['num_tool_calls']}, {b_run['tool_calls']}")
                print(f"  Glean ({file_marker}): turns={t_run['num_turns']}, tools={t_run['num_tool_calls']}, {t_run['tool_calls']}")
                print(f"  Efficiency: Baseline={b_run['tools_per_turn']:.2f} tools/turn, Glean={t_run['tools_per_turn']:.2f} tools/turn")

if __name__ == "__main__":
    main()
