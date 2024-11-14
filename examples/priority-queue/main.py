from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
import time
import random
import heapq

current_seed = int(time.time())

class Layer(Module):
    """
    Initialize the Layer class.

    :param height: The total height of the heap. Must be a positive integer.
    :param level: The current level number. Must be between 0 and height-1.
                    If level is 0, it represents the top layer, and if level equals height-1, it represents the bottom layer.
    :raises ValueError: If height is no more than 0, or if level is out of the valid range.
    """
    def __init__(self, height:int, level:int, elements:RegArray):
        super().__init__(ports={
            'action': Port(Int(1)),      # 0 means push, 1 means pop
            'index': Port(Int(32)),
            'value': Port(Int(32))
        }, no_arbiter=True)
        if height <= 0:
            raise ValueError(f"Height must be a positive integer, got {height}")        
        if level < 0 or level >= height:
            raise ValueError(f"Level must be between 0 and {height-1}, got {level}")
            
        self.height = height-level+1    # Not the heap height, but the layer height. TODO: Vacancy can be reduced by 1 in the future.
        self.level = level
        self.level_I = Int(32)(level)
        self.name = f"level_{level}"
        self.elements = elements
        
    @module.combinational
    def build(self, next_layer: 'Layer' = None, next_elements: RegArray = None):    
        
        action, index, value = self.pop_all_ports(True)        
        index_bit = max(self.level, 1)        
        index0 = index[0:index_bit-1].bitcast(Int(index_bit))
        type0 = self.elements[index0].dtype
        value0 = self.elements[index0].value
        occupied0 = self.elements[index0].is_occupied
        vacancy0 = self.elements[index0].vacancy       
                        
        # Only log the first layer.
        with Condition(self.level_I == Int(32)(0)):
            with Condition(action):
                log("Push: {}", value)
            with Condition(~action):
                # The current element is occupied.
                with Condition(occupied0):
                    log("Pop: {}", value0)
                with Condition(~occupied0):
                    log("Pop\t\tPop failed! The heap is empty.")
        
        # Two child nodes derived from the same parent node.
        index1 = (index0 * Int(32)(2))[0:31].bitcast(Int(32))
        index2 = index1 + Int(32)(1)
        value1 = Int(32)(0)
        value2 = Int(32)(0)
        occupied1 = Int(1)(0)
        occupied2 = Int(1)(0)
        vacancy1 = Int(self.height)(0)
        vacancy2 = Int(self.height)(0)
    
        if next_elements:
            # Extract data from Records
            value1 = next_elements[index1].value
            value2 = next_elements[index2].value
            occupied1 = next_elements[index1].is_occupied
            occupied2 = next_elements[index2].is_occupied
            vacancy1 = next_elements[index1].vacancy
            vacancy2 = next_elements[index2].vacancy
        
        # Each cycle only needs to do 2 things: 1. Determine the data for current layer. 2. Determine the data go to the next layer.
        
        # Determine values.
        value_mask = Bits(1)(1).concat(Bits(32)(0))
        value_c1 = action.select(action.concat(value), occupied1.concat(value1)) ^ value_mask
        value_c2 = action.select(occupied0.concat(value0), occupied2.concat(value2)) ^ value_mask
        value_c = ((value_c1<value_c2).select(value_c1, value_c2))[0:31].bitcast(Int(32))
        value_n = ((value_c1<value_c2).select(value_c2, value_c1))[0:31].bitcast(Int(32))
        
        # Current data
        vacancy_c = action.select(vacancy0-Int(self.height)(1), vacancy0+Int(self.height)(1))
        vacancy_c = (action|occupied1|occupied2).select(vacancy_c, vacancy0)
        vacancy_c = occupied0.select(vacancy_c, vacancy0)      
        occupied_c = action.select(Int(1)(1), Int(1)(0))
        occupied_c = (~action&(occupied1|occupied2)).select(Int(1)(1), occupied_c)
        
        # Next data
        vacancy_c1 = Int(33-self.height)(0).concat(vacancy2).bitcast(Int(32))
        vacancy_c2 = Int(33-self.height)(0).concat(vacancy1).bitcast(Int(32))
        index_c1 = action.select(vacancy_c1, value1)
        index_c2 = action.select(vacancy_c2, value2)
        index_cn = (index_c1<index_c2).select(index1, index2)
        index_o = (action^occupied1).select(index1,index2)        
        index_n = (occupied1^occupied2).select(index_o, index_cn)
        call_n = (action&occupied0&(vacancy0>Int(self.height)(0))).select(Int(1)(1), Int(1)(0))
        call_n = (~action&(occupied1|occupied2)).select(Int(1)(1), call_n)
        
        self.elements[index0] = type0.bundle(value=value_c, is_occupied=occupied_c, vacancy=vacancy_c)
        if next_elements:
            with Condition(call_n):
                call = next_layer.async_called(action=action, index=index_n, value=value_n)
                call.bind.set_fifo_depth(action=1, index=1, value=1)


