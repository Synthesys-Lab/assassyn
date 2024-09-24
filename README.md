# Assassyn: **As**ynchronous **S**emantics for **A**rchitectural **S**imulation & **Syn**thesis


Assassyn is aimed at developing a new programming paradigm for hardware development.
The ultimate goal is to unify the hardware modeling (simulation), implementation (RTL),
and verfication.

## Getting Started

**Users**: The language backend is implemented in Rust, while for easy-to-use sake, the frontend
is implemented in Python. You can initialize this package by running the command below.

````sh
# Install the required dpendences for Rust and Python >= 3.6
# Make sure you have: zsh, Rust toolchain, and pip
$ ./init.sh
$ source setup.sh # Add assassyn's python package to your PYTHONPATH
````

All the test cases are located in `python/unit-tests`. These cases are not only testing features,
but also self-examplified to demonstrate the usage of each feature.
Refer `python/unit-tests/README.md` for more details on the feature-case correspondence.

**Advanced Users**: If you would like to simulate the generated RTL, you can use
[verilator](https://github.com/verilator/verilator). To simply setup verilator, you can run
the following command.

````sh
$ ./scripts/build-verilator.sh
$ source setup.sh # Re-run the script since it has a different environment variable for verilator
````

**Developers**: All the test cases are located in `python/tests`, you can just run them like
all other Python scripts.

````sh
$ python python/unit-tests/test_driver.py 
````

Refer our [developer doc](./docs/developer/README.md) for more details on tooling.

## Why yet another RTL generator?

Designing circuits using RTL exposes excessive low-level details to users, from the behavioral
semantics, to timing, and placement. Our insight of simplifying this programming paradigm is
to designate a set of simple and synthesizable primitives while retaining enough
expressiveness to program a massively concurrent programming system.

Refer [language manual](./docs/language.md) for more details.

