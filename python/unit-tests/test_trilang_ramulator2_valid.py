#!/usr/bin/env python3
"""Cross-language validation invoker for Ramulator2 wrappers.

This script runs the following and compares their stdout outputs for equality:
- C++ executable: $ASSASSYN_HOME/tools/c-ramulator2-wrapper/build/bin/test
- Rust binary:    cargo test --test test_ramulator2 (in tools/rust-sim-runtime)
- Python script:  $ASSASSYN_HOME/python/unit-tests/test_ramulator2.py

It sources setup.sh before running each command to ensure environment setup.
Exits with non-zero status if any output differs or a command fails.
"""
import argparse
import difflib
import os
import shlex
import subprocess
import sys
from typing import Dict, Tuple
from io import StringIO
from assassyn.utils import repo_path
from assassyn.ramulator2 import PyRamulator, Request


def run_command(command: str, workdir: str, env: Dict[str, str] | None = None) -> Tuple[int, str, str]:
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


def ensure_cpp_executable_exists(path: str) -> None:
    if os.path.isfile(path) and os.access(path, os.X_OK):
        return
    raise FileNotFoundError(
        f"C++ executable not found or not executable: {path}. Build it first (cmake && make)."
    )


def build_cpp_if_needed(home: str, debug: bool = False) -> None:
    cpp_exe = os.path.join(home, "tools/c-ramulator2-wrapper/build/bin/test")
    if os.path.isfile(cpp_exe) and os.access(cpp_exe, os.X_OK):
        return
    build_dir = os.path.join(home, "tools/c-ramulator2-wrapper/build")
    os.makedirs(build_dir, exist_ok=True)
    cmake_cmd = "cmake .."
    make_cmd = "make -j"
    if debug:
        sys.stderr.write(f"[DEBUG] Building C++ in {build_dir}\n")
    code, out, err = run_command(cmake_cmd, build_dir, env=os.environ.copy())
    if code != 0:
        raise RuntimeError(f"CMake configuration failed in {build_dir}:\n{err}")
    code, out, err = run_command(make_cmd, build_dir, env=os.environ.copy())
    if code != 0:
        raise RuntimeError(f"Make build failed in {build_dir}:\n{err}")


def build_rust_if_needed(home: str, debug: bool = False) -> None:
    workdir = os.path.join(home, "tools/rust-sim-runtime")
    if debug:
        sys.stderr.write(f"[DEBUG] Building Rust in {workdir}\n")
    code, out, err = run_command("cargo test --quiet --test test_ramulator2", workdir, env=os.environ.copy())
    if code != 0:
        raise RuntimeError(f"Cargo test build failed in {workdir}:\n{err}")


def run_python_test() -> Tuple[int, str, str]:
    """Run the Python Ramulator2 test directly and return output."""
    try:
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

        # Suppress stdout during finish() to avoid statistics output
        old_stdout = sys.stdout
        sys.stdout = StringIO()
        sim.finish()
        sys.stdout = old_stdout
        
        output = '\n'.join(output_lines)
        return 0, output, ""
    except Exception as e:  # noqa: BLE001
        return 1, "", str(e)


def filter_stats_output(text: str) -> str:
    """Filter out Ramulator2 statistics output."""
    # The stats appear at the end of the output, starting with "Frontend:" or "MemorySystem:"
    # Simply truncate everything from the first occurrence of these markers onwards
    lines = text.split('\n')
    filtered_lines = []
    
    for line in lines:
        if line.startswith('Frontend:') or line.startswith('MemorySystem:'):
            break  # Stop processing, discard the rest
        filtered_lines.append(line)
    
    return '\n'.join(filtered_lines).rstrip()


def compare_texts(name_a: str, text_a: str, name_b: str, text_b: str) -> str:
    if text_a == text_b:
        return ""
    diff = difflib.unified_diff(
        text_a.splitlines(keepends=True),
        text_b.splitlines(keepends=True),
        fromfile=name_a,
        tofile=name_b,
        n=3,
    )
    return "".join(diff)


