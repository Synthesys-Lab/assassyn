# Guideline to Git Commit Message Tags

## Generic
- [enhancement]: Refactors or other non-functional changes that improve the codebase.
- [bugfix]: Bug fixes.
- [docs]: Documentation changes.
- [agent]: Changes related to AI-powered development tools, and rules.

## Frontend-related
- [dsl]: Changes the public interfaces exposed to users.
- [sugar]: Changes that did not change the core public interfaces, but improve the usability with more friendly wrappers or syntactic sugars.

## Backend-related
- [codegen.sim]: Changes related code gen Rust simulator.
- [runtime.sim]: Changes related to the runtime for the Rust simulator.
- [codegen.rtl]: Changes related to RTL code gen using PyCDE.
- [uarch.rtl]: Changes related to the runtime for RTL, particularly the FIFO, and credit counter.

## Test-related
- [test.unit]: Unit tests.
- [test.ci]: Continuous integration tests.

When two or more tags apply, concatenate them, e.g., `[test.unit][bugfix]`.
