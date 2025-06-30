#include "Vtb.h"
#include "verilated.h"
#include "verilated_vcd_c.h"
vluint64_t main_time = 0;
double sc_time_stamp() { return main_time; }
int main(int argc, char **argv) {
  Verilated::commandArgs(argc, argv);
  auto* top = new Vtb;
  Verilated::traceEverOn(true);
  auto* tfp = new VerilatedVcdC;
  top->trace(tfp, 99);
  tfp->open("wave.vcd");
  // Simulate until arrive $finish
  while (!Verilated::gotFinish()) {
    top->eval();
    tfp->dump(main_time);
    main_time++;
  }
  tfp->close();
  delete top;
  delete tfp;
  return 0;
}