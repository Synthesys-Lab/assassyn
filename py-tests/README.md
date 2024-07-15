| Python                           | Feature to Test                                    | Status |
|----------------------------------|----------------------------------------------------|--------|
| test_arbiter.py                  | Disable arbiter rewriting using module attr.       | R      |
| test_array_multi_{read/write}.py | Multiple readers/writers for the same reg.         | R      |
| test_async_call.py               | Basic async calling convention b/t modules.        | R      |
| test_bind.py                     | Partial function bind.                             | R      |
| test_cond_cse.py                 | Common code elimination.                           | R      |
| test_concat.py                   | Concatentate operator.                             | R      |
| test_dt_conv.py                  | Data cast operator.                                | R      |
| test_eager_bind.rs               | Conditional calling bind.                          | R      |
| test_imbalance.py                | Imbalanced data arrival from 2 difference sources. | R      |
| test_explicit_pop.py             | Explicit pop attribute and operation.              | R      |
| test_fib.py                      | Register writing roll over.                        | R      |
| test_fifo_valid.py               | FIFO.valid operator overloading in frontend.       | R      |
| test_helloworld.py               | Hello world! A simplest test case for logger.      | R      |
| test_inline{0/1}.py              | Inlined hierarchical synthesis.                    | R      |
| test_memory.py                   | Memory module read and file initialization.        | R      |
| test_multi_call.rs               | Multiple caller arbiter with backend rewriting.    | R      |
| test_select.rs                   | Select trinary operator                            | R      |
| test_wait_until.py               | Wait-until execution.                              | R      |
| test_testbench.py                | Cycled block, useful in testbench.                 | R      |


- R: Rust simulator is tested.
- V: Verilog is correctly simulated by Verilator.
- S: Verilog is correctly simulated by VCS (offline).

TODO: Simulate all the test cases in Verilator.
TODO: Port systolic array test to examples.

