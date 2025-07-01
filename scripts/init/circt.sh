# Install PyCDE

# Default values
LLVM_PARALLEL_COMPILE_JOBS=16
LLVM_PARALLEL_TABLEGEN_JOBS=16
LLVM_PARALLEL_LINK_JOBS=1

# Parse arguments
for arg in "$@"; do
  case $arg in
    --llvm-compile-jobs=*)
      LLVM_PARALLEL_COMPILE_JOBS="${arg#*=}"
      ;;
    --llvm-link-jobs=*)
      LLVM_PARALLEL_LINK_JOBS="${arg#*=}"
      ;;
    --llvm-tbg-jobs=*)
      LLVM_PARALLEL_TABLEGEN_JOBS="${arg#*=}"
      ;;
  esac

done

# TODO: Later add a flag to force CIRCT installation via source
pip install --user pycde --break-system-packages
if [ $? -eq 0 ]; then
  echo "CIRCT installed successfully via pip."
  exit 0
fi

RESTORE=`pwd`

echo "Failed to install CIRCT via pip. Fall back to building from source."
CURRENT_DIR_BEFORE_PYCDE_BUILD="$(pwd)"
cd $ASSASSYN_HOME/3rd-party/circt
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
    -DLLVM_PARALLEL_LINK_JOBS=${LLVM_PARALLEL_LINK_JOBS} \
    -DLLVM_PARALLEL_COMPILE_JOBS=${LLVM_PARALLEL_COMPILE_JOBS} \
    -DLLVM_PARALLEL_TABLEGEN_JOBS=${LLVM_PARALLEL_TABLEGEN_JOBS} \
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
  echo "Failed to build CIRCT. Please check the build output."
  exit 1
fi

cd $RESTORE

echo "CIRCT built successfully."
