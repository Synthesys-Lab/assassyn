#!/usr/bin/env bash
set -euo pipefail

workloads_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$workloads_dir/../../.." && pwd)
coremark_src="${COREMARK_SRC:-}"

if [[ ! -d "$coremark_src" ]]; then
  echo "Set COREMARK_SRC to a CoreMark source tree." >&2
  exit 1
fi

out_prefix="$workloads_dir/coremark"

riscv64-unknown-elf-gcc \
  -O2 \
  -std=c11 \
  -march=rv32i \
  -mabi=ilp32 \
  -msmall-data-limit=0 \
  -ffreestanding \
  -fno-builtin \
  -fno-pic \
  -fno-common \
  -ffunction-sections \
  -fdata-sections \
  -DTOTAL_DATA_SIZE=2000 \
  -DITERATIONS=1 \
  -DMAIN_HAS_NOARGC=1 \
  -DHAS_FLOAT=0 \
  -DHAS_STDIO=0 \
  -DHAS_PRINTF=0 \
  -I"$coremark_src" \
  -I"$workloads_dir" \
  -nostdlib \
  -nostartfiles \
  -Wl,-T,"$workloads_dir/coremark.ld" \
  -Wl,-e,_start \
  -Wl,--gc-sections \
  -o "$out_prefix.elf" \
  "$coremark_src/core_list_join.c" \
  "$coremark_src/core_main.c" \
  "$coremark_src/core_matrix.c" \
  "$coremark_src/core_state.c" \
  "$coremark_src/core_util.c" \
  "$workloads_dir/coremark_portme.c" \
  "$workloads_dir/coremark_ee_printf.c" \
  "$workloads_dir/coremark_start.c" \
  -lgcc

riscv64-unknown-elf-objdump -D "$out_prefix.elf" > "$out_prefix.dump"
python3 "$repo_root/examples/minor-cpu/utils/loader.py" --fname "$out_prefix.dump" --odir "$workloads_dir"
