# Array Operation IR Nodes

This file defines the Intermediate Representation node classes for array read and write operations.

-----

## Exposed Interfaces

```python
class ArrayWrite(Expr): ...
class ArrayRead(Expr): ...
```

-----

## ArrayWrite Class

The `ArrayWrite` class is the IR node for an array write operation, representing `arr[idx] = val`.

  * It stores the target array, index, and the value to be written as its operands. It also records which module is performing the write.
  * **String Representation**: The node has a human-readable text format that includes the name of the writing module.
      * **Example**: `_arr[_idx] <= _val /* module_name */`

-----

## ArrayRead Class

The `ArrayRead` class is the IR node for an array read operation, representing the value of `arr[idx]`.

  * The data type of the read value is the same as the array's element type.
  * **Syntactic Sugar for Writes**: This class overloads the `<=` operator to provide a more intuitive syntax for array writes. An expression like `my_array[idx] <= value` is internally translated into an `ArrayWrite` operation.
  * **String Representation**: The node has a human-readable text format.
      * **Example**: `_res = _arr[_idx]`
