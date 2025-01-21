| Python                    | Feature to Test                                    | Status |
|---------------------------|----------------------------------------------------|--------|
| test_arbiter.py           | Disable arbiter rewriting using module attr.       | RV     |
| test_array_multi_read.py  | Expose the same reg to multiple module readers.    | RV     |
| test_array_multi_write.py | Multiple exclusive writes in a same moudle.        | RV     |
| test_array_partition0.py  | Partitioned array write with constant indices.     | RV     |
| test_array_partition1.py  | Partitioned array write with variable indices.     | RV     |
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



1. `test_driver, test_helloworld, test_fib`
   + Understand what a `Module` is, and be aware of the composition and basic architecture of the project code.
   + Learn to use `log` to view output values.
   + Essentially, a `driver` is a clock-driven mechanism(in Verilog as clk), where its `cnt` value represents the clock count.
2. `test_async_call, test_multi_call`
   + Event invocation, which in Verilog is equivalent to a simple demo of sequential logic passing information for processing.
   + Key point: Observe the timing in the log output, focusing on the timing difference between event requests and event execution.
   + Syntax:
     1. Ports can only pass basic types, but implicitly, the passing is of register types.
     2. `async_call` is sequential in nature.
3. `test_array_partiton0, test_array_partion1`
   + The relationship between reg type and wire type variables in terms of read and write timing.
   + The focus is on identifying patterns in the log file.
4. `test_cse`
   + Simply observe the timing sequence.
5. `test_concat`
   + Demonstrates the `concat` operation, which corresponds to bit concatenation in Verilog.
6. `test_dt_conv`
   + Explains the methods for converting between basic types.
   + Carefully observe the differences between Cycle1 and Cycle2 to reinforce the understanding of register type read and write timing.
7. `test_finish`
   + Usage of the `finish()` function.
8. `test_inline0, test_inline1`
   + Provides an example of encapsulating a portion of logic within a function and then calling it.
9. `test_record, test_record_bundle_value, test_record_large_bits`
   + Explains the details related to the Record basic type.
     1. Record as a port type, passing register values.
     2. Accessing members of a Record, which is equivalent to normal operations and used for computation.
     3. Packaging of Records.
10. `test_reg_init`
    + Initialization of register type variables.
11. `test_select`
    + Syntax similar to the ternary operator.
12. `test_select1hot`
    + Selecting a value through a one-hot code.
    + Note: This will ultimately describe a hardware circuit, actually implemented using a multiplexer.
13. `test_testbench`
    + Usage of `with Cycle(1):`
14. `test_explict_pop`
    + An alternative method for reading port data.
15. `test_peek`
    + Similar to the operation of viewing the top of a queue in a `Queue`. It corresponds to the `front()` operation in the STL of C++ queues. Essentially, it is looking at the top element of the queue without removing it.