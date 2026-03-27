#!/usr/bin/env python3
"""Performance benchmark for radix sort implementations.

This script runs different radix sort implementations and collects performance
metrics including total cycles, stage breakdown, and cycles per element.
"""
import os
import sys
import subprocess
import re
import time
from pathlib import Path


def run_implementation(impl_file):
    """Run a radix sort implementation and capture output.

    Args:
        impl_file: Path to the implementation file (e.g., "main.py", "main_fsm.py")

    Returns:
        tuple: (success, output, elapsed_time)
    """
    examples_dir = Path(__file__).parent
    repo_root = examples_dir.parent.parent

    # Set up environment
    env = os.environ.copy()
    setup_script = repo_root / "setup.sh"

    print(f"\n{'='*70}")
    print(f"Running implementation: {impl_file}")
    print(f"{'='*70}")

    # Run the implementation
    start_time = time.time()
    try:
        # Source setup.sh and run the implementation
        # Redirect stderr to stdout to capture all output
        cmd = f"source {setup_script} && cd {examples_dir} && python3 {impl_file} 2>&1"
        result = subprocess.run(
            cmd,
            shell=True,
            executable="/bin/bash",
            capture_output=True,
            text=True,
            timeout=120
        )
        elapsed_time = time.time() - start_time

        # Combine stdout and stderr
        full_output = result.stdout + result.stderr

        if result.returncode != 0:
            print(f"✗ Implementation failed with return code {result.returncode}")
            print(f"Output length: {len(full_output)} chars")
            return False, full_output, elapsed_time

        print(f"✓ Implementation completed successfully")
        print(f"Output length: {len(full_output)} chars")
        return True, full_output, elapsed_time

    except subprocess.TimeoutExpired:
        elapsed_time = time.time() - start_time
        print(f"✗ Implementation timed out after 120 seconds")
        return False, "", elapsed_time
    except Exception as e:
        elapsed_time = time.time() - start_time
        print(f"✗ Failed to run implementation: {e}")
        return False, str(e), elapsed_time


def parse_output(output):
    """Parse simulator output to extract performance metrics.

    Args:
        output: Raw output from simulator

    Returns:
        dict: Performance metrics
    """
    metrics = {
        'total_cycles': 0,
        'read_count': 0,
        'prefix_count': 0,
        'write_count': 0,
        'reset_count': 0,
        'passes': 0,
        'elements': 2048,  # Known from numbers.data
    }

    # Count stage occurrences in output
    metrics['read_count'] = output.count('Stage 1: Read')
    metrics['prefix_count'] = output.count('Stage 2:')
    metrics['write_count'] = output.count('Stage 3-2: Writing')
    metrics['reset_count'] = output.count('Stage 3-3: Reset complete')
    metrics['passes'] = output.count('Radix Sort: Bits')

    # Check if finished
    if 'finish' not in output.lower():
        metrics['completed'] = False
    else:
        metrics['completed'] = True

    # Extract actual cycle count from simulator output
    if metrics['completed']:
        # Find all cycle numbers in format "Cycle @XXXXX.00"
        cycle_matches = re.findall(r'Cycle @(\d+(?:\.\d+)?)', output)
        if cycle_matches:
            # Get the last (maximum) cycle number
            cycles = [float(c) for c in cycle_matches]
            metrics['total_cycles'] = int(max(cycles))
            metrics['cycles_per_element'] = metrics['total_cycles'] / metrics['elements']
        else:
            # Fallback to estimation if cycle info not found
            reads_per_pass = metrics['elements']
            prefix_per_pass = 16
            writes_per_pass = metrics['elements'] * 2  # Read-write pattern
            reset_per_pass = 17

            cycles_per_pass = 1 + reads_per_pass + prefix_per_pass + writes_per_pass + reset_per_pass
            metrics['total_cycles'] = cycles_per_pass * metrics['passes']
            metrics['cycles_per_element'] = metrics['total_cycles'] / metrics['elements']

    return metrics


