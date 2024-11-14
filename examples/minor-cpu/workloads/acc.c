//  riscv64-unknown-elf-gcc -O3 file.c -march=rv32i -mabi=ilp32 -nostdlib

int a[100];
int b[100];

int main() {
  int sum = 0;
  for (int i = 0; i < 100; ++i) {
    a[i] = i;
  }
  for (int i = 0; i < 100; ++i) {
    b[a[i]]++;
    sum = sum + b[a[i]];
  }
  return sum;
}
