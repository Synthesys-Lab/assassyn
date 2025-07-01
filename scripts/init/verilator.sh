NO_VERILATOR=false
for arg in "$@"; do
  if [ "$arg" = "--no-verilator" ]; then
    NO_VERILATOR=true
    break
  fi
done

if [ "$NO_VERILATOR" = true ]; then
  echo "Verilator is disabled by --no-verilator flag"
  exit 0
fi

echo "Installing Verilator by building it from source..."

RESTORE="$(pwd)"

cd $ASSASSYN_HOME/3rd-party/verilator
autoconf
./configure
make -j$(nproc)

if [ $? -ne 0 ]; then
  echo "Failed to configure Verilator build. Please check the configuration."
  exit 1
fi

cd $RESTORE