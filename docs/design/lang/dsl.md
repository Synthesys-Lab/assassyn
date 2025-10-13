# Domain-specific Language Abstraction

I am reluctant to call Assassyn a DSL. Though it is a DSL for
hardware design and it provides specific abstractions to hardware designs
discussed in [arch.md](../../arch/arch.md), these designs are general enough
to cover a wide range of hardware designs.

In this document, we mainly discuss how a credit-based pipeline stage is
abstracted in Assassyn conceptually, since many other abstractions can
hardly make sense without understanding the frontend implementation discussed
in [trace.md](./trace.md). For the underlying architecture concepts, see
[arch.md](../../arch/arch.md).

For practical usage examples and step-by-step tutorials, see the
[tutorials](../../../tutorials/) folder, which serves as the language manual
with hands-on examples covering basic module creation, inter-module communication,
and advanced usage patterns.

## Credited Pipeline Stage

### Retrieve Data from Stage Registers

We provide a syntactical sugar to retrieve data from stage registers.
As these registers are FIFOs, we provide `pop_all_ports(True/False)`.

```python
a, b = stage.pop_all_ports(True)
```

`True` is related to the `wait_until` primitive discussed below.
When it is `True`, it implicitly waits until all the ports are valid.
When `False`, it unconditionally pops all the ports, and the user
shall guarantee the timing of data validity.

### Activate a Stage

```python
stage.async_called(**args)
```

A credit-based pipeline stage is treated as an asynchronous function.
All its inputs are passed as arguments, and calling this function
increases a credit to this function, and successfully returning
(see below for the definition of "success") from the function decreases a credit.
If there is no credit available, this pipeline stage will not be activated at all.

Just like `main` is an entry point to a software program, we have a special
stage named `Driver` who has infinite credits. This stage is unconditionally
activated in every cycle to drive the whole design.

We introduce a `wait_until` primitive to define the success of a function.
This primitive is useful for CPU decoders, which may need to wait for the
validity of operands before sending data to the executor. For more details
on intrinsics, see [intrinsics.md](./intrinsics.md).

````python
# within a stage
a = a.valid()
b = b.valid()
# wait until both a and b operands are valid
wait_until(a & b)
````

> Note that all the expressions before `wait_until` are executed no matter successful or not.
> But all the expressions after `wait_until` are only executed when the condition is met.

## Data Divergence and Convergence

A key difference between hardware and software is that when calling a function,
all the arguments are in the scope of the caller and fed to the callee.
However, in hardware, data may diverge and converge from different source to different destinations.

```python
bound = stage.bind(a=1)
```

The above binds the value `1` to the argument `a` of the stage `stage` and returns a handle to
the bound stage. This is very similar to the `functools.partial` in Python.

```python
def foo(a, b):
    return a + b
goo = functools.partial(foo, 5)
goo(3) # foo(5, 3) = 8
```
