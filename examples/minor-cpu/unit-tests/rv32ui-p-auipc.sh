#!/bin/bash

grep -E "auipc|writeback.*x05|[e]	own x05|addi.*x05"  $1

grep -E "auipc|writeback.*x05|[e] own x05|addi.*x05|rd-x05|write.*csr" log

grep -E "auipc|writeback.*x05|[e] own x05|addi.*x05|rd-x05|write.*csr\[01" log

grep -E "auipc|writeback.*x10|[e] own x10|addi.*x10|rd-x10|write.*csr\[01" log

grep -E "auipc|writeback.*x07|[e] own x07|addi.*x07|rd-x07|write.*csr\[01" log
