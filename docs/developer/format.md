# Format 

After modifying the code, you need to ensure that the format of your contributed code meets the requirements.

## Rust format

For formatting Rust code, use a standardized tool to make the modifications directly:

```
cargo fmt
```

This will automatically format the rust code.

## Python format

To format Python code, use Pylint to format it:

```
pylint --rcfile ./python/.pylintrc python/assassyn
```

This will give you the hint about format modification but you have to change the code by yourself.
