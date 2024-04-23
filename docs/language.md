# Language Manual

## Abstract

This document severs as a language manual of (EDA)^2. A language for hardware design,
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
    async adder { a = v, b = v };
    new_v = v + 1; // NOTE: This is increase variable "_1" by "one"
    a[0] = v
  }

  module foo(a: int<32>, b: int<32>) {
    a = a.pop();
    b = b.pop();
    c = a + b;
  }
}

````

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
and a basic block.
In case you do not know what is basic block, it is a region of code starts with a label which
can be the destination of a jump, ends with a jump operation, which means within a basic block,
you can only move the operations forward --- just like what you have on circuits, you can only
move on in combinational logics.

NOTE: For simplicity, we currently regard all the operations within a module is combinational,
which means everything is done within one cycle. It is our future goal to automatically partition
the pipeline stages.

To build a module, `module_builder` macro should be used:
````Rust
use eda4eda::module_builder;
// ...
module_builder!(foo[/*inputs*/][/*parameterizations*/] {
  // The body of the module
});
foo_builder(&mut sys);
````

The first braket is for the input ports of this module, and the second bracket is for the
parameterizations of this module. The parameters will be the function argument of the `*_builder`
function. Refer `tests` for more examples.

NOTE: `module_builder` is a procedural macro, which essentially does source-to-source
transformation to translate the module definition in the macro scope to IR builder API calls.
For more details, refer the developer document.


### Values and Expressions

In each module, we have several operations to describe the behaviors, including arithmetics,
read/write to arrays, and asynchronous module invocations.

1. Values: A first-order value can either be a constant or a variable. Because Rust does not
support type-based overloading, but sometimes we do need to write code like both `a + b` and
`a + 1`. Therefore, my parser will implicitly hide this from users (Of course refer the developer
docs for more details). A variable can either be from the declared port or parameterizations,
or from an assigned expression (refer next expression bullet for more details).

A constant is an immediate value. You can use `.` to specify its type, e.g. `1.int<32>`.

2. Expressions: To keep the simplicity of our frontend, for now, we have several constraints
on our expressions' expression: a) All the operands should be a first order value; b) All the
expressions should have have one operator, and all the operators should be described by a method
call; d) All the valued expression should be assigned to a variable.

To explain, `a + b` is a simple expression, while `a + b * c` is not for
it two operators (* & +). Also, instead of using `a + b`, we should use `a.add(b)`.
See the example below:

```` Rust
// TODO(@were): Fully deprecate the explicit FIFO pop later.
module_builder!(foo[a:int<32>, b:int<32>][/*parameterizations*/] {
  // The body of the module
  a = a.pop();
  b = b.pop();
  c = a.add(b);
});
````

3. Module Invocation: To invoke a module, we use the `async` keyword to indicate that this is an
external module invocation, and all the module invocations are non-valued.
Now two kinds of parameter feedings are supported: positional and named. The positional surrounded
by a pair of parentheses feeds the parameters in the order of the module definition,
while the named surrounded by a pair of curly braces feeds the parameters by the
name of the module definition.

```` Rust
module_builder!(foo[/*ports*/][adder] {
  // named
  async adder { b : 1, a : 2 };
  // positional
  async adder(1, 2);
});
````

4. Bind/Partial Function: Sometimes values are not from a single module. To support this program
behavior we support bind/partial function. We first use Python's `functools.partial` as an analogy.

```` Python
import functools
def add(a, b):
  return a + b
add5 = functools.partial(add, 5)
add5(3) # equivalent to add(5, 3)
````

To use this language feature in our language, see below:

```` Rust
module_builder!(add[a:int<32>, b:int<32>][] {
  // The body of the module
  a = a.pop();
  b = b.pop();
  a + b
});
module_builder!(add5[][] {
  // bind
  add5 = bind add(5);
});
module_builder!(driver[/*ports*/][add5] {
  // The body of the module
  v = read a[0];
  async add5 { a = v, b = v };
  new_v = v + 1;
  a[0] = new_v;
});
````

See `tests/bind.rs` a runnable example.

5. Scopes and Contional Execution: Unlike what we have in software programming, we do not have
an instruction pointer to move around. Instead, we have a set of combinational logics which can only
move forward. Therefore, here we only have three kinds of scopes: a) conditional execution;
b) self-spin execution; c) cycled execution.

```` Rust
// For conditional execution, we do NOT have a `else` branch.
module_builder!(foo[/*ports*/][/*parameterizations*/] {
  // The body of the module
  cond = a > b;
  when cond {
    a = a.add(b);
  }
  // If you really need an `else` branch, you can use `cond.flip()`
  ncond = cond.flip();
  when ncond {
    c = a.add(b);
  }
});
````

```` Rust
module_builder!(foo[/*ports*/][/*parameterizations*/] {
  // A spin lock always accepts a array value, because an array value is side-effected, which
  // can give different results when invoked multiple times. Wait until will wait the value in
  // the given array to be true to execute the body.
  //
  // NOTE: A spin lock can only appear in the main body of a module.
  wait_until lock[0] {
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
