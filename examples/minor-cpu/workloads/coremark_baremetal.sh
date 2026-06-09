#!/usr/bin/env bash
set -euo pipefail

logfile=$1

rg -n "ebreak \| halt \| ecall|trap" "$logfile" > /dev/null
rg -n "mem-write        \| addr: 0x3ff08\|.*wdada: 0x7fffffff" "$logfile" > /dev/null
rg -n "mem-write        \| addr: 0x3ff24\|.*wdada: 0x00000000" "$logfile" > /dev/null
rg -n "mem-write        \| addr: 0x3ff0c\|.*wdada: 0x0000e9f5" "$logfile" > /dev/null
