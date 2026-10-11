# Minor RV32I CPU

`src/main.py` builds the in-order RV32I example. `src/decoder.py` and
`src/instructions.py` describe its decode bundle; `src/memory_access.py`
extracts signed/unsigned bytes and halfwords and forwards load results.
`test_regressions.py` creates self-contained programs with explicit register
expectations, including masked stores and redirects into prefetched instructions.

From the Assassyn root, source `setup.sh`, then run:

```sh
python examples/minor-cpu/src/main.py --sim-only --sim-threshold 600000 0to100
python examples/minor-cpu/test_regressions.py
pytest -q python/unit-tests/test_minor_cpu_runner.py
```

`--sim-only` disables both RTL elaboration and Verilator execution. The default
600,000-cycle budget is intentionally bounded because the simulator preloads
Driver events. Long runs can explicitly increase `--sim-threshold`; 100 million
cycles can require a large event queue even before execution starts.

## Correctness and limitations

- Fetch captures the instruction, address and epoch together at SRAM response
  time. Taken redirects squash stale epochs, even when a stale token's PC equals
  the target: the fetcher issues that target again in the new epoch. JAL/JALR
  bypass their return address, not the branch target.
- Subword stores drive bit masks; loads select byte lanes and sign/zero extend.
  Misaligned cross-word accesses are not implemented.
- Workload-specific `.sh` checkers are authoritative. Otherwise `find_pass.sh`
  requires an executed legacy ISA pass marker and architectural halt.
  A prefetched/decoded marker on a squashed path is not accepted. A halt or trap alone never means success.
  Packaged workloads without such a checker/marker need a correctness checker
  before the runner will accept them. A failed check retains `raw.log`.
- The legacy `rv32ui-p-lbu` fixture is missing its data image. The legacy `sb`
  fixture contains inconsistent load widths/expected values (for example, a byte
  store followed by a halfword comparison against an untruncated word). These
  remain disabled in the default list; the self-contained regression explicitly
  checks LB/LBU/LH/LHU/SB/SH instead. Do not relax the CPU to match invalid fixtures.
- `coremask` aliases the packaged name `coremark`. The runner accepts packaged
  RV32I workload names only; source compilation and CoreMark source packaging
  are not provided. Prepare executable/configuration assets and a correctness
  checker separately. No full CoreMark validation is claimed.
- `kernel_cycles.csv` records either first-to-last executed CSR cycle-read marker
  distance (`csr_b00`, also recognizing CSR 0xc00) or the full last logged cycle.
  Decode/stall messages are not counted as committed markers. This is a trace
  measurement, not an implementation of an architectural cycle CSR.
