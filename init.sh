#!/usr/bin/env zsh
git submodule update --init --recursive

source ./scripts/init/py-package.sh
source ./scripts/init/circt.sh
source ./scripts/init/verilator.sh
source ./scripts/init/ramulator2.sh
source ./scripts/init/wrapper.sh
