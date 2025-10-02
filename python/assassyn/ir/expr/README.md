# Expression

This module implements all the IR expressions in Assassyn, as well as their
trace-based IR builder interfaces. For each IR builder function, once the IR node is built,
if it is desired to push the node into AST, a `@ir_builder` decorator shall be applied.