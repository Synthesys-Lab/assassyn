#!/usr/bin/env zsh

# Install sccache
cargo install --list | grep sccache > /dev/null
if [ $? -eq 0 ]; then
  echo "\"sccache\" already installed, you can manually update it with \"cargo install sccache\"."
else
  echo "Installing sccache..."
  cargo install sccache
fi

# Install the dependencies
REPO_DIR=`dirname $0`
pip install --user -r $REPO_DIR/python/requirements.txt

# Install PyCDE
echo "Installing PyCDE by building it from source..."
CURRENT_DIR_BEFORE_PYCDE_BUILD="$(pwd)"
cd $REPO_DIR/3rd-party/circt
git submodule update --init
mkdir -p build 
cd build
cmake \
    -DCMAKE_BUILD_TYPE=Debug \
    -DLLVM_ENABLE_PROJECTS=mlir \
    -DLLVM_ENABLE_ASSERTIONS=ON \
    -DLLVM_EXTERNAL_PROJECTS=circt \
    -DLLVM_EXTERNAL_CIRCT_SOURCE_DIR=.. \
    -DMLIR_ENABLE_BINDINGS_PYTHON=ON \
    -DCIRCT_BINDINGS_PYTHON_ENABLED=ON \
    -DCIRCT_ENABLE_FRONTENDS=PyCDE \
    -G Ninja ../llvm/llvm
ninja
cd "$CURRENT_DIR_BEFORE_PYCDE_BUILD"

echo "Installing Verilator by building it from source..."
CURRENT_DIR_BEFORE_VERILATOR_BUILD="$(pwd)"
cd $REPO_DIR/3rd-party/verilator
autoconf
./configure
make -j$(nproc)
cd "$CURRENT_DIR_BEFORE_VERILATOR_BUILD"
