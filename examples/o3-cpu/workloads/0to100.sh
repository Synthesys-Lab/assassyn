#!/bin/bash

if [ `uname` == "Darwin" ]; then
    S2N=""
else
    S2N="strtonum"
fi

sum=$(cat $1 | grep "writeback.*x14" | awk "{ sum += $S2N(\$NF) } END { print $S2N(sum) }")
acc=$(printf "%d\n" $(cat $1 | grep "writeback.*x10" | tail -n 1 | awk "{ print $S2N(\$NF) }"))
ref=$(cat $2 | awk '{ print "0x"$1 }' |  awk "{ sum += $S2N(\$1) } END { print sum }")

if [ $sum -ne $ref ]; then
    echo "Error Sum! $sum != $ref"
    exit 1
fi

if [ $acc -ne $ref ]; then
    echo "Error Acc! $sum != $acc"
    exit 1
fi