def print_metrics(impl_name, metrics, elapsed_time):
    """Print performance metrics in a formatted table.

    Args:
        impl_name: Name of the implementation
        metrics: Performance metrics dict
        elapsed_time: Wall-clock time in seconds
    """
    print(f"\n{'='*70}")
    print(f"Performance Metrics: {impl_name}")
    print(f"{'='*70}")

    if not metrics['completed']:
        print("⚠ INCOMPLETE: Implementation did not finish")
        return

    print(f"Elements:              {metrics['elements']}")
    print(f"Passes:                {metrics['passes']}")
    print(f"Total Cycles (est):    {metrics['total_cycles']:,}")
    print(f"Cycles per Element:    {metrics['cycles_per_element']:.2f}")
    print(f"Wall-clock Time:       {elapsed_time:.2f}s")
    print()
    print("Stage Breakdown (estimated):")

    # Calculate stage percentages
    if metrics['total_cycles'] > 0:
        reset_cycles = metrics['passes']
        read_cycles = metrics['read_count']
        prefix_cycles = metrics['prefix_count'] * 16  # Assuming 16 cycles per prefix
        write_cycles = metrics['write_count'] * 2  # 2 cycles per write (approx)
        reset_radix_cycles = metrics['reset_count'] * 17

        print(f"  - Reset Phase:       {reset_cycles:>8,} cycles ({reset_cycles/metrics['total_cycles']*100:>5.1f}%)")
        print(f"  - Read Phase:        {read_cycles:>8,} cycles ({read_cycles/metrics['total_cycles']*100:>5.1f}%)")
        print(f"  - Prefix Phase:      {prefix_cycles:>8,} cycles ({prefix_cycles/metrics['total_cycles']*100:>5.1f}%)")
        print(f"  - Write Phase:       {write_cycles:>8,} cycles ({write_cycles/metrics['total_cycles']*100:>5.1f}%)")
        print(f"  - Reset Radix:       {reset_radix_cycles:>8,} cycles ({reset_radix_cycles/metrics['total_cycles']*100:>5.1f}%)")

    print(f"{'='*70}\n")


def compare_implementations(results):
    """Compare multiple implementations and print comparison table.

    Args:
        results: List of (impl_name, metrics, elapsed_time) tuples
    """
    if len(results) < 2:
        return

    print(f"\n{'='*70}")
    print("Performance Comparison")
    print(f"{'='*70}")
    print(f"{'Implementation':<20} {'Cycles':>12} {'Speedup':>10} {'Time (s)':>10}")
    print(f"{'-'*70}")

    # Find baseline (first successful implementation)
    baseline_cycles = None
    for impl_name, metrics, _ in results:
        if metrics['completed'] and metrics['total_cycles'] > 0:
            baseline_cycles = metrics['total_cycles']
            break

    if baseline_cycles is None:
        print("No successful implementations to compare")
        return

    for impl_name, metrics, elapsed_time in results:
        if not metrics['completed']:
            print(f"{impl_name:<20} {'INCOMPLETE':>12} {'-':>10} {elapsed_time:>10.2f}")
        else:
            speedup = baseline_cycles / metrics['total_cycles']
            print(f"{impl_name:<20} {metrics['total_cycles']:>12,} {speedup:>10.2f}x {elapsed_time:>10.2f}")

    print(f"{'='*70}\n")


def main():
    """Main benchmark execution."""
    # List of implementations to benchmark
    implementations = [
        "main.py",
        "main_fsm.py",
    ]

    results = []

    for impl_file in implementations:
        impl_path = Path(__file__).parent / impl_file
        if not impl_path.exists():
            print(f"⚠ Skipping {impl_file}: File not found")
            continue

        success, output, elapsed_time = run_implementation(impl_file)

        if success:
            metrics = parse_output(output)
            print_metrics(impl_file, metrics, elapsed_time)
            results.append((impl_file, metrics, elapsed_time))
        else:
            print(f"✗ {impl_file} failed\n")
            # Still add to results for comparison
            results.append((impl_file, {'completed': False, 'total_cycles': 0}, elapsed_time))

    # Print comparison if multiple implementations ran
    if len(results) > 0:
        compare_implementations(results)

    # Save baseline if main.py succeeded
    for impl_name, metrics, elapsed_time in results:
        if impl_name == "main.py" and metrics['completed']:
            baseline_file = Path(__file__).parent / "baseline_main.txt"
            with open(baseline_file, 'w') as f:
                f.write(f"Baseline Performance - main.py\n")
                f.write(f"{'='*70}\n")
                f.write(f"Date: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
                f.write(f"Elements: {metrics['elements']}\n")
                f.write(f"Passes: {metrics['passes']}\n")
                f.write(f"Total Cycles: {metrics['total_cycles']:,}\n")
                f.write(f"Cycles per Element: {metrics['cycles_per_element']:.2f}\n")
                f.write(f"Wall-clock Time: {elapsed_time:.2f}s\n")
            print(f"✓ Baseline saved to {baseline_file}")
            break


if __name__ == "__main__":
    main()
