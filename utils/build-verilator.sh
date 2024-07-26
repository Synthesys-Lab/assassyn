#!/usr/bin/env zsh

# NOTE: This script should be sourced by ZSH! O.w. the directory behaviors will be wrong!

RESTORE_DIR=`pwd`

# Go to the script directory
cd `dirname $0`/../

if [ ! -d $REPO_DIR/verilator ]; then
  git clone https://github.com/verilator/verilator/
fi

# Build Verilator
cd verilator
git checkout ca4858eb7f6142a0da367e0c299762d0922f1a6c

# Check if it is Mac OS
if [ `uname` = "Darwin" ]; then
  echo "Patching Verilator for Mac OS"
  # Apply the patch for Mac OS
  git apply ../utils/5222-gnu-20.patch
fi
autoconf
./configure
make -j

cd $RESTORE_DIR
