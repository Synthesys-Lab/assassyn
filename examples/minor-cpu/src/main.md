# Minor CPU pipeline and runner

## Exposed interfaces

`build_cpu(depth_log, *, sim_only=False, sim_threshold=600000)` returns the
system, compiled simulator and optional Verilog path. The positive CLI budget
bounds Driver scheduling; simulator-only runs suppress RTL elaboration.

`Execution`, `Decoder` and `FetcherImpl` carry aligned raw instructions, PCs
and eight-bit epochs. Execution records the redirect's originating epoch.
Fetch acknowledges it by incrementing its epoch; old tokens cannot retire even
when their PC matches the redirect target. Register, CSR and memory side effects
are gated by liveness. Link bypass uses PC+4. `Onwrite` clears retiring register
bits before setting newly issued writers.

`run_cpu(..., run_verilator_check=True)` runs available backends and calls
`check()`. `check()` invokes a workload script or the legacy ISA pass-marker
script with `check=True`; any nonzero exit is propagated. It does not reinterpret
halt as success. Failed runs retain the raw log.

`parse_args(argv)` accepts packaged workload names, `--sim-only` (alias
`--skip-verilator`) and a positive `--sim-threshold`. `run_workloads` runs the
requested cases and returns rows for `write_cycle_summary`.

## Internal helpers

`extract_kernel_cycles` recognizes only executed raw CSR read instructions,
excluding decode/stall messages; `measure_cycles` otherwise uses the final trace
cycle. `cp_required` fails on missing executable/configuration inputs. Optional
CoreMark packaging helpers validate the external script before invoking it.

See [the example README](../README.md) for fixture limitations and tests, and
[the architecture](../../../docs/design/arch/arch.md) for async call semantics.
