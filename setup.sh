# NOTE: This script should be sourced by ZSH! O.w. the directory behaviors will be wrong!

# Check for --no-verilator flag
NO_VERILATOR=false
for arg in "$@"; do
  if [ "$arg" = "--no-verilator" ]; then
    NO_VERILATOR=true
    break
  fi
done

# Restore the original directory
RESTORE_DIR=`pwd`

# Go to the setup.sh directory
cd `dirname $0`

# Use the repository path to set the PYTHONPATH and ASSASSYN_HOME
REPO_PATH=`git rev-parse --show-toplevel`

which sccache > /dev/null 2>&1

if [ $? -eq 0 ]; then
  echo "Setting up SCCACHE to $SCCACHE"
  export RUSTC_WRAPPER=`which sccache`
else
  echo "No sccache found! Skip!"
fi

echo "Adding $REPO_PATH/python to PYTHONPATH"
export PYTHONPATH=$REPO_PATH/python:$PYTHONPATH

echo "Setting ASSASSYN_HOME to $REPO_PATH"
export ASSASSYN_HOME=$REPO_PATH

echo "Adding PyCDE to PYTHONPATH."
export PYTHONPATH="$REPO_PATH/3rd-party/circt/build/tools/circt/python_packages/pycde:$PYTHONPATH"

if [ "$NO_VERILATOR" = false ]; then
  echo "In-repo verilator found, setting VERILATOR_ROOT to $REPO_PATH/verilator"
  export VERILATOR_ROOT=$REPO_PATH/3rd-party/verilator
else
  echo "Verilator is disabled by --no-verilator flag"
fi

# Go back to the original directory
cd $RESTORE_DIR
