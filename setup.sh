# Restore the original directory
RESTORE_DIR=`pwd`

# Go to the setup.sh directory
cd `dirname $0`

# Use the repository path to set the PYTHONPATH and ASSASSYN_HOME
REPO_PATH=`git rev-parse --show-toplevel`

export PYTHONPATH=$REPO_PATH/python:$PYTHONPATH
export ASSASSYN_HOME=$REPO_PATH

# Go back to the original directory
cd $RESTORE_DIR
