"""Test to verify build caching performance."""

import time
import os
import shutil

from assassyn.frontend import *
from assassyn.backend import elaborate, config
from assassyn.utils import build_simulator, run_simulator


class SimplePrinter(Module):
    """A minimal module for testing build cache."""

    def __init__(self):
        super().__init__(
            ports={},
        )

    @module.combinational
    def build(self):
        """Simple print logic."""
        log("Cache test message")


def test_build_cache_performance():
    test_name = "cache_perf_test"
    
    # Use isolated workspace to avoid interfering with parallel tests
    workspace_dir = './workspace_cache_test'

    cache_file = os.path.join(os.path.dirname(__file__), '.build_cache.json')
    if os.path.exists(cache_file):
        os.remove(cache_file)
        print(f"Cleaned up cache file: {cache_file}")
    
    # Clean up only our isolated workspace (ignore errors if files are locked)
    if os.path.exists(workspace_dir):
        shutil.rmtree(workspace_dir, ignore_errors=True)
        print(f"Cleaned up workspace: {workspace_dir}")
    
    test_config = config(
        path=workspace_dir,  # Use isolated workspace
        verbose=False,
        simulator=True,
        verilog=False,
        sim_threshold=100,
        idle_threshold=100,
        enable_cache=True,  # Explicitly enable cache for this test
    )
    
    def build_and_run():
        """Helper to build and run the system."""
        sys = SysBuilder(test_name)
        with sys:
            SimplePrinter().build()
        
        # Time only the build part (elaborate + compile), not the execution
        start = time.time()
        
        # Elaborate the system
        simulator_path, _ = elaborate(sys, **test_config)
        
        # Build the simulator (this is where caching happens)
        binary_path = build_simulator(simulator_path)
        
        build_time = time.time() - start
        
        # Run to verify it works (not timed - execution speed is same either way)
        output = run_simulator(binary_path=binary_path)
        
        return build_time, output
    
    # First build (cold - includes compilation)
    print("\nFirst Build")
    first_build_time, output1 = build_and_run()
    print(f"First build time: {first_build_time * 1000:.2f}ms")
    
    
    print("\nSecond Build")
    second_build_time, output2 = build_and_run()
    print(f"Second build time: {second_build_time * 1000:.2f}ms")
    
    speedup = first_build_time / second_build_time
    print(f"Speedup: {speedup:.2f}x")
    print(f"Time saved: {(first_build_time - second_build_time) * 1000:.2f}ms")
    
    # Verify cache provides at least 2x speedup
    assert speedup >= 2.0, (
        f"Cache did not provide expected speedup. "
        f"Expected at least 2x, got {speedup:.2f}x. "
        f"First: {first_build_time * 1000:.2f}ms, Second: {second_build_time * 1000:.2f}ms"
    )
    
    # Cleanup: remove isolated workspace and cache file (ignore errors)
    if os.path.exists(workspace_dir):
        shutil.rmtree(workspace_dir, ignore_errors=True)
    if os.path.exists(cache_file):
        os.remove(cache_file)


if __name__ == '__main__':
    test_build_cache_performance()
