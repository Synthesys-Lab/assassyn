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

echo "Adding $REPO_PATH/python to PYTHONPATH"
export PYTHONPATH=$REPO_PATH/python:$PYTHONPATH
echo "Setting ASSASSYN_HOME to $REPO_PATH"
export ASSASSYN_HOME=$REPO_PATH
if [ -d "$REPO_PATH/3rd-party/circt/frontends/PyCDE/dist/lib" ]; then
  echo "Adding PyCDE to PYTHONPATH."
  export PYTHONPATH="$REPO_PATH/3rd-party/circt/frontends/PyCDE/dist/lib:$PYTHONPATH"
fi

# Set up Rust simulator runtime cache directory
export CARGO_TARGET_DIR=$REPO_PATH/.sim-runtime-cache

if [ "$NO_VERILATOR" = false ]; then
  echo "In-repo verilator found, setting VERILATOR_ROOT to $REPO_PATH/verilator"
  export VERILATOR_ROOT=$REPO_PATH/3rd-party/verilator
  export PATH=$VERILATOR_ROOT/bin:$PATH
else
  echo "Verilator is disabled by --no-verilator flag"
fi

# Install pre-commit hook if not already installed
if [ ! -f "$REPO_PATH/.git/hooks/pre-commit" ]; then
  echo "Installing pre-commit hook..."
  ln -s "$REPO_PATH/scripts/pre-commit" "$REPO_PATH/.git/hooks/pre-commit"
  echo "Pre-commit hook installed successfully."
else
  echo "Pre-commit hook already installed."
fi
# Go back to the original directory
cd $RESTORE_DIR
