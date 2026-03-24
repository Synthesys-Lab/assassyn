#!/bin/bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "Usage: $(basename "$0") <workload_name> [log_path]" >&2
  exit 1
fi

NAME=$1
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/.." && pwd)
REPO_DIR=$(cd "${ROOT_DIR}/../.." && pwd)

LOG_PATH="${2:-}"
if [ -z "$LOG_PATH" ]; then
  if [ -f "${PWD}/${NAME}.log" ]; then
    LOG_PATH="${PWD}/${NAME}.log"
  elif [ -f "${REPO_DIR}/${NAME}.log" ]; then
    LOG_PATH="${REPO_DIR}/${NAME}.log"
  else
    LOG_PATH="${ROOT_DIR}/${NAME}.log"
  fi
fi
START_ADDR="${START_ADDR:-}"
END_ADDR="${END_ADDR:-}"
START_VAL="${START_VAL:-0x00000001}"
END_VAL="${END_VAL:-0x00000002}"

if [ ! -f "$LOG_PATH" ]; then
  echo "Missing log: $LOG_PATH" >&2
  exit 1
fi

find_cycle() {
  local addr="$1"
  local val="$2"
  local addr_int=$((addr))
  local addr_short
  local addr_full
  addr_short=$(printf '0x%x' "$addr_int")
  addr_full=$(printf '0x%08x' "$addr_int")
  { grep -E "addr=(${addr_full}|${addr_short}).*final=${val}|mem-write.*addr: (${addr_full}|${addr_short}).*wdada: ${val}" "$LOG_PATH" || true; } \
    | sed -n 's/.*Cycle @\([0-9.]*\):.*/\1/p' \
    | sed 's/\..*//' \
    | head -n 1
}

if [ -z "$START_ADDR" ] || [ -z "$END_ADDR" ]; then
  for candidate in 0x8003ff00 0x3ff00; do
    candidate_short=$(printf '0x%x' $((candidate)))
    candidate_full=$(printf '0x%08x' $((candidate)))
    if grep -q -E "addr: (${candidate_full}|${candidate_short})|addr=(${candidate_full}|${candidate_short})" "$LOG_PATH"; then
      START_ADDR="${START_ADDR:-$candidate}"
      END_ADDR="${END_ADDR:-$(printf '0x%08x' $((candidate + 4)))}"
      break
    fi
  done
fi

START_ADDR="${START_ADDR:-0x8003ff00}"
END_ADDR="${END_ADDR:-0x8003ff04}"

START_CYCLE="$(find_cycle "$START_ADDR" "$START_VAL")"
END_CYCLE="$(find_cycle "$END_ADDR" "$END_VAL")"

if [ -z "$START_CYCLE" ] || [ -z "$END_CYCLE" ]; then
  # Fallback: CSR-based setStats (csrr mcycle/minstret), if marker stores are absent.
  find_csr_cycles() {
    local csr_imm="$1"
    grep -E "imm: ${csr_imm}" "$LOG_PATH" \
      | sed -n 's/.*Cycle @\([0-9.]*\):.*/\1/p' \
      | sed 's/\..*//'
  }
  CSR_CYCLES="$(find_csr_cycles '0xb00')"
  CSR_START="$(echo "$CSR_CYCLES" | head -n 1)"
  CSR_END="$(echo "$CSR_CYCLES" | sed -n '2p')"
  if [ -z "$CSR_END" ]; then
    CSR_END="$(echo "$CSR_CYCLES" | tail -n 1)"
  fi
  if [ -n "$CSR_START" ] && [ -n "$CSR_END" ] && [ "$CSR_START" != "$CSR_END" ]; then
    DIFF=$((CSR_END - CSR_START))
    ABS_DIFF=$(( DIFF < 0 ? -DIFF : DIFF ))
    echo "$ABS_DIFF"
    exit 0
  fi

  CSR_CYCLES="$(find_csr_cycles '0xb02')"
  CSR_START="$(echo "$CSR_CYCLES" | head -n 1)"
  CSR_END="$(echo "$CSR_CYCLES" | sed -n '2p')"
  if [ -z "$CSR_END" ]; then
    CSR_END="$(echo "$CSR_CYCLES" | tail -n 1)"
  fi
  if [ -n "$CSR_START" ] && [ -n "$CSR_END" ] && [ "$CSR_START" != "$CSR_END" ]; then
    DIFF=$((CSR_END - CSR_START))
    ABS_DIFF=$(( DIFF < 0 ? -DIFF : DIFF ))
    echo "$ABS_DIFF"
    exit 0
  fi

  echo "Could not find marker stores or CSR cycles in $LOG_PATH" >&2
  exit 1
fi

DIFF=$((END_CYCLE - START_CYCLE))
ABS_DIFF=$(( DIFF < 0 ? -DIFF : DIFF ))

echo "$ABS_DIFF"
