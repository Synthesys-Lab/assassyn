#!/usr/bin/env zsh

# Install sccache
# cargo install --list | grep sccache > /dev/null
# if [ $? -eq 0 ]; then
#   echo "\"sccache\" already installed, you can manually update it with \"cargo install sccache\"."
# else
#   echo "Installing sccache..."
#   cargo install sccache
# fi

# Install the dependencies
REPO_DIR=`dirname $0`
pip install --user -r $REPO_DIR/python/requirements.txt --break-system-packages

if [ $? -ne 0 ]; then
  echo "Failed to install Python dependencies. Please check the requirements."
  exit 1
fi

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
    -DLLVM_TARGETS_TO_BUILD="host;RISCV" \
    -DLLVM_PARALLEL_LINK_JOBS=1 \
    -DLLVM_PARALLEL_COMPILE_JOBS=16 \
    -DLLVM_PARALLEL_TABLEGEN_JOBS=16 \
    -DMLIR_ENABLE_BINDINGS_PYTHON=ON \
    -DCIRCT_BINDINGS_PYTHON_ENABLED=ON \
    -DCIRCT_ENABLE_FRONTENDS=PyCDE \
    -G Ninja ../llvm/llvm

if [ $? -ne 0 ]; then
  echo "Failed to configure CIRCT build. Please check the CMake configuration."
  exit 1
fi

ninja

if [ $? -ne 0 ]; then
  echo "Failed to build PyCDE. Please check the build output."
  exit 1
fi

cd "$CURRENT_DIR_BEFORE_PYCDE_BUILD"

echo "Installing Verilator by building it from source..."
CURRENT_DIR_BEFORE_VERILATOR_BUILD="$(pwd)"
cd $REPO_DIR/3rd-party/verilator
autoconf
./configure
make -j$(nproc)

if [ $? -ne 0 ]; then
  echo "Failed to configure Verilator build. Please check the configuration."
  exit 1
fi

cd "$CURRENT_DIR_BEFORE_VERILATOR_BUILD"
