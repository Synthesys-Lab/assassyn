#include <stdint.h>

extern int main(void);

extern volatile uint16_t coremark_seedcrc_result;
extern volatile uint16_t coremark_crclist_result;
extern volatile uint16_t coremark_crcmatrix_result;
extern volatile uint16_t coremark_crcstate_result;
extern volatile uint16_t coremark_crcfinal_result;
extern volatile int16_t coremark_total_errors_result;
extern volatile uint32_t coremark_total_ticks_result;

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

    mmio_write(0x0003ff0cu, coremark_seedcrc_result);
    mmio_write(0x0003ff10u, coremark_crclist_result);
    mmio_write(0x0003ff14u, coremark_crcmatrix_result);
    mmio_write(0x0003ff18u, coremark_crcstate_result);
    mmio_write(0x0003ff1cu, coremark_crcfinal_result);
    mmio_write(0x0003ff20u, coremark_total_ticks_result);
    mmio_write(0x0003ff24u, (uint32_t)(int32_t)coremark_total_errors_result);
    mmio_write(0x0003ff08u,
               (coremark_total_errors_result == 0) ? 0x7fffffffu : 0u);

    __asm__ volatile("ebreak");
    while (1) { }
}
