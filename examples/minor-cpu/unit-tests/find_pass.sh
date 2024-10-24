#!/bin/bash

# 读取日志文件路径
logfile=$1

if grep -q "02301063" "$logfile"; then
    # bne     zero,gp,800001ec <pass>
    exit 0
else
    # fail
    exit 1
fi