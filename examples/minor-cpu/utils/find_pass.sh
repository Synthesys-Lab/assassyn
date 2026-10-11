#!/bin/sh
# The legacy RV32I success branch must execute, not merely be prefetched.
set -eu
logfile=$1
grep -Eq '\[E\].*decoded[[:space:]]+\|[[:space:]]+raw: 0x02301063([[:space:]]|$)' "$logfile" &&
    grep -Eq '\[E\].*(ebreak \| halt \| ecall|[[:space:]]trap[[:space:]]*$)' "$logfile"
