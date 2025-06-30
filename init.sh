#!/usr/bin/env zsh

# Install sccache
cargo install --list | grep sccache > /dev/null
if [ $? -eq 0 ]; then
  echo "\"sccache\" already installed, you can manually update it with \"cargo install sccache\"."
else
  echo "Installing sccache..."
  cargo install sccache
fi


REPO_DIR=`dirname $0`
pip install --user -r $REPO_DIR/python/requirements.txt


# Install PyCDE
python -c "import pycde" &> /dev/null
if [ $? -eq 0 ]; then
  echo "\"PyCDE\" already installed. You can manually update it if needed."
else
  echo "PyCDE not found. Installing and building PyCDE..."
  CURRENT_DIR_BEFORE_PYCDE_BUILD="$(pwd)"
  cd $REPO_DIR/3rd-party/circt
  git submodule update --init
  python -m pip install -r frontends/PyCDE/python/requirements.txt
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
  Ninja
  cd "$CURRENT_DIR_BEFORE_PYCDE_BUILD"

fi

# build Ramulator2
echo "Checking if Ramulator2 is already built..."
RAMULATOR_BUILD_DIR="$REPO_DIR/3rd-party/ramulator2/build"
RAMULATOR_LIB="$REPO_DIR/3rd-party/ramulator2/libramulator.so"
if [ -f "$RAMULATOR_LIB" ]; then
  echo "\"Ramulator2\" already built"
else
  echo "Building Ramulator2..."
  CURRENT_DIR_BEFORE_RAMULATOR_BUILD="$(pwd)"
  cd "$REPO_DIR/3rd-party/ramulator2"
  mkdir -p build
  cd build
  cmake ..
  make -j$(nproc)
  cd "$CURRENT_DIR_BEFORE_RAMULATOR_BUILD"
fi

# build wrapper
# Check if wrapper (simulator) is already built
echo "Checking if wrapper (simulator) is already built..."
WRAPPER_BUILD_DIR="$REPO_DIR/testbench/simulator/build"
WRAPPER_LIB="$WRAPPER_BUILD_DIR/lib/libwrapper.so"

if [ -f "$WRAPPER_LIB" ]; then
  echo "\"libwrapper.so\" already built at $WRAPPER_LIB"
else
  CURRENT_DIR_BEFORE_WRAPPER_BUILD="$(pwd)"
  echo "Building libwrapper.so from testbench/simulator..."
  cd "$REPO_DIR/testbench/simulator"
  mkdir -p build
  cd build
  cmake ..
  make
  cd "$CURRENT_DIR_BEFORE_WRAPPER_BUILD"
fi