# NOTE: This script should be sourced by ZSH! O.w. the directory behaviors will be wrong!

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

if [ -d $REPO_PATH/verilator ]; then
  echo "In-repo verilator found, setting VERILATOR_ROOT to $REPO_PATH/verilator"
  export VERILATOR_ROOT=$REPO_PATH/verilator
fi

# Go back to the original directory
cd $RESTORE_DIR
