# Trace-based DSL Frontend Embedded in Python

To save the excessive engineering effort of developing parser
for this DSL, we adopt a trace based DSL. All the Assassyn
[AST/IR nodes](../../python/assassyn/ir/) are overloaded.
Everytime an operation is done, it is implicitly put into
the IR current insert point maintained by the
[builder singleton](../../python/assassyn/builder/__init__.py).

When building a module, either a [pipeline stage](../../python/assassyn/ir/module/module.py)
or a [downstream](../../python/assassyn/ir/module/downstream.py),
a `combinational` decorator is annotated to the entrance of tracing
so that the singleton changes the current insert point to this module.
Essentially, builder maintains a stack of inserting point, so that
we can recursively build other modules when building a current module.

# Conditions

If we use `Condition(xxx)` in trace-based DSL, it will inject a conditional block
in AST. This `with` scope changes the current insert point to block when entering
and pops the stack to the block before when exiting.

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