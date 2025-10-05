#!/bin/zsh

os=$(uname)

cp $1 .$1.bak

while [ -s $1 ]; do

  todo=$(head -n 1 $1)

  echo "\nWorking on: $todo\n"

  claude -p $todo \
      --allowedTools "Bash,Read,Write,Edit,Create" \
      --permission-mode acceptEdits \
      --verbose

  if [ $os = "Darwin" ]; then
    sed -i '' 1d $1
  else
    sed -i 1d $1
  fi

done
