# Developer Doc

## Rust Lang

**Tooling:** This project is written purely in Rust, which means you can easily use this
project as a library in other projects, or modularly invoke a function to test the correctness. 

**A test case** not only checks the correctness of a newly written module, but also serves as an
example to see how certain interfaces should be used. Moreover, it also offers a light-weighted
way to write a toy example to play with this framework, and sanity-check the build success.

In Rust Cargo, you can use `cargo test --lib` to invoke all test cases. If you want to invoke
a specific single test case, you can use `cargo test --lib <filename>::<funcname>`. If the
`funcname` is unique, it is possible to omit the `<filename>`. When debugging output logs
are desired, `cargo test --lib --nocapture` can be used to enable the logs dumped to both
stdout and stderr. Refer `src/tests` for the testcases we have.

## Github

TODO: Write instructions for Github PR usage.

