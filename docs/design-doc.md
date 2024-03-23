### Programming Primitives

````
// Listing 1: Pseudo-code of the programming paradigms.

system {
  // array declarations
  // all the arrays are considered global.
  array: i32 a[1]

  module driver() {
    _1 = read a[0]
    self.trigger foo [ i0=_1, i1=_1 ]
    _2 = _1 + 1 // NOTE: This is increase variable "_1" by "one"
    write a[0], _2
    self.trigger driver [ ]
  }

  module foo(i0: i32, i1: i32) {
    _1 = i0.pop()
    _2 = i1.pop()
    _3 = _1 + _2;
  }
}

````

NOTE: For simplificity, we currently assume each module can be executed in one cycle.
Later, the compiler will try to partition the modules to balance the timing among these
operations.

0. System: A whole system is comprised by several modules (see above).
A system always has a "driver" module, which is initially invoked,
and then invoke itself each cycle after. This "driver" serves like a "main"
function, which is both the entrance and drives the system execution.

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
