#!/bin/zsh

# Restore the original directory
RESTORE_DIR=`pwd`

function test_case {
  eval $1
  if [ $? -ne 0 ]; then
    echo "Test failed: $1"
    cd $RESTORE_DIR
    exit 1
  fi
}

# Go to the unit-test directory
cd `dirname $0`
REPO_DIR=`git rev-parse --show-toplevel`

# Test unit tests
cd $REPO_DIR/python/unit-tests
test_case "pytest --workers 8"

# Test examples

# Systolic Array
cd $REPO_DIR/examples/systolic-array/
test_case "python systolic_array.py"

# A single-issue CPU
cd $REPO_DIR/examples/cpu
test_case "python src/main.py"

cd $RESTORE_DIR
