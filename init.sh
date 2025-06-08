#!/usr/bin/env zsh

# Install sccache
cargo install --list | grep sccache > /dev/null
if [ $? -eq 0 ]; then
  echo "\"sccache\" already installed, you can manually update it with \"cargo install sccache\"."
else
  echo "Installing sccache..."
  cargo install sccache
fi


REPO_DIR=`dirname $0`
pip install --user -r $REPO_DIR/python/requirements.txt

