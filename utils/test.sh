#!/bin/zsh

# Restore the original directory
RESTORE_DIR=`pwd`

# Go to the unit-test directory
cd `dirname $0`
REPO_DIR=`git rev-parse --show-toplevel`

# Test unit tests
cd $REPO_DIR/python/unit-tests
pytest --workers 8

# Test examples

# Systolic Array
cd $REPO_DIR/examples/systolic-array/
python systolic_array.py

# A single-issue CPU
cd $REPO_DIR/examples/cpu
python src/main.py


cd $RESTORE_DIR
