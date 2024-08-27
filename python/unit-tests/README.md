| Python                    | Feature to Test                                    | Status |
|---------------------------|----------------------------------------------------|--------|
| test_arbiter.py           | Disable arbiter rewriting using module attr.       | RV     |
| test_array_multi_read.py  | Expose the same reg to multiple module readers.    | RV     |
| test_array_multi_write.py | Multiple exclusive writes in a same moudle.        | RV     |
| teat_array_partition0.py  | Partitioned array write with constant indices.     | RV     |
| teat_array_partition1.py  | Partitioned array write with variable indices.     | RV     |
| test_async_call.py        | Basic async calling convention b/t modules.        | RV     |
| test_bind.py              | Partial function bind.                             | RV     |
| test_cse.py               | Common code elimination.                           | RV     |
| test_concat.py            | Concatentate operator.                             | RV     |
| test_downstream.py        | Combinational logic across multiple normal modules.| RV     |
| test_driver.py            | Test a standalone driver module.                   | RV     |
| test_dt_conv.py           | Data cast operator.                                | RV     |
| test_eager_bind.rs        | Conditional calling bind.                          | RV     |
| test_explicit_pop.py      | Explicit pop attribute and operation.              | RV     |
| test_fib.py               | Register writing roll over.                        | RV     |
| test_fifo_valid.py        | FIFO.valid operator overloading in frontend.       | RV     |
| test_helloworld.py        | Hello world! A simplest test case for logger.      | RV     |
| test_imbalance.py         | Imbalanced data arrival from 2 difference sources. | RV     |
| test_inline{0/1}.py       | Inlined hierarchical synthesis.                    | RV     |
| test_memory.py            | Memory module read and file initialization.        | RV     |
| test_multi_call.rs        | Multiple caller arbiter with backend rewriting.    | RV     |
| test_reg_init.py          | Register initialization.                           | RV     |
| test_select.rs            | Select trinary operator                            | RV     |
| test_testbench.py         | Cycled block, useful in testbench.                 | RV     |
| test_wait_until.py        | Wait-until execution.                              | RV     |

- R: Rust simulator is tested.
- V: Verilog is correctly simulated by Verilator.
- S: Verilog is correctly simulated by VCS (offline).

TODO: Simulate all the test cases in Verilator.
TODO: Port systolic array test to examples.

