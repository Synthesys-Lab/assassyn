from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils 
from decoder import *
from instructions import *
from scoreboard import *
import assassyn
 
class Driver(Module):

    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self,  RMT:Array  ):
        
        signals = decode_logic(Bits(32)(2039))
        Index = signals.rs1
        noWAW = Bits(1)(1)
        rmt_entry = (RMT[signals.rd] ==Bits(5)(16) )
        noWAW =  ( rmt_entry ).select(Bits(1)(1),Bits(1)(0))
        # # log("not WAW {:03}",noWAW) 
        wait_until(noWAW)
        RMT[signals.rd]= (signals.rd_valid).select(Index,Bits(5)(SCOREBOARD.size) )
             

def test_record():
    sys = SysBuilder('record')
    with sys:
        reg_map_table = RegArray(Bits(5),33,initializer=[16]*33,attr=[Array.FULLY_PARTITIONED])
 
        driver = Driver()
        call = driver.build( reg_map_table)

    print(sys)

    config = assassyn.backend.config(
            verilog=utils.has_verilator(),
            sim_threshold=200,
            idle_threshold=200,
            random=True)

    simulator_path, verilator_path = elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    
    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        


if __name__ == '__main__':
    test_record()

