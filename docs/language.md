# Language Manual

## Abstract

This document severs as a language manual of Assasyn[^1], a language that unifies hardware design,
implementation, and verification. It is designed to be a relatively high-level language,
so that developers can focus on the behavior of the system, rather than timing, state machine,
and other rules.

## Introduction

Below, is a text representation of our language IR. In the rest of this document, each component
of this example will explained in both sides, the programming frontend, and the IR representation.
For specific examples, refer the `tests` directory, which is made to be self-examplatory.

````
// Listing 1: Pseudo-code of the programming paradigms.

system main {
  // Array declarations
  // All the arrays are considered global.
  array: a[int<32>; 1]

  // Implicitly driver is executed every cycle.
  module driver() {
    v = read a[0];
    func = bind = adder { a = v, b = v };
    async_call func();
    new_v = v + 1; // NOTE: This is increase variable "_1" by "one"
    a[0] = v
  }

  module foo(a: int<32>, b: int<32>) {
    c = a + b;
  }
}
````

### Hardware Design

Hardware design is unique to software programming in many ways. Here we characterize several major
differences:

1. Excessive concurrency: Transistors that builds different hardware modules can be concurrently
busy, which is also the source of high performance. However, this also makes the hardware design
hard to debug. Though we do have parallel programming in software, they are managed in a relatively
heavy-weighted way, like threads, processes, and tasks. Therefore, a clear way that manages the
concurrency in a light-weighted way is highly desirable.

2. Data write: In software programming, a variable write is visible to users immediately. However,
in hardware design, a variable can only be written once, and will be only visible in the next cycle.
In this language, the "write-once" rule will be double-enforced by both the compiler[^2] and the
generated simulator runtime.

3. Resource Constraint: In software programming, different functions can share the same computing
resources the ISA. However, hardware design is to allocate the resources themselves.
In this language, the time-multiplexed resource sharing, and the dedicated allocation should be
well abstracted.

## Language Components

In this section, each component of the language will be explained in detail, both the frontend
programming interfaces, and the IR representations.

### System

A whole system is comprised by several modules (see above). A system may have a `driver` module
(see more details in the next paragraph to explain the module),
which is invoked every cycle to drive the whole system. This "driver" serves like a `main`
function, which is both the entrance and drives the system execution.


To build a system, `SysBuilder` should be used:
```` Rust
use eir::builder::SysBuilder;
// ...
let mut sys = SysBuilder::new("system");
````

The `SysBuilder` not only serves as the system itself, but also works as an IR builder to grow
the hardware description.

### Module

Module is a basic build block block of the system, but it is also slightly different from
the modules we have in both software and hardware programming.

To make an analogy to existing concepts in software programming, a module is like both a function,
and a basic block. Basic block is a very common concept in compiler design, which indicates a
region of code starts with a label which can be the destination of a jump,
ends with a jump operation. This is used to support control flows (if-then-else, loops, etc.).
However, in a physical circuit, no operations can move backward --- you can only have a cyclic
graphs in combinational logics.

NOTE: For simplicity, we currently regard all the operations within a module is combinational,
which means everything is done within one cycle. Our future goal will automatically partition
the pipeline stages to improve the clock frequency.

To build a module, `module_builder` macro should be used:
````Rust
use eda4eda::module_builder;
// ...
module_builder!(adder(/*parameterizations*/)(/*inputs*/a: int<32>, b: int<32>) {
  // The body of the module
  c = a.add(b);
});
let adder_module = adder_builder(&mut sys);
````

The first parenthesis is for the parameterization of this module, and the 2nd pranthesis is for
the input ports of this module. This design makes the whole programming model feels like
we first call a function `goo = foo(/*params*/)` to return a function, whose signature is
`goo(/*inputs*/)`. TODO(@were): Write a document to discuss our design decisions on dynamic
and static bindings.

NOTE: `module_builder` is a procedural macro, which essentially does source-to-source
transformation to translate the module definition in the macro scope to IR builder API calls.
For more details, refer the developer document.

`module_builder` will declared a `foo_builder` function, and this function will be called to
construct a dedicated module of `foo` in the system.

### Values and Expressions

In each module, we have several operations to describe the behaviors, including arithmetics,
read/write to arrays, and asynchronous module invocations.

1. Values: A first-order value can either be a constant or a variable. Because Rust does not
support type-based overloading, but sometimes we do need to write code like both `a + b` and
`a + 1`. Therefore, my parser will implicitly hide this from users (Of course refer the developer
docs for more details). A variable can be from the declared port, parameterizations,
or from an assigned expression (refer next expression bullet for more details).

A constant is an immediate value, and by defaulty it is typed `i32`.
`.` can be used to specify its type, e.g. `1.uint<32>`.

2. Expressions: To keep the simplicity of our frontend parser, for now, several constraints
are imposed  on our expressions' expression: a) All the operands should be a first order value;
b) All the expressions should have have one operator, and all the operators should be described by
a method call; d) All the valued expression should be assigned to a variable.

To explain, if we want to write a mac unit, we should write like this:

