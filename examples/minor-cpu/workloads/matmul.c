#define MATMUL_N 8
#define MATMUL_ELEMS (MATMUL_N * MATMUL_N)

static volatile int matmul_a[MATMUL_ELEMS];
static volatile int matmul_b[MATMUL_ELEMS];
static volatile int matmul_c[MATMUL_ELEMS];

static void matmul_init(void) {
  for (int i = 0; i < MATMUL_N; ++i) {
    for (int j = 0; j < MATMUL_N; ++j) {
      matmul_a[i * MATMUL_N + j] = (i * 3 + j * 5 + 1) & 15;
      matmul_b[i * MATMUL_N + j] = (i * 7 + j * 2 + 3) & 15;
      matmul_c[i * MATMUL_N + j] = 0;
    }
  }
}

static void matmul_kernel(void) {
  for (int i = 0; i < MATMUL_N; ++i) {
    for (int j = 0; j < MATMUL_N; ++j) {
      int sum = 0;
      for (int k = 0; k < MATMUL_N; ++k) {
        sum += matmul_a[i * MATMUL_N + k] * matmul_b[k * MATMUL_N + j];
      }
      matmul_c[i * MATMUL_N + j] = sum;
    }
  }
}

static int matmul_checksum(void) {
  int checksum = 0;
  for (int i = 0; i < MATMUL_ELEMS; ++i) {
    checksum += (i + 1) * matmul_c[i];
  }
  return checksum;
}

int main(void) {
  matmul_init();
  matmul_kernel();
  return matmul_checksum() == 902656 ? 0 : 1;
}
