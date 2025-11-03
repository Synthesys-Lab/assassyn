from assassyn.frontend import *
from assassyn.test import run_test
import random
# Stage 3: full adder
class FinalAdder(Module):
    def __init__(self):
        super().__init__(ports={
            'a': Port(Int(32)),
            'b': Port(Int(32)),
            'cnt': Port(Int(32)),
            's': Port(Int(64)),
            'carry': Port(Int(64)),
        }
        )

    @module.combinational
    def build(self):
        a, b, cnt, s, carry = self.pop_all_ports(True)
        result = s + carry
        log("Final result {:?} * {:?} = {:?}", a, b, result)
            
            
# MulStage 2: CSA + Pseudo-Wallace Tree
class CSATree(Module):
    def __init__(self):
        super().__init__(
            ports={
                'a': Port(Int(32)),
                'b': Port(Int(32)),
                'cnt': Port(Int(32)),
                'booth0': Port(Int(64)),
                'booth1': Port(Int(64)),
                'booth2': Port(Int(64)),
                'booth3': Port(Int(64)),
                'booth4': Port(Int(64)),
                'booth5': Port(Int(64)),
                'booth6': Port(Int(64)),
                'booth7': Port(Int(64)),
                'booth8': Port(Int(64)),
                'booth9': Port(Int(64)),
                'booth10': Port(Int(64)),
                'booth11': Port(Int(64)),
                'booth12': Port(Int(64)),
                'booth13': Port(Int(64)),
                'booth14': Port(Int(64)),
                'booth15': Port(Int(64)),
            }
        )

    @module.combinational
    def build(self, finaladder: FinalAdder):
        a, b, cnt, booth0, booth1, booth2, booth3, booth4, booth5, booth6, booth7, \
        booth8, booth9, booth10, booth11, booth12, booth13, booth14, booth15 = self.pop_all_ports(True)
    
        def csa(x1, x2, x3):
            x1_b = x1.bitcast(Bits(64))
            x2_b = x2.bitcast(Bits(64))
            x3_b = x3.bitcast(Bits(64))
            
            s_b = (x1_b ^ x2_b) ^ x3_b
            c_b = ((x1_b & x2_b) | (x2_b & x3_b) | (x3_b & x1_b)) << Bits(64)(1)
            
            return s_b.bitcast(Int(64)), c_b.bitcast(Int(64))
        
        current_pps = []
        current_pps.append(booth0)
        current_pps.append(booth1)
        current_pps.append(booth2)
        current_pps.append(booth3)
        current_pps.append(booth4)
        current_pps.append(booth5)
        current_pps.append(booth6)
        current_pps.append(booth7)
        current_pps.append(booth8)
        current_pps.append(booth9)
        current_pps.append(booth10)
        current_pps.append(booth11)
        current_pps.append(booth12)
        current_pps.append(booth13)
        current_pps.append(booth14)
        current_pps.append(booth15)

        while len(current_pps) > 2:

            next_pps = []
            num_inputs = len(current_pps)
            i = 0
            while (i + 2 < num_inputs):
                in1 = current_pps[i]
                in2 = current_pps[i + 1]
                in3 = current_pps[i + 2]
                s, c = csa(in1, in2, in3)
                next_pps.append(s)
                next_pps.append(c)
                i += 3

            while i < num_inputs:
                next_pps.append(current_pps[i])
                i += 1

            current_pps = next_pps

        final_s = current_pps[0]
        final_carry = current_pps[1]
        log("CSATree: sum = {:?}, carry = {:?}",final_s,final_carry)
        finaladder.async_called(a=a, b=b, cnt=cnt, s=final_s, carry=final_carry)

