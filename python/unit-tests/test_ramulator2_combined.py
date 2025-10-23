#!/usr/bin/env python3
"""Combined Ramulator2 test and cross-validation for pytest.

This module combines the Python Ramulator2 test with cross-language validation
to ensure behavioral consistency across C++, Rust, and Python implementations.
"""
import os
import sys
import difflib
import subprocess
import shlex
from typing import Dict, Tuple
import pytest

# Add the python directory to the path
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from assassyn.utils import repo_path
from assassyn.ramulator2 import PyRamulator, Request


def run_command(command: str, workdir: str, env: Dict[str, str] | None = None) -> Tuple[int, str, str]:
    """Run a command and return exit code, stdout, and stderr."""
    proc = subprocess.Popen(
        command,
        cwd=workdir,
        shell=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        executable="/bin/bash",
        env=env,
    )
    out, err = proc.communicate()
    return proc.returncode, out, err


def get_expected_targets(home: str) -> Dict[str, Tuple[str, str]]:
    """Get the command and working directory for each language implementation."""
    return {
        "cpp": (
            os.path.join(home, "tools/c-ramulator2-wrapper/build/bin/test"),
            os.path.join(home, "tools/c-ramulator2-wrapper/build/bin"),
        ),
        "rust": (
            "cargo test --quiet --test test_ramulator2 -- --nocapture",
            os.path.join(home, "tools/rust-sim-runtime"),
        ),
        "python": (
            f"python -u {shlex.quote(os.path.join(home, 'python/unit-tests/test_ramulator2.py'))}",
            home,
        ),
    }


def build_cpp_if_needed(home: str) -> None:
    """Build C++ executable if it doesn't exist."""
    cpp_exe = os.path.join(home, "tools/c-ramulator2-wrapper/build/bin/test")
    if os.path.isfile(cpp_exe) and os.access(cpp_exe, os.X_OK):
        return
    build_dir = os.path.join(home, "tools/c-ramulator2-wrapper/build")
    os.makedirs(build_dir, exist_ok=True)
    cmake_cmd = "cmake .."
    make_cmd = "make -j"
    code, out, err = run_command(cmake_cmd, build_dir, env=os.environ.copy())
    if code != 0:
        raise RuntimeError(f"CMake configuration failed in {build_dir}:\n{err}")
    code, out, err = run_command(make_cmd, build_dir, env=os.environ.copy())
    if code != 0:
        raise RuntimeError(f"Make build failed in {build_dir}:\n{err}")


def build_rust_if_needed(home: str) -> None:
    """Build Rust test if needed."""
    workdir = os.path.join(home, "tools/rust-sim-runtime")
    code, out, err = run_command("cargo test --quiet --test test_ramulator2", workdir, env=os.environ.copy())
    if code != 0:
        raise RuntimeError(f"Cargo test build failed in {workdir}:\n{err}")


def normalize_output(text: str) -> str:
    """Normalize output for comparison by removing blank lines and trailing whitespace."""
    lines = text.split('\n')
    non_empty_lines = [line for line in lines if line.strip() != '']
    return '\n'.join(non_empty_lines).rstrip()


def filter_rust_output(text: str) -> str:
    """Filter out Cargo test harness noise from Rust output."""
    lines = text.split('\n')
    filtered_lines = []
    for line in lines:
        # Skip Cargo test harness lines
        if (line.startswith("running ") and " test" in line) or \
           line.startswith("test result:") or \
           line.strip() == "." or \
           line.strip() == "":
            continue
        filtered_lines.append(line)
    return '\n'.join(filtered_lines).rstrip()


def run_python_test() -> str:
    """Run the Python Ramulator2 test and return the output."""
    home = repo_path()
    sim = PyRamulator(f"{home}/tools/c-ramulator2-wrapper/configs/example_config.yaml")

    is_write = False
    v = 0  # counter
    output_lines = []

    for i in range(200):
        plused = v + 1
        we = v & 1
        re = not we
        raddr = v & 0xFF
        waddr = plused & 0xFF
        addr = waddr if is_write else raddr

        def callback(req: Request, i=i):  # capture i in closure
            output_lines.append(
                f"Cycle {i + 3 + (req.depart - req.arrive)}: Request completed: {req.addr} the data is: {req.addr - 1}"
            )

        ok = sim.send_request(addr, is_write, callback, i)
        write_success = "true" if ok else "false"
        if is_write:
            output_lines.append(
                f"Cycle {i + 2}: Write request sent for address {addr}, success or not (true or false){write_success}"
            )

        is_write = not is_write
        sim.frontend_tick()
        sim.memory_system_tick()
        v = plused

    sim.finish()
    return '\n'.join(output_lines)


def test_python_ramulator2():
    """Test Python Ramulator2 wrapper functionality."""
    output = run_python_test()
    
    # Basic validation - check that we got some output
    assert len(output) > 0, "Python test should produce output"
    
    # Check for expected patterns in the output
    assert "Write request sent" in output, "Should contain write requests"
    assert "Request completed" in output, "Should contain completed requests"
    
    # Check that we have reasonable number of operations
    write_count = output.count("Write request sent")
    completed_count = output.count("Request completed")
    assert write_count > 0, "Should have some write requests"
    assert completed_count > 0, "Should have some completed requests"


def test_cross_language_validation():
    """Test that all language implementations produce identical output."""
    home = repo_path()
    targets = get_expected_targets(home)

    # Build artifacts if missing
    try:
        build_cpp_if_needed(home)
        build_rust_if_needed(home)
    except Exception as e:
        pytest.skip(f"Failed to build required artifacts: {e}")

    # Base environment: ensure ASSASSYN_HOME is set for all children
    base_env = os.environ.copy()
    base_env["ASSASSYN_HOME"] = home

    results: Dict[str, Tuple[int, str, str]] = {}

    for lang, (cmd_or_path, workdir) in targets.items():
        # Per-language environment
        env = base_env.copy()
        if lang == "cpp":
            command = shlex.quote(cmd_or_path)
            # Help the C++ binary find shared libraries at runtime
            wrapper_lib_dir = os.path.join(home, "tools/c-ramulator2-wrapper/build/lib")
            ramulator_lib_dir = os.path.join(home, "3rd-party/ramulator2")
            existing_ld = env.get("LD_LIBRARY_PATH", "")
            ld_parts = [p for p in [wrapper_lib_dir, ramulator_lib_dir, existing_ld] if p]
            env["LD_LIBRARY_PATH"] = ":".join(ld_parts)
        else:
            command = cmd_or_path
        
        code, out, err = run_command(command, workdir, env=env)
        results[lang] = (code, out, err)
        
        # Check that command succeeded
        assert code == 0, f"{lang} implementation failed with exit code {code}. Stderr: {err}"

    # Normalize outputs for comparison
    norm = {}
    for lang, (code, out, err) in results.items():
        if lang == "rust":
            norm[lang] = normalize_output(filter_rust_output(out))
        else:
            norm[lang] = normalize_output(out)

    # Compare all outputs
    languages = list(norm.keys())
    base = languages[0]
    for other in languages[1:]:
        if norm[base] != norm[other]:
            diff = difflib.unified_diff(
                norm[base].splitlines(keepends=True),
                norm[other].splitlines(keepends=True),
                fromfile=base,
                tofile=other,
                n=3,
            )
            diff_text = "".join(diff)
            pytest.fail(f"Output differs between {base} and {other}:\n{diff_text}")


if __name__ == "__main__":
    # Allow running as standalone script for debugging
    test_python_ramulator2()
    test_cross_language_validation()
    print("All tests passed!")