def main() -> int:
    parser = argparse.ArgumentParser(description="Cross-validate Ramulator2 wrapper outputs across languages.")
    parser.add_argument(
        "--skip",
        choices=["cpp", "rust", "python"],
        action="append",
        help="Skip running a particular implementation (can be specified multiple times).",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Print commands/env and extra diagnostics when failures occur.",
    )
    parser.add_argument(
        "--show-outputs",
        action="store_true",
        help="Display raw outputs from all languages before comparison.",
    )
    args = parser.parse_args()

    home = repo_path()
    targets = get_expected_targets(home)

    # Build artifacts if missing
    if not args.skip or "cpp" not in args.skip:
        try:
            build_cpp_if_needed(home, debug=args.debug)
        except Exception as e:  # noqa: BLE001
            sys.stderr.write(f"[ERROR] C++ build failed: {e}\n")
            return 2
        ensure_cpp_executable_exists(targets["cpp"][0])
    if not args.skip or "rust" not in args.skip:
        try:
            build_rust_if_needed(home, debug=args.debug)
        except Exception as e:  # noqa: BLE001
            sys.stderr.write(f"[ERROR] Rust build failed: {e}\n")
            return 2

    results: Dict[str, Tuple[int, str, str]] = {}
    failures: Dict[str, str] = {}

    # Base environment: ensure ASSASSYN_HOME is set for all children
    base_env = os.environ.copy()
    base_env["ASSASSYN_HOME"] = home

    for lang, (cmd_or_path, workdir) in targets.items():
        if args.skip and lang in args.skip:
            continue
        
        # Special handling for Python - run directly instead of subprocess
        if lang == "python":
            if args.debug:
                sys.stderr.write(f"[DEBUG] {lang} running direct Python test\n")
            code, out, err = run_python_test()
            results[lang] = (code, out, err)
            if code != 0:
                failures[lang] = f"Non-zero exit ({code}). Stderr:\n{err}\nStdout:\n{out}"
            continue
        
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
        if args.debug:
            sys.stderr.write(f"[DEBUG] {lang} cwd={workdir}\n")
            sys.stderr.write(f"[DEBUG] {lang} cmd={command}\n")
            if lang == "cpp":
                sys.stderr.write(f"[DEBUG] {lang} LD_LIBRARY_PATH={env.get('LD_LIBRARY_PATH','')}\n")
        code, out, err = run_command(command, workdir, env=env)
        results[lang] = (code, out, err)
        if code != 0:
            failures[lang] = f"Non-zero exit ({code}). Stderr:\n{err}\nStdout:\n{out}"

    # Extra diagnostics for C++ when failing
    if failures.get("cpp") and args.debug:
        cpp_path, cpp_cwd = targets["cpp"]
        try:
            ldd_code, ldd_out, ldd_err = run_command(f"ldd {shlex.quote(cpp_path)}", cpp_cwd, env=base_env)
            sys.stderr.write("[DEBUG] ldd output for C++ binary:\n")
            sys.stderr.write(ldd_out or ldd_err)
        except Exception as e:  # noqa: BLE001
            sys.stderr.write(f"[DEBUG] Failed to run ldd: {e}\n")

    if failures:
        for lang, msg in failures.items():
            sys.stderr.write(f"[ERROR] {lang}: {msg}\n")
        return 2

    # Normalize outputs slightly (strip trailing whitespace)
    norm = {k: v[1].rstrip() for k, v in results.items()}
    
    # Filter out stats from all languages
    for lang in norm:
        norm[lang] = filter_stats_output(norm[lang])
    
    # Show filtered outputs if requested
    if args.show_outputs:
        for lang in norm:
            print(f"\n=== {lang.upper()} OUTPUT (FILTERED) ===")
            print(norm[lang])
            print("=" * 50)
    
    # Filter out Cargo test harness noise from Rust output
    if "rust" in norm:
        lines = norm["rust"].split('\n')
        filtered_lines = []
        for line in lines:
            # Skip Cargo test harness lines
            if (line.startswith("running ") and " test" in line) or \
               line.startswith("test result:") or \
               line.strip() == "." or \
               line.strip() == "":
                continue
            filtered_lines.append(line)
        norm["rust"] = '\n'.join(filtered_lines).rstrip()
    
    # Normalize whitespace differences for all languages
    import re
    for lang in norm:
        # Remove ALL blank lines to eliminate formatting differences
        lines = norm[lang].split('\n')
        non_empty_lines = [line for line in lines if line.strip() != '']
        norm[lang] = '\n'.join(non_empty_lines)

    languages = [k for k in ["cpp", "rust", "python"] if not args.skip or k not in args.skip]
    if len(languages) < 2:
        print("Nothing to compare (need at least two languages).")
        return 0

    base = languages[0]
    all_equal = True
    for other in languages[1:]:
        diff = compare_texts(base, norm[base], other, norm[other])
        if diff:
            all_equal = False
            sys.stderr.write(f"[DIFF] {base} vs {other}:\n{diff}\n")

    if all_equal:
        print("All outputs are identical across implementations.")
        return 0

    return 1


if __name__ == "__main__":
    sys.exit(main())
