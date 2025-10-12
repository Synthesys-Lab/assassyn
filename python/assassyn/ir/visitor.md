# Visitor Module

This module implements the visitor pattern for traversing the assassyn frontend AST. The Visitor class serves as a base class for implementing different traversal strategies across the AST, enabling code generation and analysis operations.

---

## Section 1. Exposed Interfaces

### class Visitor

The base visitor pattern class for traversing the assassyn frontend AST. This class provides a framework for implementing different traversal strategies by defining visit methods for each AST node type.

#### Attributes

- `current_module: Module` - Tracks the module being visited during traversal. Set to the current module for regular modules, `None` for arrays and downstreams.

#### Methods

#### `__init__(self)`

```python
def __init__(self):
    '''Initialize the visitor with no current module'''
    self.current_module = None
```

Initializes the visitor with no current module set.

#### `visit_system(self, node: SysBuilder)`

```python
def visit_system(self, node: SysBuilder):
    '''Enter a system'''
    for elem in node.arrays:
        self.visit_array(elem)
    for elem in node.modules:
        self.current_module = elem
        self.visit_module(elem)
    self.current_module = None
    for elem in node.downstreams:
        self.visit_module(elem)
```

**Explanation:** Entry point for visiting a complete system. Traverses the system in a specific order: first arrays, then modules (with `current_module` set), and finally downstreams (with `current_module` cleared). This ordering ensures that modules are processed with proper context while downstreams are processed without module context.

#### `visit_module(self, node: Module)`

```python
def visit_module(self, node: Module):
    '''Enter a module'''
    self.visit_block(node.body)
```

**Explanation:** Visits a module by delegating to its body block. This method provides a hook for subclasses to perform module-specific processing before visiting the module's body.

#### `visit_block(self, node: Block)`

```python
def visit_block(self, node: Block):
    '''Enter a block'''
    for elem in node.iter():
        self.dispatch(elem)
```

**Explanation:** Visits a block by iterating through its elements and dispatching each element to the appropriate visitor method. This method handles the traversal of block contents and delegates to `dispatch()` for proper routing.

#### `dispatch(self, node)`

```python
def dispatch(self, node):
    '''Dispatch the node in a block to the corresponding visitor'''
    if isinstance(node, Expr):
        self.visit_expr(node)
    if isinstance(node, Block):
        self.visit_block(node)
```

**Explanation:** Dispatches a node to the appropriate visitor method based on its type. Routes `Expr` nodes to `visit_expr()` and `Block` nodes to `visit_block()`. This method enables polymorphic traversal of different node types within blocks.

#### `visit_array(self, node)`

```python
def visit_array(self, node):
    '''Enter an array'''
```

**Explanation:** Hook method for visiting array nodes. Empty implementation that subclasses can override to provide array-specific processing.

#### `visit_expr(self, node: Expr)`

```python
def visit_expr(self, node: Expr):
    '''Enter an expression'''
```

**Explanation:** Hook method for visiting expression nodes. Empty implementation that subclasses can override to provide expression-specific processing.

#### `visit_port(self, node: Port)`

```python
def visit_port(self, node: Port):
    '''Enter a port'''
```

**Explanation:** Hook method for visiting port nodes. Empty implementation that subclasses can override to provide port-specific processing.

---

## Section 2. Internal Helpers

This module contains no internal helper functions or data structures. All functionality is exposed through the Visitor class interface.
