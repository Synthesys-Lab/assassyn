# Assassyn: **As**ynchronous **S**emantics for **A**rchitectural **S**imulation & **Syn**thesis

Assassyn is aimed at developing a new programming paradigm for hardware development.
The ultimate goal is to unify the hardware modeling (simulation), implementation (RTL),
and verfication.

---

## Getting Started

We tried minimizing the depencencies to make it easy to get started.

### Built from Source

Prerequisites packages can be found in `Dockerfile`, and you should use `zsh` as your shell.
As `zsh` has different behavior on accessing the 0-th argument, the shell path itself, of a script.


````sh
git submodule update --init --recursive
source setup.sh
source init.sh
python -c "import assassyn"
````

## Run an Example

TODO: A tutorial to running the example here.
