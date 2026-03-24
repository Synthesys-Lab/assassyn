#!/usr/bin/env python3

import shutil
import subprocess
import sys
from pathlib import Path


def require_tool(name: str) -> str:
    path = shutil.which(name)
    if path is None:
        raise SystemExit(f"missing required tool: {name}")
    return path


def main() -> int:
    repo = Path(__file__).resolve().parents[3]
    workload_dir = repo / "examples" / "minor-cpu" / "workloads"
    src = workload_dir / "msort.cpp"
    binary = workload_dir / "msort"
    elf = workload_dir / "msort.elf"
    dump = workload_dir / "msort.dump"
    loader = repo / "examples" / "minor-cpu" / "utils" / "loader.py"

    gxx = require_tool("riscv64-unknown-elf-g++")
    objdump = require_tool("riscv64-unknown-elf-objdump")

    compile_cmd = [
        gxx,
        "-O3",
        "-std=c++17",
        "-march=rv32i",
        "-mabi=ilp32",
        "-ffreestanding",
        "-nostdlib",
        "-fno-exceptions",
        "-fno-rtti",
        "-fno-threadsafe-statics",
        "-fno-unwind-tables",
        "-fno-asynchronous-unwind-tables",
        "-Wl,--gc-sections",
        "-Wl,-e,_start",
        str(src),
        "-o",
        str(binary),
    ]
    subprocess.check_call(compile_cmd)
    shutil.copy2(binary, elf)

    with dump.open("w", encoding="utf-8") as handle:
        subprocess.check_call([objdump, "-D", str(binary)], stdout=handle)

    subprocess.check_call([sys.executable, str(loader), "--fname", str(dump), "--odir", str(workload_dir)])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
