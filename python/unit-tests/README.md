## Testcase Overview

| Python                                                       | Feature to Test                        |
| ------------------------------------------------------------ | -------------------------------------- |
| `test_driver, test_helloworld, test_fib`                     | Module(esp driver), log                |
| `test_async_call, test_multi_call`                           | async_call, Port                       |
| `test_array_partiton0, test_array_partion1`                  | reg vs wire variables                  |
| `test_cse`                                                   | timing sequence                        |
| `test_concat`                                                | gramme : `concat`                      |
| `test_dt_conv`                                               | gramme  : `type convert`               |
| `test_finish`                                                | `finish()` function                    |
| `test_inline0, test_inline1`                                 | Function Extraction                    |
| `test_record, test_record_bundle_value,`<br>`test_record_large_bits` | gramme : `record`                      |
| `test_reg_init`                                              | gramme : `RegArray initial`            |
| `test_select, test_select1hot`                               | gramme : `select`                      |
| `test_testbench`                                             | Usage of `with Cycle(1):`              |
| `test_explict_pop, test_peek`                                | gramme in `Port`                       |
|                                                              |                                        |
| `test_fifo1, test_bind, `<br>`test_eager_bind, test_imbalance, `<br>`test_fifo_valid, test_wait_until` | sth about **Pure Sequential Logic**    |
| `test_comb_expose, test_toposort`<br />`test_downstream, `   | sth about **Pure Combinational Logic** |


## Testcase detail

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