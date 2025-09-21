# Assassyn: **As**ynchronous **S**emantics for **A**rchitectural **S**imulation & **Syn**thesis

[![Tests](https://github.com/were/assassyn/actions/workflows/test.yaml/badge.svg)](https://github.com/were/assassyn/actions/workflows/test.yaml)

Assassyn is aimed at developing a new programming paradigm for hardware development.
The ultimate goal is to unify the hardware modeling (simulation), implementation (RTL),
and verfication.

---

## Getting Started

You either refer to the [docker doc](./docs/docker.md) to use the framework in a virtual
machine, or build it on your physical machine. The instructions below are Ubuntu based:

1. Make sure you have all the repos propoerly cloned.
````sh
git submodule update --init --recursive
````

2. Have your packaged updated.
````sh
sudo apt-get update
````

3. This commands rips of the Docker container build command to install all the dependent packages.
````sh
sudo apt-get install -y $(sed -n '/apt-get install/,/apt-get clean/p' Dockerfile | tail -n+2 | head -n-1 | sed 's/\\//')
````

4. Have this repo built from source.
````sh
source setup.sh
source init.sh
````

5. Verify your installation.
````sh
python -c 'import assassyn' # import this module
echo $? # 0 is expected
````
