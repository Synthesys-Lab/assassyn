echo "Building Ramulator2 and Wrapper..."

RESTORE="$(pwd)"

# First build Ramulator2
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
      echo "Failed to apply ramulator2 patch."
      cd "$RESTORE"
      return 1
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

# Then build Wrapper
mkdir -p "$ASSASSYN_HOME/tools/c-ramulator2-wrapper"
cd "$ASSASSYN_HOME/tools/c-ramulator2-wrapper"
mkdir -p build
cd build
cmake ..
make -j
if [ $? -ne 0 ]; then
  echo "Failed to build Wrapper."
  cd "$RESTORE"
  return 1
fi
echo "Wrapper build completed."
cd "$RESTORE"
return 0