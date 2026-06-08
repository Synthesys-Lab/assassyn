#!/usr/bin/env bash
set -euo pipefail

logfile=$1

rg -n "ebreak \| halt \| ecall|trap" "$logfile" > /dev/null
rg -n "mem-write        \| addr: 0x3ff08\|.*wdada: 0x7fffffff" "$logfile" > /dev/null
