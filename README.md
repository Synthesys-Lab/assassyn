# Assassyn: **As**ynchronous **S**emantics for **A**rchitectural **S**imulation & **Syn**thesis


Assassyn is aimed at developing a new programming paradigm for describing hardware.
The ultimate goal is to unify the hardware model (simulation), register-transfer-level (RTL)
language, and verfication.

## Getting Started

**Users**: The frontend of this language is embedded in RUST, so just simply add this project to
your `Cargo.toml`, and use it!
TODO: Add a project uses this project as a submodule to demonstrate the usage.

**Developers**: All the test cases are located in `src/tests`, you can use the command below to
run all the test cases.

TODO: Make test cases self-examplified and document some usages in the test case. Users can quickly
try some proof of concepts in test cases.

````sh
cargo test [case-name]
````

Refer our [developer doc](./docs/developers.md) for more details.

## Why yet another RTL generator?

Designing circuits using RTL exposes excessive low-level details to users, from the
semantics, to timing, and placement. Our insight of simplifying this programming paradigm is
to designate a set of simple and synthesizable primitives and repropose a programming paradigm
that is close to programming a highly concurrent system, e.g. a multi-threading or distributed
system, so that many programming experiences could be borrowed.

Refer [language manual](./docs/language.md) for more details.

## A Demonstrative Example

TODO: Make a demonstrative example here.
