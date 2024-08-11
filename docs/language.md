# Language Manual

## Abstract

This document serves as a language manual of Assasyn[^1], a language that unifies hardware design,
implementation, and verification. It is designed to be a relatively high-level language,
so that developers can focus on the behavior of the system, rather than timing, state machine,
and other rules.

## Introduction

Below, is a text representation of our language IR. In the rest of this document, each component
of this example will be explained in both sides, the programming frontend, and the IR representation.
For specific examples, refer to the `tests` directory, which is made to be self-examplatory.

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

1. Excessive concurrency: Transistors that build different hardware modules can be concurrently
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

## Language & System Components

In this section, each component of the language will be explained in detail, both the frontend
programming interfaces, and the IR representations.

The frontend programming interfaces are embedded in Python involving decorators and operator
overloading, which makes the feeling of programming as close to software as possible. The backend
programming interfaces are implemented in Rust, a language with very steep learning curve, so
we decide not to expose this too much to users.

### System

A whole system is comprised by several modules (see above) and arrays.
A system may have a `driver` module (see more details in the next paragraph to explain the module),
which is invoked every cycle to drive the whole system. This "driver" serves like a `main`
function, which is both the program entrance and drives the system execution.


To build a system, `SysBuilder` should be used:
````Python
from assassyn import *
from assassyn.frontend import *

sys = SysBuilder("main")

with sys:
    # Build modules
````

The `SysBuilder` not only serves as the system itself, but also works as an IR builder to grow
the hardware description.

### Module

Module is a basic build block of the system, but it is also slightly different from
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

To describe a module, both `module.constructor` and `module.combinational` macro should be used.
The `module.constructor` decorate the constructor of the module, and the `module.combinational`
decorates the combinational logic of this module. See the example below.

In the constructor, all the ports should be declared. Using the `Port` class. The decorator will
implicitly maintain the additional port information, like the name (identifier) of the port.

In the combinational logic, all the operations are described. The operations involving Assassyn
objects are overloaded. These overloaded operations will transistively build the IR representation.
These IR nodes will be implicitly pushed into the module by the `module.combinational` decorator.

````Python
class Adder(Module):
    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))
        self.b = Port(Int(32))

    @module.combinational
    def build(self):
        c = self.a + self.b

class Driver(Module):
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, adder: Adder):
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        cnt[0] = cnt[0] + 1
        adder.async_called(a=v, b=v)
````

A Driver module is a special module, like a `main` function in software programming, which serves
as the entrance of the system. It is unconditionally invoked every cycle to drive the whole system.

Such a described module should be instantiated under the scope of a `SysBuilder`, which is like
the top function in RTL programming.

````Python
with sys:
    adder = Adder()
    driver = Driver()

    adder.build()
    driver.build(adder)
````

We adopt a design/implementation separated style in our language so that we do not suffer from
cyclic dependences.

### Values and Expressions

In each module, we have several operations to describe the behaviors, including arithmetics,
read/write to arrays, and asynchronous module invocations.

1. Types: Currently, we support `{Int/UInt/Bits}(bit)` and `Float` types.
2. Values start with ports, arrays, and constants, and can be built by operations among them.
    * Ports are the inputs of a module, which are typically scalars.
    * Arrays are first delared by `RegArray(type, size)`, where `size` should be a constant in the IR.
    * Constants can be declared by `type(value)`, e.g. `Int(32)(1)`.
3. Expressions
    * Arithmetic operations: `+`, `-`, `*`, `/`, `%`, `**`, `&`, `|`, `^`, `~`, `<<`, `>>`.
    * Comparison operations: `==`, `!=`, `>`, `>=`, `<`, `<=`.
    * Port FIFO methods: `Port.{pop/push/peek/valid}`.
    * Addressing: `array[index]` for both left and right value.
    * Slicing: `array[start:end]` for only right value. Unlike Python, the slicing is inclusive on both.
    * Concatenation: `a.concat(b)`, where `a` is the msb, and `b` is the lsb.
    * Module invocation: `module.async_called(**kwargs)`. Since the ports are declared in the constructor, which are unordered, therefore, we use the named arguments to feed the parameters.
    * Binds: `module.bind(**kwargs)`. This is like function binding in functional programming, where fixes several parameters fed to the module, and returns a handle to this module without invoking it.
4. Scopes and Conditional Execution:
    * We support if-statement (without else) in our combinational logic.
```` Python
with Conditional(cond):
    # do something
````
    * Besides, we also support cycle-speicific operation to write testbenches.
```` Python
with Cycle(1):
    # do something
with Cycle(2):
    # do something
````
5. Array Operations: This is a supplimentary description to expression addressing. All the array reads are immediate, while all the array writes are chronological --- the values are only visible next cycle. No two array writes within the same cycle are allowed. The generated simulator will enforce this.

[^1]: The name "Assasyn" stands for "**As**ynchronous **S**emantics for **A**rchitectural
**S**imulation and **Syn**thesis".
