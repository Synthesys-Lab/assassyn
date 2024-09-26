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
    @module.constructor
    def __init__(self, height:int, level:int, elements:RegArray):
        super().__init__(disable_arbiter_rewrite=True)
        if height <= 0:
            raise ValueError(f"Height must be a positive integer, got {height}")        
        if level < 0 or level >= height:
            raise ValueError(f"Level must be between 0 and {height-1}, got {level}")
            
        self.height = height-level+1    # Not the heap height, but the layer height. TODO: Vacancy can be reduced by 1 in the future.
        self.level = level
        self.level_I = Int(32)(level)
        self.name = f"level_{level}"
        self.elements = elements
        
        self.action = Port(Int(1))      # 0 means push, 1 means pop
        self.index = Port(Int(32))
        self.value = Port(Int(32))

    @module.combinational
    def build(self, next_layer: 'Layer' = None, next_elements: RegArray = None):    
        ZERO = Int(1)(0)
        ONE = Int(1)(1)
        
        index_bit = max(self.level, 1)        
        index0 = self.index[0:index_bit-1].bitcast(Int(index_bit))
        type0 = self.elements[index0].dtype
        value0 = self.elements[index0].value
        occupied0 = self.elements[index0].is_occupied
        vacancy0 = self.elements[index0].vacancy
        
        if next_elements:
            # Two child nodes derived from the same parent node.
            index1 = (index0 * Int(32)(2))[0:31].bitcast(Int(32))
            index2 = index1 + Int(32)(1)
            # Extract datas from Records
            value1 = next_elements[index1].value
            value2 = next_elements[index2].value
            occupied1 = next_elements[index1].is_occupied
            occupied2 = next_elements[index2].is_occupied
            vacancy1 = next_elements[index1].vacancy
            vacancy2 = next_elements[index2].vacancy

        # PUSH
        with Condition(~self.action):
            # The current element is valid.
            with Condition(~occupied0):
                self.elements[index0] = type0.bundle(value=self.value, is_occupied=ONE, vacancy=vacancy0)
                log("Push {}  \tin\tLevel_{}[{}]\tFrom  {} + {} + {}\tto  {} + {} + {}",
                    self.value, self.level_I, index0, value0, occupied0, vacancy0, self.value, ONE, vacancy0)
                
            # The current element is occupied.
            with Condition(occupied0):
                # There is no vacancy on the subtree.
                with Condition(vacancy0 == Int(self.height)(0)):
                    log("Push {}  \tPush failed, There is no vacancy!", self.value)
                    
                # There is vacancy on the subtree.
                with Condition(vacancy0 > Int(self.height)(0)):
                    vacancy = vacancy0 - Int(self.height)(1)
                    # value write to current level
                    value_current = (self.value > value0).select(value0, self.value)
                    # value write to next level
                    value_next = (self.value > value0).select(self.value, value0)
                    self.elements[index0] = type0.bundle(value=value_current, is_occupied=ONE, vacancy=vacancy)
                    log("Push {}  \tin\tLevel_{}[{}]\tFrom  {} + {} + {}\tto  {} + {} + {}",
                        self.value, self.level_I, index0, value0, occupied0, vacancy0, value_current, ONE, vacancy)
                    
                    # Call next layer
                    if next_elements:
                        # At least one child is valid
                        with Condition(~occupied1 | ~occupied2):
                            valid_ab = (~occupied2).concat(~occupied1).bitcast(Int(2))
                            pred = ((~valid_ab) + Int(2)(1)) & valid_ab
                            index_next = pred.select1hot(index1, index2)
                            call = next_layer.async_called(action=ZERO, index=index_next, value=value_next)
                            call.bind.set_fifo_depth(action=1, index=1, value=1)

                        # Two child nodes are both occupied.
                        with Condition(occupied1 & occupied2):
                            index_next = (vacancy1 < vacancy2).select(index2, index1)
                            call = next_layer.async_called(action=ZERO, index=index_next, value=value_next)
                            call.bind.set_fifo_depth(action=1, index=1, value=1)

        # POP
        with Condition(self.action):
            # The current element is valid.
            with Condition(~occupied0):
                log("Pop\t\tPop failed! The heap is empty.")
            # The current element is occupied.
            with Condition(occupied0):
                with Condition(self.level_I == Int(32)(0)):
                    log("Pop: {}", value0)
                with Condition(self.level_I != Int(32)(0)):
                    log("Pop  {}  \tfrom\tLevel_{}[{}]\tFrom  {} + {} + {}",
                        value0, self.level_I, index0, value0, occupied0, vacancy0)
                    
                # Call next layer                                        
                if next_elements is None:
                    self.elements[index0] = type0.bundle(value=Int(32)(0), is_occupied=ZERO, vacancy=vacancy0)
                    
                if next_elements:                    
                    # Two child nodes are both occupied.
                    with Condition(occupied1 & occupied2):
                        # value write to current level
                        value_update = (value1 < value2).select(value1, value2)
                        index_next = (value1 < value2).select(index1, index2)                        
                        vacancy = vacancy0 + Int(self.height)(1)
                        self.elements[index0] = type0.bundle(value=value_update, is_occupied=ONE, vacancy=vacancy)

                        call = next_layer.async_called(action=ONE, index=index_next, value=Int(32)(0))
                        call.bind.set_fifo_depth(action=1, index=1, value=1)
                        
                    # One child is valid, another is occupied.
                    with Condition(occupied1 ^ occupied2):
                        occupied_ab = occupied2.concat(occupied1).bitcast(Int(2))
                        pred = ((~occupied_ab) + Int(2)(1)) & occupied_ab
                        index_next = pred.select1hot(index1, index2)
                        value_update = pred.select1hot(value1, value2)
                        vacancy = vacancy0 + Int(self.height)(1)
                        self.elements[index0] = type0.bundle(value=value_update, is_occupied=ONE, vacancy=vacancy)

                        call = next_layer.async_called(action=ONE, index=index_next, value=Int(32)(0))
                        call.bind.set_fifo_depth(action=1, index=1, value=1)

                    # Two child nodes are both valid.
                    with Condition(~occupied1 & ~occupied2):
                        self.elements[index0] = type0.bundle(value=Int(32)(0), is_occupied=ZERO, vacancy=vacancy0)

