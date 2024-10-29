import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils

cachesize = 8

class MemMontage(Module):
    def __init__(self, width):
        super().__init__(
            ports={'ldata': Port(Bits(width)),
                   'rdata': Port(Bits(width)),
                   'mark': Port(UInt(8))
                   }, 
        )
        
    @module.combinational
    def build(self):
        
        width = self.ldata.dtype.bits
        width2 = width<<1
        ldata, rdata, mark = self.pop_all_ports(False)
        ldata = Bits(width)(0).concat(ldata).bitcast(Int(width2))
        rdata = Bits(width)(0).concat(rdata).bitcast(Int(width2))
        mark = mark * UInt(8)(32)

        data_joint = (Bits(width2)(0) | ldata).bitcast(UInt(width2))
        # data_joint = data_joint << UInt(8)(1)        
        data_joint = data_joint << mark
        data_joint = data_joint | rdata
        
        data_joint = ldata.concat(rdata)

        log("Cacheline:\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            data_joint[224:255],
            data_joint[192:223],
            data_joint[160:191],
            data_joint[128:159],
            data_joint[96:127],
            data_joint[64:95],
            data_joint[32:63],
            data_joint[0:31])

class MemUser(Module):

    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width))}, 
        )
        
    def kcount(self, maskpart:Bits):
        maskpart = maskpart[0:1]
        k_count = (maskpart[0:0]).select(UInt(8)(2),UInt(8)(0))
        k_count = (maskpart[0:0]^maskpart[1:1]).select(UInt(8)(1),k_count)
        return k_count
        
    @module.combinational
    def build(self, montage):
        
        width = self.rdata.dtype.bits
        rdata = self.pop_all_ports(False)
        bitmask = Bits(cachesize)(0b10101010)
        
        data1 = bitmask[7:7].select(Bits(32)(0).concat(rdata[224:255]).bitcast(UInt(64)), UInt(64)(0))
        data2 = bitmask[6:6].select(Bits(32)(0).concat(rdata[192:223]).bitcast(UInt(64)), UInt(64)(0))
        data3 = bitmask[5:5].select(Bits(32)(0).concat(rdata[160:191]).bitcast(UInt(64)), UInt(64)(0))
        data4 = bitmask[4:4].select(Bits(32)(0).concat(rdata[128:159]).bitcast(UInt(64)), UInt(64)(0))
        data5 = bitmask[3:3].select(Bits(32)(0).concat(rdata[96:127]).bitcast(UInt(64)), UInt(64)(0))
        data6 = bitmask[2:2].select(Bits(32)(0).concat(rdata[64:95]).bitcast(UInt(64)), UInt(64)(0))
        data7 = bitmask[1:1].select(Bits(32)(0).concat(rdata[32:63]).bitcast(UInt(64)), UInt(64)(0))
        data8 = bitmask[0:0].select(Bits(32)(0).concat(rdata[0:31]).bitcast(UInt(64)), UInt(64)(0))

        mask2 = bitmask[6:6].select(UInt(8)(32), UInt(8)(0))
        mask3 = bitmask[5:5].select(UInt(8)(32), UInt(8)(0))
        mask4 = bitmask[4:4].select(UInt(8)(32), UInt(8)(0))
        mask5 = bitmask[3:3].select(UInt(8)(32), UInt(8)(0))
        mask6 = bitmask[2:2].select(UInt(8)(32), UInt(8)(0))
        mask7 = bitmask[1:1].select(UInt(8)(32), UInt(8)(0))
        mask8 = bitmask[0:0].select(UInt(8)(32), UInt(8)(0))
        
        data12 = (((UInt(64)(0) | data1).bitcast(UInt(64))) << mask2) | data2
        data12 = (UInt(64)(0)).concat(data12).bitcast(UInt(128))
        
        data34 = (((UInt(64)(0) | data3).bitcast(UInt(64))) << mask4) | data4
        data34 = (UInt(64)(0)).concat(data34).bitcast(UInt(128))
        
        data56 = (((UInt(64)(0) | data5).bitcast(UInt(64))) << mask6) | data6
        data56 = (UInt(64)(0)).concat(data56).bitcast(UInt(128))
        
        data78 = (((UInt(64)(0) | data7).bitcast(UInt(64))) << mask8) | data8
        data78 = (UInt(64)(0)).concat(data78).bitcast(UInt(128))        
        
        log("data12 {:x}\tdata34 {:x}\tdata56 {:x}\tdata78 {:x}",data12, data34, data56, data78)
        
        mask34 = mask3 + mask4
        mask56 = mask5 + mask6
        mask78 = mask7 + mask8
        
        # data1234 = (((UInt(128)(0) | data12).bitcast(UInt(128))) << mask34) | data34
        
        # data_joint = None

        # for i in range(cachesize):
        #     offest = cachesize - i - 1
        #     data_masked = bitmask[offest:offest].select(rdata[offest*32:offest*32+31], Bits(32)(0))
        #     data_joint = data_masked if data_joint is None else data_joint.concat(data_masked)

        # log("Cacheline:\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        #     data_joint[224:255],
        #     data_joint[192:223],
        #     data_joint[160:191],
        #     data_joint[128:159],
        #     data_joint[96:127],
        #     data_joint[64:95],
        #     data_joint[32:63],
        #     data_joint[0:31])

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user):
        cnt = RegArray(Int(width), 1)
        v = cnt[0]
        we = Int(1)(0)
        re = ~v[0:0]
        plused = v + Int(width)(1)
        raddr = v[0:8].bitcast(Int(9))
        
        shift = Int(9)(cachesize.bit_length())
        
        shift
        addr_access = raddr >> shift
        
        cnt[0] = plused
        sram = SRAM(width, 512, init_file)
        sram.build(we, re, addr_access, v.bitcast(Bits(width)), user)
        with Condition(re):
            sram.bound.async_called()

def check(raw):
    for line in raw.splitlines():
        if '[sram]' in line:
            toks = line.split()
            c = int(toks[-1])
            b = int(toks[-3])
            a = int(toks[-5])
            assert a % 2 == 1 or c == 0, f'Expected odd number or zero, got {line}'
            assert c == a + b, f'{a} + {b} = {c}'


def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        
        montage = MemMontage(width>>1)
        montage.build()
        
        user = MemUser(width)
        user.build(montage)
        
        # Build the driver
        driver = Driver()
        driver.build(width, init_file, user)

    config = backend.config(sim_threshold=200, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    print(raw)
    # check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        # check(raw)

def test_memory():
    impl('memory_v3', 32*cachesize, 'init_2.hex', f'{utils.repo_path()}/python/unit-tests/resources')

if __name__ == "__main__":
        test_memory()