```` Rust
module_builder!(mac()(a: int<32>, b: int<32>, c: int<32>) {
  // The body of the module
  mul = a.mul(b);
  add = mul.add(c);

  // TODO: Support these.
  // v = a.mul(b).add(c); // Only one operator is allowed.
  // v = c.add(a.mul(b)); // Only simple first-order operands supported.
});
````

3. Module Invocation: To invoke a module, we use the `async_call` keyword to indicate that this is
an inter-module invocation, and all the module invocations are non-valued.
Now two kinds of parameter feedings are supported: positional and named. The positional surrounded
by a pair of parentheses feeds the parameters in the order of the module definition,
while the named surrounded by a pair of curly braces feeds the parameters by the
name of the module definition.

```` Rust
module_builder!(driver()() {
  // named
  async adder { b : 1, a : 2 };
  // positional
  async adder(1, 2);
});
````

Stick on this example, we have several important concepts to explain bind and multi-caller. If
an example in `tests` is run, this will be noticed:

````
  _1 = bind adder { a : 1, b : 2 };
  async_call _1();
````

Essentially, all the module invocations will be done by pushing data to the callee's FIFO channels.
Then, in the next cycle, the callee will pop the data from the FIFO channels, and execute the
combination logic. For more details on multi-caller case, refer `4.`.

4. Bind/Partial Function: Bind is introduced to handle the condition that the values for an
`async_call` are not from a single same module. To explain, We first use Python's
`functools.partial` as an analogy:

```` Python
import functools
def add(a, b):
  return a + b
add5 = functools.partial(add, 5)
add5(3) # equivalent to add(5, 3)
````

Therefore, if we want to call a module with values from different modules, we can do below:

```` Rust
module_builder!(adder()(a: int<32>, b: int<32>) {
  // The body of the module
  c = a.add(b);
});

let adder = adder_builder(&mut sys);

module_builder!(add5(adder)() {
  // bind
  bound_add = bind adder(5);
}.expose(bound_add));

let (add5, bound_add) = add5_builder(&mut sys, adder);

module_builder!(driver(bound_add)() {
  // The body of the module
  v = read a[0];
  async_call bound_add(v);
  new_v = v + 1;
  a[0] = new_v;
});

let driver = driver_builder(&mut sys, bound_add);

````

See `tests/bind.rs` for a runnable example.

Moreover, as discussed above in 2 in Hardware Design and 3 in this section, multi-caller in a
same cycle will be an issue: A FIFO push is essentially a register write, and a multi-caller
implies a multi-write in a same cycle. To hide the implementation details from users, we will
instantiate different bundles of FIFOs for each caller. Since each caller actually calls the
`bind`, the FIFOs will be instantiated by the binds.


5. Scopes and Contional Execution: Unlike what we have in software programming, we do not have
an instruction pointer to move around. Instead, we have a set of combinational logics which can only
move forward. Therefore, here we only have three kinds of scopes: a) conditional execution;
b) self-spin execution; c) cycled execution.

```` Rust
// For conditional execution, we do NOT have a `else` branch.
module_builder!(foo()(a: int<32>, b: int<32>) {
  // The body of the module
  cond = a.igt(b);
  when cond {
    a = a.add(b);
    // do something
  }
});
````

```` Rust
module_builder!(foo()(a: int<32>, b: int<32>) {
  // A spin lock always accepts a array value, because an array value is side-effected, which
  // can give different results when invoked multiple times. Wait until will wait the value in
  // the given array to be true to execute the body.
  //
  // NOTE: A spin lock can only appear in the main body of a module.
  wait_until {
    a_valid = a.valid();
    b_valid = b.valid();
    valid = a_valid.bitwise_and(b_valid);
    valid
  } {
    // implicitly pops the FIFOs
    // a = a.pop();
    // b = b.pop();
    a = a.add(b);
  }
});
````

```` Rust
// NOTE: A cycled execution will only be available in the testbench module.
module_builder!(testbench[/*ports*/][/*parameterizations*/] {
  // This body will only be executed on cycle 1.
  cycle 1 {
    a = a.add(b);
  }
  cycle 2 {
    a = a.add(b);
  }
  // The compiler will implicitly tick cycle 3 and 4.
  cycle 5 {
    a = a.add(b);
  }
  // This is ILLEGAL, because the cycle number should be monotonically increasing.
  // Compiler will cast an error.
  cycle 4 {
    a = a.add(b);
  }
});
````

6. Array Operations: All the arrays are declared globally. Arrays are used
to describe any stateful execution like local state machine, register file,
and locks between modules (see 5.).

```` Rust
a = array(int<32>, 1);
a[0] = 1; // write the array, value will be seen next cycle.
          // each array can only be written once in a cycle.
          // TODO: The compiler will try to analyze this.
          // DONE: The simulator will give an error if the array is written more than once.
v = a[0]; // read the array, not necessarily 1, should be the value in last cycle.
````

[^1]: The name "Assasyn" stands for "**As**ynchronous **S**emantics for **A**rchitectural
**S**imulation and **Syn**thesis".

[^2]: TODO: The compiler will use the condition caluses to detect if two writes will
happen in the same cycle. If so, a error will be casted.
