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

    * NOTE: For simplicity, we currently regard all the operations within a module is combinational,
    which means everything is done within one cycle. It is our future goal to  *

To build a module, `module_builder` macro should be used:
````Rust
use eda4eda::module_builder;
// ...
module_builder!(foo[/*inputs*/][/*parameterizations*/] {
  // The body of the module
});
foo_builder(&mut sys);
````

    * NOTE: `module_builder` is a procedural macro, which essentially does source-to-source
    transformation to translate the module definition in the macro scope to IR builder API calls.
    For more details, refer the developer document.


1. Module: Each module is just like a function, which exposes several interfaces for external
callers.  As it is shown in Listing 1, module `foo` can be invoked by feeding `i0` and `i1`.

NOTE 0: Each module has no explicit outputs. Exposing data to external modules are done by
triggering other modules (see trigger below for more details).

NOTE 1: I finally decide to explicitly expose FIFOs to users, because it is clearer for both
partial invocation and back pressure (also see trigger below for more details).

2. Logics & Predication: Within each module, logics are operators among operands for computations,
including but not limited to arithmetic operations (e.g. +-*/),
bitwise operations (e.g. &|^~), and trinary selection (:?).

In the example above, a counter is added every cycle, and push it to module "foo" for
computation.

2.1. Unlike branches in imperative execution, instructions can be skipped by fetching from
different program counters. Since it is impssible to "remove" the circuits taped
out/resources allocated, they can only be gated.
Therefore, for each logic operator, there will optionally be one predicate associated, where
each logic operator will only be executed while its predication is true.

NOTE 0: Predications will be propagated implicitly. Consider the example below:

````
c = (a + b).when(x == 1) // c will only be executed when x is 1
d = c + 1 // c's predication will be propagated. If c is not executed, d is not either.
````

NOTE 1: There will be a rewriting pass to propagate the predications. Gather all the operations
into their predicated blocks. However, unlike imperative conditional blocks, there is NO
else-block. "Else" will be done by flipping the condition.

NOTE 2: Though each operator appears in a sequence, they are not necessarily executed sequentially.
Only partial order among them are gauranteed. Consider the example below:

````
_1 = a + b
_2 = a - b
// No dependences between _1 and _2, so by default they are scheduled to execute together.
_3 = _1 * _2
````

Question: Is it possible to have an abstraction to develop a time-multiplex adder to share between
_1 and _2?

Answer: Possible, but we need to use "external call"/"trigger" for that.

3. Trigger: trigger is something like an async function call or a pulse signal
to invoke a module in verilog.
As it is shown in Listing 1, module `driver` unconditionally invokes `foo` each cycle.

Trigger is a syntactical sugar, for calling the destination.

````
foo.i0.push(_1)
foo.i1.push(_1)
call foo
````

By exposing FIFOs to users, partial trigger can be done. Considering if we do not have
both arguments for foo, then we have one module just push without invoking it, and the other
module serves as a "master" to push and invoke.

Consider an example below:

````
module a() {
  // ...
  add.lhs.push(some value)
  // push without triggering
}

module b() {
  add.rhs.push(some value)
  trigger add
}

// ... module add
module add(lhs, rhs) {
  _1 = lhs.pop()
  _2 = rhs.pop()
  _1 + _2
}
````

4. Array Operations: Though arrays are declared globally, they can be localized by further
compiler analysis and transformations once recognizing an array's use pattern. Arrays are used
to describe any stateful execution like local state machine, register file, and locks between
modules (using array-read as a predication).

Array + Trigger: Spin Trigger

````
// master module
spin_trigger other [a, b], lock

// agent module
if !lock {
  trigger myself
}
if lock {
  a.pop()
  b.pop()
  trigger other
}

// other module
a.pop()
b.pop()
....
````
