RAMULATOR_LIB="$ASSASSYN_HOME/3rd-party/ramulator2/libramulator.so"

# Exit early if libramulator.so already exists
if [ -f "$RAMULATOR_LIB" ]; then
  echo "libramulator.so already exists — skipping build steps."
  return 0
fi

echo "Building Ramulator2..."

RESTORE="$(pwd)"
cd "$ASSASSYN_HOME/3rd-party/ramulator2"
git submodule update --init
mkdir -p build
cd build
cmake ..
make -j$(nproc)

if [ $? -ne 0 ]; then
  echo "Failed to build Ramulator2."
  return 1
fi

echo "Ramulator2 build completed."
cd "$RESTORE"
return 0