class HeapPush(Module):    
    def __init__(self):
        super().__init__(
            ports={'push_value': Port(Int(32))},
            no_arbiter=True)

    @module.combinational
    def build(self, layer: Layer):
        push_value = self.pop_all_ports(True)
        bound = layer.bind(action=Int(1)(1), index=Int(32)(0))
        call = bound.async_called(value=push_value)
        call.bind.set_fifo_depth(action=1, index=1, value=1)

        
class HeapPop(Module):    
    def __init__(self):
        super().__init__(ports={}, no_arbiter=True)

    @module.combinational
    def build(self, layer: Layer):
        bound = layer.bind(action=Int(1)(0), index=Int(32)(0), value=Int(32)(1))
        call = bound.async_called()        
        call.bind.set_fifo_depth(action=1, index=1, value=1)
  
      
class Testbench(Module):    
    def __init__(self, heap_height):
        super().__init__(no_arbiter=True, ports={})
        self.size = 2 ** heap_height - 1

    @module.combinational
    def build(self, push: HeapPush, pop: HeapPop):
        random.seed(current_seed)
        cnt = 0
        for i in range(15):
            with Cycle(i * 2 + 1):
                op = random.randint(0, 1)
                if cnt==0 or (op==0 and cnt<self.size):
                    random_integer = random.randint(1, 100)
                    push.async_called(push_value=Int(32)(random_integer))
                    cnt += 1
                elif cnt==self.size or op==1:
                    pop.async_called()
                    cnt -= 1
                else:
                    assert False, "Unreachable branch of heap operation."


def check(raw,heap_height):
    random.seed(current_seed)
    size = 2 ** heap_height - 1
    cnt = 0
    heap = []
    pops = []

    for i in range(15):
        op = random.randint(0, 1)
        if cnt==0 or (op==0 and cnt<size):
            random_integer = random.randint(1, 100)
            heapq.heappush(heap, random_integer)
            cnt += 1        
        elif cnt==size or op==1:
            smallest = heapq.heappop(heap)
            pops.append(smallest)
            cnt -= 1
        else:
            assert False, "Unreachable branch of heap operation."
        
    outputs = []
    for i in raw.split('\n'):
        if f'Pop:' in i:
            line_toks = i.split()
            value = line_toks[-1]
            outputs.append(int(value))
            cnt += 1

    for i in range(len(pops)):
        assert pops[i] == outputs[i] 
    assert len(outputs) == len(pops), f'heap pops: {len(outputs)} != {len(pops)}'


def priority_queue(heap_height=3):    
    # Build a layer with the given heap height and layer level.
    def build_layer(heap_height: int, level: int):
        element_type = Record({  # TODO: Vacancy can be reduced by 1 in the future.
            (0, 31):('value', Int),
            (32, 32): ('is_occupied', Bits),
            (33, 33+heap_height-level): ('vacancy', Int),
        })
        size = 2 ** level
        vacancy = (2 ** (heap_height - level) - 2) << 33
        initializer = [vacancy] * size
        reg_array = RegArray(
            scalar_ty=element_type,
            size=size,
            initializer=initializer
        )
        return reg_array
    
    sys = SysBuilder('priority_queue')
    
    with sys:
        # Generate arrays, each containing a RegArray.
        arrays = [build_layer(heap_height, i) for i in range(heap_height)]         
        # Create a list of layers, num_layers is determined by heap_height.
        layers = [Layer(height=heap_height, level=i, elements=arrays[i]) for i in range(heap_height)]        
        # Establish the relationships between layers
        for i in range(heap_height):
            if i == heap_height - 1:
                layers[i].build()
            else:
                layers[i].build(layers[i+1], arrays[i+1])
        
        heap_push = HeapPush()
        heap_push.build(layers[0])
        
        heap_pop = HeapPop()
        heap_pop.build(layers[0])
        
        testbench = Testbench(heap_height=heap_height)
        testbench.build(heap_push, heap_pop)
    
    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())
  
    raw = utils.run_simulator(simulator_path)
    check(raw, heap_height=heap_height)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw, heap_height=heap_height)        
        
    print(f"Seed is {current_seed}.") # For reproducing when problems occur
    
if __name__ == '__main__':
    priority_queue(heap_height=3)
