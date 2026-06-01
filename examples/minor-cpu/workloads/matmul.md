# Matmul Workload Fixture

`matmul.c` is the source sidecar for the cache-sensitivity matrix multiply
row. It is a deterministic single-thread scalar C workload with fixed 8 by 8
matrices and no libc dependency.

## Interface

- `main()` initializes both input matrices, runs the multiply kernel, verifies
  a weighted checksum, and returns `0` on success.

## Data Structures

- `matmul_a` and `matmul_b` store the initialized input matrices.
- `matmul_c` stores the computed output matrix.

## Internal Helpers

- `matmul_init()` initializes all matrix storage.
- `matmul_kernel()` computes the dense matrix product.
- `matmul_checksum()` computes the deterministic output checksum.
