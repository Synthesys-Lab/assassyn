// Generated from DISC_CPU/src/msort.cpp and DISC_CPU/src/msort_dataset.h.

#include <algorithm>
#include <cstddef>
#include <limits>

#ifndef PREALLOCATE
#define PREALLOCATE 0
#endif

using type = unsigned int;

unsigned int stack_space[256] __asm__("stack_space") __attribute__((aligned(16), used));

extern "C" __attribute__((naked, section(".text.startup"))) void _start(void) {
  __asm__ volatile(
    "la gp, __global_pointer$\n"
    "la sp, stack_space\n"
    "addi sp, sp, 1024\n"
    "jal ra, main\n"
    "ebreak\n"
    "1: j 1b\n"
  );
}

static inline void setStats(int enable) {
  if (enable) {
    __asm__ volatile(".global stat_start\nstat_start:");
  } else {
    __asm__ volatile(".global stat_end\nstat_end:");
  }
}

template <typename T>
static inline void printArray(const char name[], int n, const T arr[]) {
  (void)name;
  (void)n;
  (void)arr;
}

template <typename T>
static inline int verify(int n, const volatile T* test, const T* expected) {
  for (int i = 0; i < (n / 2) * 2; i += 2) {
    T t0 = test[i];
    T t1 = test[i + 1];
    T e0 = expected[i];
    T e1 = expected[i + 1];
    if (t0 != e0) {
      return i + 1;
    }
    if (t1 != e1) {
      return i + 2;
    }
  }
  if ((n & 1) != 0 && test[n - 1] != expected[n - 1]) {
    return n;
  }
  return 0;
}

static constexpr int DATA_SIZE = 100;

static type input_data[DATA_SIZE] = {
    179, 968, 116, 259, 844, 769, 182, 1002, 1011, 856, 392, 36, 383, 959, 527, 275, 512,
    874, 851, 592, 238, 608, 930, 457, 0, 234, 563, 168, 844, 513, 886, 730, 767, 159, 743,
    657, 970, 139, 518, 686, 272, 222, 940, 569, 492, 393, 304, 70, 766, 148, 363, 478, 236,
    841, 480, 258, 321, 262, 110, 192, 602, 351, 855, 125, 105, 136, 996, 687, 27, 26, 527,
    531, 576, 826, 567, 469, 391, 537, 388, 759, 325, 819, 744, 668, 69, 1011, 344, 264, 132,
    439, 565, 703, 719, 643, 556, 601, 596, 27, 26, 783,
};

static type verify_data[DATA_SIZE] = {
    0, 26, 26, 27, 27, 36, 69, 70, 105, 110, 116, 125, 132, 136, 139, 148, 159, 168, 179, 182,
    192, 222, 234, 236, 238, 258, 259, 262, 264, 272, 275, 304, 321, 325, 344, 351, 363, 383,
    388, 391, 392, 393, 439, 457, 469, 478, 480, 492, 512, 513, 518, 527, 527, 531, 537, 556,
    563, 565, 567, 569, 576, 592, 596, 601, 602, 608, 643, 657, 668, 686, 687, 703, 719, 730,
    743, 744, 759, 766, 767, 769, 783, 819, 826, 841, 844, 844, 851, 855, 856, 874, 886, 930,
    940, 959, 968, 970, 996, 1002, 1011, 1011,
};

static constexpr type kInf = std::numeric_limits<type>::max();

static void sort(std::size_t n, type arr_in[], type scratch_in[]) {
  type* a = arr_in;
  type* b = scratch_in;

  for (std::size_t i = 1; i < n; i <<= 1) {
    std::swap(a, b);

    for (std::size_t j = 0; j < n; j += (i << 1)) {
      std::size_t l_end = std::min(j + i, n);
      std::size_t r_end = std::min(j + (i << 1), n);

      for (std::size_t l = j, r = l_end, k = j; l < l_end || r < r_end; ++k) {
        type v0 = (l < l_end) ? b[l] : kInf;
        type v1 = (r < r_end) ? b[r] : kInf;

        if (v0 <= v1) {
          a[k] = v0;
          ++l;
        } else {
          a[k] = v1;
          ++r;
        }
      }
    }
  }

  if (a != arr_in) {
    for (std::size_t i = 0; i < n; ++i) {
      arr_in[i] = a[i];
    }
  }
}

extern "C" int main() {
  static type scratch[DATA_SIZE];

  printArray("input", DATA_SIZE, input_data);
  printArray("verify", DATA_SIZE, verify_data);

#if PREALLOCATE
  sort(DATA_SIZE, verify_data, scratch);
  if (verify(DATA_SIZE, input_data, input_data)) {
    return 1;
  }
#endif

  setStats(1);
  sort(DATA_SIZE, input_data, scratch);
  setStats(0);

  printArray("test", DATA_SIZE, input_data);
  return verify(DATA_SIZE, input_data, verify_data);
}
