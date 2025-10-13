# Trace-based DSL Frontend Embedded in Python

To save the excessive engineering effort of developing parser
for this DSL, we adopt a trace-based DSL. All the operations
done within the scope of tracing are overloaded.

For the conceptual overview of the DSL, see [dsl.md](./dsl.md).
For details on intrinsic functions, see [intrinsics.md](./intrinsics.md).

To explain, `a + b` does not calculate the result of an addition,
but creates an `Add` node to implicitly
append itself to the current insertion point (see below for more details).

## Naming

A key tradeoff between a parser-based frontend and a trace-based
frontend is the variable naming. Most of the operators can be overloaded
but left value assignment `a = b` cannot be overloaded as it changes
Python global environment. To have this solved, refer to [naming.md](../../internal/naming.md)
for more details.

## Insertion Point

Assassyn provides two decorators to annotate a function as an
entry point of tracing:

```python
class Adder(Module):

    def __init__(self):
        super().__init__({
            a = Input(UInt(32)),
            b = Input(UInt(32)),
        })

    @module.combinational
    def build(self):
        a, b = self.pop_all_ports(True)
        c = a + b
        log("{} + {} = {}", a, b, c)
```


We have a singleton to maintain a global insert point.
Before entering the function annotated with `{module/downstream}.combinational`,
the insertion point, an empty block of the current, `self`, will be pushed
into a stack. When leaving this function, the stack will be popped.
Maintaining a stack allows to build multiple modules recursively.

## Conditions

If we use `Condition(xxx)` in trace-based DSL, it will inject a conditional block
in AST when entering this `with` context, and set this block to the current insert point
by pushing it to a block stack.
When leaving this `with` context, this block insertion point is popped.

````python
with Condition(xxx):
    pass
````

Be careful, if you use `if` statement, it just changes the path of tracing:

```python
if xxx:
    # trace 1
    pass
else:
    # trace 2
    pass
```

This is similar to C/C++'s macro-based pre-processing:
```C
# if xxx
// this will be compiled
#else
// other will be compiled
#endif
```
