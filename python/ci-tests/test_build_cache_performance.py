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
    
    workspace_dir = './workspace'
    test_subfolder = os.path.join(workspace_dir, test_name)

    cache_file = os.path.join(os.path.dirname(__file__), '.build_cache.json')
    if os.path.exists(cache_file):
        os.remove(cache_file)
        print(f"Cleaned up cache file: {cache_file}")
    
    if os.path.exists(test_subfolder):
        shutil.rmtree(test_subfolder, ignore_errors=True)
        print(f"Cleaned up test folder: {test_subfolder}")
    
    test_config = config(
        path=workspace_dir, 
        verbose=False,
        simulator=True,
        verilog=False,
        sim_threshold=100,
        idle_threshold=100,
        enable_cache=True,  
    )
    
    def build_and_run():
        sys = SysBuilder(test_name)
        with sys:
            SimplePrinter().build()
        

        start = time.time()
        

        simulator_path, _ = elaborate(sys, **test_config)
        
        binary_path = build_simulator(simulator_path)
        
        build_time = time.time() - start
        
        output = run_simulator(binary_path=binary_path)
        
        return build_time, output
    
    print("\nFirst Build")
    first_build_time, output1 = build_and_run()
    print(f"First build time: {first_build_time * 1000:.2f}ms")
    
    
    print("\nSecond Build")
    second_build_time, output2 = build_and_run()
    print(f"Second build time: {second_build_time * 1000:.2f}ms")
    
    speedup = first_build_time / second_build_time
    print(f"Speedup: {speedup:.2f}x")
    print(f"Time saved: {(first_build_time - second_build_time) * 1000:.2f}ms")
    
    assert speedup >= 2.0, (
        f"Cache did not provide expected speedup. "
        f"Expected at least 2x, got {speedup:.2f}x. "
        f"First: {first_build_time * 1000:.2f}ms, Second: {second_build_time * 1000:.2f}ms"
    )
    
    if os.path.exists(test_subfolder):
        shutil.rmtree(test_subfolder, ignore_errors=True)
    if os.path.exists(cache_file):
        os.remove(cache_file)


if __name__ == '__main__':
    test_build_cache_performance()
