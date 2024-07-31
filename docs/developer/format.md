# Format

Refer to utils/pre-commit, it is highly recommended to add this pre-commit to your .git hook.

After modifying the code, you need to ensure that the format of your contributed code meets the requirements.

## Rust format

For formatting Rust code, use a standardized tool to make the modifications directly:

```
cargo fmt
```

This will automatically format the rust code according to the `rustfmt.toml` in this repo.

You can also use cargo clippy to get potential suggestions from static analysis.

```
cargo clippy -- -Dclippy::all
```

And you may also use cargo clippy fix to fix these suggestions.
```
cargo clippy -- fix 
```

However, this does not fix everything. Too aggressive rewritings are still required to be done by the developers.

## Python format

To format Python code, use Pylint to format it:

```
pylint --rcfile ./python/.pylintrc python/assassyn
```

This will give you the hint about format modification but you have to change the code by yourself.
