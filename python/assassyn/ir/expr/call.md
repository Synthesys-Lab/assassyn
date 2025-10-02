# Async Call Related IR Nodes

This module defines the `Bind` and `AsyncCall` IR nodes, which represent
the binding of arguments to a "function", which is a `Module` and the
asynchronous invocation of that node.

## Exposed Interface

````python
class Bind(Expr):
    # The module being bound
    m: Module
    # Essentially, a function binding in hardware is a FIFO push to
    # the module's input ports.
    pushes: list[FIFOPush]
````