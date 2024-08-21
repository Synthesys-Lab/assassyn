# Format

After modifying the code, you need to ensure that the format of your contributed code meets the
formatting requirements. Refer to `scripts/pre-commit`, it is highly recommended to add this
pre-commit to your `.git/hooks/pre-commit`.

However, `cargo clippy` is a little bit strict on coding styles, if you want to commit some
intermediate developments, you can use `git commit -m 'your messages' --no-verify` to temporarily
skip the pre-commit check.

## Rust Format

For formatting Rust code, use a standardized tool to make the modifications directly.
This will automatically format the rust code according to the `rustfmt.toml` in this repo.

```
cargo fmt
```


You can also use cargo clippy to get potential suggestions from static analysis.

```
cargo clippy -- -Dclippy::all
```

`cargo clippy` can also fix some of the issues automatically. To do so, first commit all your codes
to version control using `--no-verify`, then run:

```
cargo clippy -- fix 
```

However, this does not fix everything. Too aggressive rewritings are still required to be done
by the developers.

## Python Format

To check the Python code, use Pylint to format it:

```
pylint --rcfile ./python/.pylintrc python/assassyn
```

This gives you the hint about format suggestions but you have to change the code by yourself.
