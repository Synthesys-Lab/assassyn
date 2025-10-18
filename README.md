# Assassyn: **As**ynchronous **S**emantics for **A**rchitectural **S**imulation & **Syn**thesis

[![Tests](https://github.com/synthesys-lab/assassyn/actions/workflows/test.yaml/badge.svg)](https://github.com/synthesys-lab/assassyn/actions/workflows/test.yaml)
[![Apptainer Tests](https://github.com/synthesys-lab/assassyn/actions/workflows/apptainer.yaml/badge.svg)](https://github.com/synthesys-lab/assassyn/actions/workflows/apptainer.yaml)

Assassyn is aimed at developing a new programming paradigm for hardware development.
The ultimate goal is to unify the hardware modeling (simulation), implementation (RTL),
and verfication.

---

## Getting Started

### Virtual Machine
You either refer to the [docker](./docs/vm/docker.md) or [apptainer](./docs/vm/apptainer.md)
to use the framework in a virtual machine.


### Physical Machine
It can also be built on your physical machine. The instructions below are Ubuntu based:

1. Make sure you have all the repos propoerly cloned:
````sh
git submodule update --init --recursive
````

2. Install dependences:

````sh
sudo apt-get update
sudo apt-get install -y $(
  awk '/apt-get install/,/apt-get clean/' Dockerfile \
  | sed '1d;$d; s/[\\[:space:]]*$//; s/^[[:space:]]*//' \
  | grep -v '^$' \
  | tr '\n' ' '
)
````

4. Have this repo built from source:
````sh
source setup.sh  # Set up environment variables
make build-all   # Build all components
````

If you do not have enough memory, a lower job number is recommended.

5. Verify your installation.
````sh
python -c 'import assassyn' # import this module
echo $? # 0 is expected
make test-all # Optional, runs all the tests.
echo $?
````

## File Structure

Our file structure is as follows:

```
- assassyn/                 # The main assassyn package
  |- python/                # All the Python-related code
  |  |- assassyn/           # The assassyn Python package
  |  `- ci-tests/           # Application-level tests for CI
  |- 3rd-party/             # External depdendencies
  |  |- circt/              # It relies on CIRCT for Verilog backend
  |  |- ramulator2          # It relies on Ramulator2 for DRAM modeling
  |  `- verilator           # It relies on Verilator for Verilog simulation
  |- scripts/
  |  |- *.patch             # Useful scripts for patching 3rd-party
  |  |- pre-commit          # Git pre-commit hook
  |  `- init/*.inc          # Makefile includes for building components
  |- docs/                  # The chater document of this framework
  |  |- developer/          # Developer guidelines
  |  |- design/             # High-level design decisions of Assassyn
  |  `- builder/            # Document to build and setup the project
  |- tools/                 # All the helper modules
  |  |- c-ramulator-wrapper # Which wraps Ramulator2 in C for FFI access
  |  `- rust-sim-runtime    # Which provides runtime support for Rust simulator, including Ramulator access
  |- examples/              # Example applications
  `- tutorials/             # Tutorials written in Quarto qmd
```
