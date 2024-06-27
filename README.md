# Assassyn: **As**ynchronous **S**emantics for **A**rchitectural **S**imulation & **Syn**thesis


Assassyn is aimed at developing a new programming paradigm for hardware development.
The ultimate goal is to unify the hardware modeling (simulation), implementation (RTL),
and verfication.

## Getting Started

**Users**: The language backend is implemented in Rust, while for easy-to-use sake, the frontend
is implemented in Python. You can initialize this package by running the command below.

````sh
# Install the required dpendences for Rust and Python
# Make sure you have: zsh, Rust toolchain, and pip
$ ./init.sh
$ source setup.sh # Add assassyn's python package to your PYTHONPATH
````

Because of the nature of Rust toolchain, rust backend will be built along with your designs.

**Developers**: All the test cases are located in `python/tests`, you can just run them like
all other Python scripts.

````sh
$ python python/tests/test_driver.py 
````

Refer our [developer doc](./docs/developers.md) for more details.

## Why yet another RTL generator?

Designing circuits using RTL exposes excessive low-level details to users, from the behavioral
semantics, to timing, and placement. Our insight of simplifying this programming paradigm is
to designate a set of simple and synthesizable primitives while retaining enough
expressiveness to program a massively concurrent programming system.

Refer [language manual](./docs/language.md) for more details.