# MulStage 1: radix-4 booth encoding
class BoothEncoder(Module):
    def __init__(self):
        super().__init__(
            ports={
                'a': Port(Int(32)),
                'b': Port(Int(32)),
                'cnt': Port(Int(32)),
            }
        )

    @module.combinational
    def build(self, csatree: CSATree):
        a, b, cnt = self.pop_all_ports(True)

        b_unsigned = b.bitcast(Bits(32))

        #calculate the complement of -2a
        a_comp = ((Int(32)(-1)*a) << Int(32)(1)).bitcast(Int(32))

        #bit 0-5
        b01 = Int(32)(0)
        b0 = (b_unsigned & Bits(32)(1)).bitcast(Int(32))
        b1 = ((b_unsigned >> Bits(32)(1)) & Bits(32)(1)).bitcast(Int(32))
        b2 = ((b_unsigned >> Bits(32)(2)) & Bits(32)(1)).bitcast(Int(32))
        b3 = ((b_unsigned >> Bits(32)(3)) & Bits(32)(1)).bitcast(Int(32))
        b4 = ((b_unsigned >> Bits(32)(4)) & Bits(32)(1)).bitcast(Int(32))
        b5 = ((b_unsigned >> Bits(32)(5)) & Bits(32)(1)).bitcast(Int(32))
        booth0 = ((b01 + b0) * a + b1 * a_comp).bitcast(Int(64))      
        booth1 = (((b1 + b2) * a + b3 * a_comp) << Int(32)(2)).bitcast(Int(64))
        booth2 = (((b3 + b4) * a + b5 * a_comp) << Int(32)(4)).bitcast(Int(64))
        
        #bit 6-11
        b_shift_unsigned = ((b_unsigned >> Bits(32)(6)) & Bits(32)(31)) # 
        b0 = (b_shift_unsigned & Bits(32)(1)).bitcast(Int(32))
        b1 = ((b_shift_unsigned >> Bits(32)(1)) & Bits(32)(1)).bitcast(Int(32))
        b2 = ((b_shift_unsigned >> Bits(32)(2)) & Bits(32)(1)).bitcast(Int(32))
        b3 = ((b_shift_unsigned >> Bits(32)(3)) & Bits(32)(1)).bitcast(Int(32))
        b4 = ((b_shift_unsigned >> Bits(32)(4)) & Bits(32)(1)).bitcast(Int(32))
        booth3 = (((b5 + b0) * a + b1 * a_comp) << Int(32)(6)).bitcast(Int(64)) 
        booth4 = (((b1 + b2) * a + b3 * a_comp) << Int(32)(8)).bitcast(Int(64))
        b5 = ((b_shift_unsigned >> Bits(32)(5)) & Bits(32)(1)).bitcast(Int(32)) 
        booth5 = (((b3 + b4) * a + b5 * a_comp) << Int(32)(10)).bitcast(Int(64))
        
        #bit 12-17
        b_shift_unsigned = ((b_shift_unsigned >> Bits(32)(6)) & Bits(32)(31)) 
        b0 = (b_shift_unsigned & Bits(32)(1)).bitcast(Int(32))
        b1 = ((b_shift_unsigned >> Bits(32)(1)) & Bits(32)(1)).bitcast(Int(32))
        b2 = ((b_shift_unsigned >> Bits(32)(2)) & Bits(32)(1)).bitcast(Int(32))
        b3 = ((b_shift_unsigned >> Bits(32)(3)) & Bits(32)(1)).bitcast(Int(32))
        b4 = ((b_shift_unsigned >> Bits(32)(4)) & Bits(32)(1)).bitcast(Int(32))      
        booth6 = (((b5 + b0) * a + b1 * a_comp) << Int(32)(12)).bitcast(Int(64)) 
        booth7 = (((b1 + b2) * a + b3 * a_comp) << Int(32)(14)).bitcast(Int(64))
        b5 = ((b_shift_unsigned >> Bits(32)(5)) & Bits(32)(1)).bitcast(Int(32)) 
        booth8 = (((b3 + b4) * a + b5 * a_comp) << Int(32)(16)).bitcast(Int(64))      
        
        #bit 18-23
        b_shift_unsigned = ((b_shift_unsigned >> Bits(32)(6)) & Bits(32)(31)) 
        b0 = (b_shift_unsigned & Bits(32)(1)).bitcast(Int(32))
        b1 = ((b_shift_unsigned >> Bits(32)(1)) & Bits(32)(1)).bitcast(Int(32))
        b2 = ((b_shift_unsigned >> Bits(32)(2)) & Bits(32)(1)).bitcast(Int(32))
        b3 = ((b_shift_unsigned >> Bits(32)(3)) & Bits(32)(1)).bitcast(Int(32))
        b4 = ((b_shift_unsigned >> Bits(32)(4)) & Bits(32)(1)).bitcast(Int(32))
        booth9 = (((b5 + b0) * a + b1 * a_comp) << Int(32)(18)).bitcast(Int(64))  
        booth10 = (((b1 + b2) * a + b3 * a_comp) << Int(32)(20)).bitcast(Int(64))
        b5 = ((b_shift_unsigned >> Bits(32)(5)) & Bits(32)(1)).bitcast(Int(32)) 
        booth11 = (((b3 + b4) * a + b5 * a_comp) << Int(32)(22)).bitcast(Int(64))  
        
        #bit 24-29
        b_shift_unsigned = ((b_shift_unsigned >> Bits(32)(6)) & Bits(32)(31)) 
        b0 = (b_shift_unsigned & Bits(32)(1)).bitcast(Int(32))
        b1 = ((b_shift_unsigned >> Bits(32)(1)) & Bits(32)(1)).bitcast(Int(32))
        b2 = ((b_shift_unsigned >> Bits(32)(2)) & Bits(32)(1)).bitcast(Int(32))
        b3 = ((b_shift_unsigned >> Bits(32)(3)) & Bits(32)(1)).bitcast(Int(32))
        b4 = ((b_shift_unsigned >> Bits(32)(4)) & Bits(32)(1)).bitcast(Int(32))
        booth12 = (((b5 + b0) * a + b1 * a_comp) << Int(32)(24)).bitcast(Int(64))   
        booth13 = (((b1 + b2) * a + b3 * a_comp) << Int(32)(26)).bitcast(Int(64))
        b5 = ((b_shift_unsigned >> Bits(32)(5)) & Bits(32)(1)).bitcast(Int(32)) 
        booth14 = (((b3 + b4) * a + b5 * a_comp) << Int(32)(28)).bitcast(Int(64))  
        
        #bit 30-31
        b_shift_unsigned = ((b_shift_unsigned >> Bits(32)(6)) & Bits(32)(31)) 
        b0 = (b_shift_unsigned & Bits(32)(1)).bitcast(Int(32))
        b1 = ((b_shift_unsigned >> Bits(32)(1)) & Bits(32)(1)).bitcast(Int(32))
        booth15 = (((b5 + b0) * a + b1 * a_comp) << Int(32)(30)).bitcast(Int(64)) 

        log("BoothEncoder: DONE booth coding for {:?} * {:?}", a, b)
        csatree.async_called(a=a, b=b, cnt=cnt, booth0=booth0, booth1=booth1, booth2=booth2, booth3=booth3, booth4=booth4, booth5=booth5,
            booth6=booth6, booth7=booth7, booth8=booth8, booth9=booth9, booth10=booth10, booth11=booth11, booth12=booth12, booth13=booth13,
            booth14=booth14, booth15=booth15,)

class Driver(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, boothencoder: BoothEncoder):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        cond = cnt[0] < Int(32)(95)
        # test input from 0 to 94
        input_a = RegArray(Int(32),1)
        input_b = RegArray(Int(32),1)
        (input_a & self)[0] <= input_a[0] + Int(32)(1)
        (input_b & self)[0] <= input_b[0] + Int(32)(1)
        with Condition(cond):
            boothencoder.async_called(a=input_a[0], b=input_b[0], cnt=cnt[0])

def build_system():

    finaladder = FinalAdder()
    finaladder.build()
    csatree = CSATree()
    csatree.build(finaladder)
    boothencoder = BoothEncoder()
    boothencoder.build(csatree)
    driver = Driver()
    driver.build(boothencoder)

def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        if 'Final result' in i:
            line_toks = i.split()
            c = line_toks[-1]
            b = line_toks[-3]
            a = line_toks[-5]
            assert int(a) * int(b) == int(c)
            cnt += 1

def test_multiplier():
    run_test('multiplier_test', build_system, check_raw)

if __name__ == '__main__':
    test_multiplier()
