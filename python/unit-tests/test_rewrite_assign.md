# Test Cases for Rewrite Assignments

# Case 1: Simple Assignment

```python
def simplest():
    x = 10
```

# Case 2: Multiple Assignments

```python
def multiple():
    a = 5
    b = 15
    c = a + b
```

# Case 3: Tupled Assignment

```python
def loop_assign():
    a, b = 1, 2
```

# Case 4: Array Assignment

```python
def array_assign():
    arr = [0, 1, 2]
    arr[0] = 10
```

# Case 5: Attribute Assignment

```python
def attr_assign():
    @dataclass
    class Point:
        x: int
        y: int
    x.x = 10
```

# Case 6: Assignment Under Other Nodes

```python
def assign_under_other_nodes():
    for i in range(5):
        x = i * 2
```