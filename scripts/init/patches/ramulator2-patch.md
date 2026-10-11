# Ramulator compatibility patch

The pinned Ramulator revision includes these changes. The lightweight
`ramulator2.patch` remains available for older checkouts and for the existing
`make patch-ramulator2` check/apply interface.

1. `src/base/param.h` uses `.template as<T>()` for YAML conversion in a template.
2. `src/dram_controller/impl/generic_dram_controller.cpp` queues completed writes
   for callbacks, as required by the external memory wrapper. Its acknowledgement
   delay reuses `m_read_latency` after the final write command. This is a
   compatibility approximation, not physical write-recovery timing or an LSQ.
3. Pending writes count toward write queue occupancy, not read queue occupancy.
   Only read completions contribute to average read latency; the no-read case
   reports zero instead of division by zero.

`tests/generic_callbacks.cpp` in Ramulator exercises mixed, read-only and
write-only streams through the real frontend, memory system and DDR4 model.
Configure with `-DBUILD_TESTING=ON`, build, and run `ctest --test-dir build`.

The patch format replaces one exact original line with one or more replacement
lines. `scripts/patch-apply.sh check` recognizes already-applied replacements;
`apply` and `reverse` support testing old checkouts. The committed upstream-style
CRLF line endings are preserved in Ramulator to avoid unrelated whole-file diffs.
The legacy patch applier normalizes line endings when applying to old checkouts.
