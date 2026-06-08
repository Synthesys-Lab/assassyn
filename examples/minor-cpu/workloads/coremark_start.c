#include <stdint.h>

extern int main(void);
extern volatile uint32_t coremark_status;
extern char __stack_top[];

static inline void mmio_write(uint32_t addr, uint32_t value)
{
    *(volatile uint32_t *)addr = value;
}

__attribute__((section(".text.init")))
void _start(void)
{
    __asm__ volatile("la gp, __global_pointer$");
    __asm__ volatile("la sp, __stack_top");

    mmio_write(0x0003ff00u, 1u);
    (void)main();
    mmio_write(0x0003ff04u, 2u);
    mmio_write(0x0003ff08u, coremark_status);

    __asm__ volatile("ebreak");
    while (1) { }
}
