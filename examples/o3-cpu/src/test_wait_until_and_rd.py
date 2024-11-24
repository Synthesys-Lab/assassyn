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
        sb_tail = RegArray(Bits(5),1,initializer=[3])
        signals = decode_logic(Bits(32)(1975))
        
        Index = sb_tail[0]
        with Condition(signals.rd_valid):
            noWAW =  ( RMT[signals.rd] ==Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size) ).select(Bits(1)(1),Bits(1)(0))
            wait_until(noWAW)
        RMT[signals.rd]= (signals.rd_valid).select(Index,Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size) )
        sb_tail[0]= (((sb_tail[0].bitcast(UInt(32)) )+UInt(32)(1) )%SCOREBOARD.sizeI ).bitcast(Bits(SCOREBOARD.Bit_size))
        

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

