# Array Write Port Module

This module enables multi-ported register array access by defining classes that support the syntactic sugar `(array & module)[index] <= value` for array writes.

-----

## Exposed Interfaces

```python
class WritePort: ...
class IndexedWritePort: ...
```

-----

## Syntactic Sugar for Multi-Port Writes

The primary purpose of this module is to provide an intuitive syntax for allowing multiple modules to write to the same array. This is achieved by overloading Python's operators to build an `ArrayWrite` IR node in stages.

The expression `(array & module)[index] <= value` is processed as follows:

1.  **`(array & module)`**: The bitwise AND operator `&` is used to create a `WritePort` instance. This object represents a dedicated connection, binding the target `array` to the specific `module` that is performing the write.
2.  **`[index]`**: The indexing operation `[]` is called on the `WritePort` object. This does not perform a write, but instead returns a temporary `IndexedWritePort` proxy object that stores both the parent `WritePort` and the `index`.
3.  **`<= value`**: The less-than-or-equal operator `<=` is called on the `IndexedWritePort` proxy. This final step triggers the creation of the `ArrayWrite` IR node, passing the array, index, value, and the original module context to its constructor to correctly represent the multi-ported write in the IR.

-----

## Class Descriptions

  * **`WritePort`**: An object that represents a dedicated write connection from a specific module to an array. It is the entry point for the syntactic sugar and contains the core logic for creating the `ArrayWrite` node.
  * **`IndexedWritePort`**: A temporary proxy object whose sole purpose is to capture the array `index` and handle the final `<=` assignment in the syntactic sugar chain.
