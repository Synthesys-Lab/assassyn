# Developer Doc

## Rust Toolchain

**Tooling:** This project is written purely in Rust, which means you can easily use this
project as a library in other projects, or modularly invoke a unit test. 

**A test case** not only checks the correctness of a newly written module, but also serves as an
example to see how certain interfaces should be used. Moreover, it also offers a light-weighted
way to write a toy example to play with this framework, and sanity-check the build success.

In Rust Cargo, you can use `cargo test` to invoke all test cases. If you want to invoke
a specific single test case, you can use `cargo test [test-name]`. When debugging output logs
are desired, `cargo test [test-name] -- --nocapture` can be used to enable the logs dumped to both
stdout and stderr. Refer `../tests` for the testcases we have.

TODO: Write an instruction on how the frontend of this language works, which also serves as a
simple tutorial to how Rust AST library and procedural macro works.

## Github

<!-- TODO: Write instructions for Github PR usage. -->

**Formatting:** To ensure consistent code style, copy `assassyn/utils/pre-commit` to `.git/hooks/pre-commit`. This setup automatically formats your code and checks coding style with each commit.

**Branching:** 
1. Fork the repository to your account.
2. Clone your fork: `git clone <url-to-your-fork>`.
3. Create a new branch for your changes: `git checkout -b <your-dev-branch>`.
4. After development, submit a pull request to the master branch from your branch.