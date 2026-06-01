# Histogram Workload Fixture

`histogram.c` is the source sidecar for the cache-sensitivity histogram row.
It is a deterministic single-thread scalar C workload with no libc dependency.

## Interface

- `main()` initializes 128 synthetic input bins, accumulates 16 histogram bins,
  computes a weighted checksum, and returns `0` when the checksum matches.

## Data Structures

- `histogram_inputs` stores the generated input bin stream.
- `histogram_bins` stores the 16 counters updated by the kernel.

## Internal Helpers

- `histogram_seed(index)` maps each input index to a deterministic bin.
