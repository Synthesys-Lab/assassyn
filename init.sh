#!/usr/bin/env zsh
git submodule update --init

./scripts/init/py-package.sh
./scripts/init/circt.sh
./scripts/init/verilator.sh
./scripts/init/ramulator2.sh
./scripts/init/wrapper.sh