class HeapPush(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)
        self.push_value = Port(Int(32))

    @module.combinational
    def build(self, layer: Layer):
        bound = layer.bind(action=Int(1)(0), index=Int(32)(0))
        call = bound.async_called(value=self.push_value)
        call.bind.set_fifo_depth(action=1, index=1, value=1)

        
class HeapPop(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)

    @module.combinational
    def build(self, layer: Layer):
        bound = layer.bind(action=Int(1)(1), index=Int(32)(0), value=Int(32)(1))
        call = bound.async_called()        
        call.bind.set_fifo_depth(action=1, index=1, value=1)
        
class Testbench(Module):
    
    @module.constructor
    def __init__(self, heap_height):
        super().__init__(disable_arbiter_rewrite=True)
        self.size = 2 ** heap_height - 1

    @module.combinational
    def build(self, push: HeapPush, pop: HeapPop):
        random.seed(current_seed)
        cnt = 0
        for i in range(15):
            with Cycle(i * 2 + 1):
                op = random.randint(0, 1)
                if cnt == 0 or (op == 0 and cnt<self.size):
                    random_integer = random.randint(1, 100)
                    push.async_called(push_value=Int(32)(random_integer))
                    cnt += 1
                elif cnt == self.size or op == 1:
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
        if cnt == 0 or (op == 0 and cnt<size):
            random_integer = random.randint(1, 100)
            heapq.heappush(heap, random_integer)
            cnt += 1        
        elif cnt == size or op == 1:
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
        element_type = Record(value=Int(32), is_occupied=Bits(1), vacancy=Int(heap_height-level+1)) # TODO: Vacancy can be reduced by 1 in the future.
        size = 2 ** level
        vacancy = 2 ** (heap_height - level) - 2
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
    
    print(f"Seed is {current_seed}.") # For reproducing when problems occur    
    raw = utils.run_simulator(simulator_path)
    check(raw, heap_height=heap_height)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw, heap_height=heap_height)    
    
if __name__ == '__main__':
    priority_queue(heap_height=3)
