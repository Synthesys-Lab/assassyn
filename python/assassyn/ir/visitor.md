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

**Explanation:** Entry point for visiting a complete system. Traverses the system in a specific order: first arrays, then modules (with `current_module` set), and finally downstreams (with `current_module` cleared). Resetting `current_module` between phases prevents downstream traversals (which often represent SRAM payloads or pure combinational blocks) from accidentally inheriting the previous module context.

#### `visit_module(self, node: Module)`

```python
def visit_module(self, node: Module):
    '''Enter a module'''
    for elem in node.body or ():
        self.dispatch(elem)
```

**Explanation:** Visits a module by iterating through its flat body list. This hook allows subclasses to run module-specific logic before the traversal when needed.

#### `dispatch(self, node)`

```python
def dispatch(self, node):
    '''Dispatch the node in a block to the corresponding visitor'''
    if isinstance(node, Expr):
        self.visit_expr(node)
```

**Explanation:** Dispatches a node to the appropriate visitor method based on its type. Currently only `Expr` nodes are handled, as module bodies are flat lists of expressions guarded by predicate intrinsics.

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
