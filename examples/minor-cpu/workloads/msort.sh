#!/bin/bash

logfile=$1

last=$(grep "writeback        | x10" "$logfile" | tail -n 1 | awk '{print $NF}')
if [ "$last" = "0x00000000" ]; then
    exit 0
fi
exit 1
