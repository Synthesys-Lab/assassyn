#!/bin/bash

grep -E "auipc|writeback.*x05|[e]	own x05|addi.*x05"  $1

