# NOTE: This script should be sourced by ZSH! O.w. the directory behaviors will be wrong!

# Use the repository path to set the PYTHONPATH and ASSASSYN_HOME
REPO_PATH=`git rev-parse --show-toplevel`

ENV_JSON=""

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

# Activate virtual environment if it exists
if [ -d "$REPO_PATH/.assassyn-venv" ]; then
  echo "Activating Python virtual environment..."
  . "$REPO_PATH/.assassyn-venv/bin/activate"
else
  echo "No virtual environment found. Run 'make install-py-package' to create one."
fi

echo "In-repo verilator found, setting VERILATOR_ROOT to $REPO_PATH/verilator"
export VERILATOR_ROOT=$REPO_PATH/3rd-party/verilator
export PATH=$VERILATOR_ROOT/bin:$PATH

# Install pre-commit hook if not already installed
if [ ! -f "$REPO_PATH/.git/hooks/pre-commit" ]; then
  echo "Installing pre-commit hook..."
  ln -s "$REPO_PATH/scripts/pre-commit" "$REPO_PATH/.git/hooks/pre-commit"
  echo "Pre-commit hook installed successfully."
else
  echo "Pre-commit hook already installed."
fi
