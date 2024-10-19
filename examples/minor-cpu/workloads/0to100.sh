#!/bin/bash

sum=$(cat $1 | grep "writeback.*x14" | awk '{ sum += $NF } END { print sum }')
acc=$(printf "%d\n" $(cat $1 | grep "writeback.*x10" | tail -n 1 | awk '{ print $NF }'))
ref=$(cat $2 | awk '{ print "0x"$1 }' |  awk '{ sum += $1 } END { print sum }')

if [ $sum -ne $ref ]; then
    echo "Error Sum! $sum != $ref"
    exit 1
fi

if [ $acc -ne $ref ]; then
    echo "Error Acc! $sum != $acc"
    exit 1
fi

