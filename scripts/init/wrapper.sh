WRAPPER_LIB="$ASSASSYN_HOME/testbench/simulator/build/lib/libwrapper.so"

# Exit early if libwrapper.so already exists
if [ -f "$WRAPPER_LIB" ]; then
  echo "libwrapper.so already exists — skipping build steps."
  return 0
fi
echo "Building Wrapper..."
RESTORE="$(pwd)"
cd "$ASSASSYN_HOME/testbench/simulator"
mkdir -p build
cd build
cmake ..
make -j$(nproc)
if [ $? -ne 0 ]; then
  echo "Failed to build Wrapper."
  return 1
fi
echo "Wrapper build completed."
cd "$RESTORE"
return 0
