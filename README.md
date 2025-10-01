# Assassyn: **As**ynchronous **S**emantics for **A**rchitectural **S**imulation & **Syn**thesis

[![Tests](https://github.com/synthesys-lab/assassyn/actions/workflows/test.yaml/badge.svg)](https://github.com/synthesys-lab/assassyn/actions/workflows/test.yaml)

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
sudo apt-get install -y $(
  awk '/apt-get install/,/apt-get clean/' Dockerfile \
  | sed '1d;$d; s/[\\[:space:]]*$//; s/^[[:space:]]*//' \
  | grep -v '^$' \
  | tr '\n' ' '
)
````

4. Have this repo built from source.
> make sure you are using zsh
````sh
zsh
source setup.sh
source init.sh
````

If you encounter an error such as `Error 1 g++: fatal error: Killed signal terminated program cc1plus compilation terminated.`, it likely means your machine is out of memory (OOM). In this case, try replacing all `make -j` commands in the scripts referenced by [`init.sh`](./init.sh) with `make -j4` or a lower parallelism value to reduce memory usage.




5. Verify your installation.
````sh
python -c 'import assassyn' # import this module
echo $? # 0 is expected
````
