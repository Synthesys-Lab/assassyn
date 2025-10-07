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

# Apply patch if it exists and hasn't been applied yet
PATCH_FILE="$ASSASSYN_HOME/scripts/ramulator2-template.patch"
if [ -f "$PATCH_FILE" ]; then
  # Check if patch is already applied by testing if git apply would work in reverse
  if git apply --reverse --check "$PATCH_FILE" 2>/dev/null; then
    echo "Ramulator2 patch already applied — skipping patch step."
  else
    echo "Applying ramulator2 patch..."
    git apply "$PATCH_FILE" 2>/dev/null
    if [ $? -ne 0 ]; then
      # Check if the patch failed because changes are already applied
      # by checking if the expected changes exist in the files
      if grep -q "template as<T>" src/base/param.h 2>/dev/null && grep -q "*.dylib" .gitignore 2>/dev/null; then
        echo "Ramulator2 patch changes already present — skipping patch step."
      else
        echo "Failed to apply ramulator2 patch."
        cd "$RESTORE"
        return 1
      fi
    fi
  fi
else
  echo "Patch file not found: $PATCH_FILE"
fi

mkdir -p build
cd build
cmake ..
make -j

if [ $? -ne 0 ]; then
  echo "Failed to build Ramulator2."
  cd "$RESTORE"
  return 1
fi

echo "Ramulator2 build completed."
cd "$RESTORE"
return 0