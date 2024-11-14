from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
import time
import random
import heapq

current_seed = int(time.time())
# current_seed = 1731606257

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

        ZERO = Int(1)(0)
        ONE = Int(1)(1)

        
        index_bit = max(self.level, 1)        
        index0 = index[0:index_bit-1].bitcast(Int(index_bit))
        type0 = self.elements[index0].dtype
        value0 = self.elements[index0].value
        occupied0 = self.elements[index0].is_occupied
        vacancy0 = self.elements[index0].vacancy
        
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
        
        value_mask = Bits(1)(1).concat(Bits(32)(0))
        value_c1 = action.select(action.concat(value), occupied1.concat(value1)) ^ value_mask
        value_c2 = action.select(occupied0.concat(value0), occupied2.concat(value2)) ^ value_mask
        value_c = ((value_c1<value_c2).select(value_c1, value_c2))[0:31].bitcast(Int(32))
        value_n = ((value_c1<value_c2).select(value_c2, value_c1))[0:31].bitcast(Int(32))
        value_n = action.select(value_n, Int(32)(0))
        
        # Current data
        vacancy_c = action.select(vacancy0-Int(self.height)(1), vacancy0+Int(self.height)(1))   # Basic changes for push and pop
        # log("vacancy_c:{}", vacancy_c)
        vacancy_c = (action|occupied1|occupied2).select(vacancy_c, vacancy0)    # It's pop, and the two child elements are empty
        # log("vacancy_c:{}", vacancy_c)
        vacancy_c = occupied0.select(vacancy_c, vacancy0)   # The current element is empty
        
        occupied_c = action.select(Int(1)(1), Int(1)(0))    # Basic changes for push and pop
        # log("occupied_c:{}", occupied_c)
        occupied_c = (~action&(occupied1|occupied2)).select(Int(1)(1), occupied_c)  # It's pop, and at least one child element is non empty

        

        
        # log("NEW CURRENT VALUE:\t value: {}\toccupied: {}\tvacancy: {}", value_c, occupied_c, vacancy_c)
        
        # Next data
        vacancy_c1 = Int(33-self.height)(0).concat(vacancy2).bitcast(Int(32))
        vacancy_c2 = Int(33-self.height)(0).concat(vacancy1).bitcast(Int(32))
        index_c1 = action.select(vacancy_c1, value1)
        index_c2 = action.select(vacancy_c2, value2)
        index_cn = (index_c1<=index_c2).select(index1, index2)
        index_o = (action^occupied1).select(index1,index2)
        
        index_n = (occupied1^occupied2).select(index_o, index_cn)


        call_n = (action&occupied0&(vacancy0>Int(self.height)(0))).select(Int(1)(1), Int(1)(0))
        call_n = (~action&(occupied1|occupied2)).select(Int(1)(1), call_n)
        
                
        # POP
        with Condition(~action):
            # The current element is occupied.
            with Condition(occupied0):
                with Condition(self.level_I == Int(32)(0)):
                    log("Pop: {}", value0)
        
        self.elements[index0] = type0.bundle(value=value_c, is_occupied=occupied_c, vacancy=vacancy_c)
        with Condition(call_n):
            if next_elements:
                # log("------------------------------------------------------NEW NEXT VALUE:\t value: {}\tindex: {}\t call:{}", value_n, index_n, call_n)
                call = next_layer.async_called(action=action, index=index_n, value=value_n)
                call.bind.set_fifo_depth(action=1, index=1, value=1)

        # # PUSH
        # with Condition(action):
        #     # The current element is valid.
        #     with Condition(~occupied0):
        #         self.elements[index0] = type0.bundle(value=value, is_occupied=ONE, vacancy=vacancy0)
                
        #         log("OLD CURRENT VALUE:\t value: {}\toccupied: {}\tvacancy: {}", value, ONE, vacancy0)
                
        #         log("Push {}  \tin\tLevel_{}[{}]\tFrom  {} + {} + {}\tto  {} + {} + {}",
        #             value, self.level_I, index0, value0, occupied0, vacancy0, value, ONE, vacancy0)

        #     # The current element is occupied.
        #     with Condition(occupied0):
        #         # There is no vacancy on the subtree.
        #         with Condition(vacancy0 == Int(self.height)(0)):
        #             log("Push {}  \tPush failed, There is no vacancy!", value)
                    
        #         # There is vacancy on the subtree.
        #         with Condition(vacancy0 > Int(self.height)(0)):
        #             vacancy = vacancy0 - Int(self.height)(1)
        #             # value write to current level
        #             value_current = (value > value0).select(value0, value)
        #             # value write to next level
        #             value_next = (value > value0).select(value, value0)
        #             self.elements[index0] = type0.bundle(value=value_current, is_occupied=ONE, vacancy=vacancy)
                    
        #             log("OLD CURRENT VALUE:\t value: {}\toccupied: {}\tvacancy: {}", value_current, ONE, vacancy)
                    
        #             log("Push {}  \tin\tLevel_{}[{}]\tFrom  {} + {} + {}\tto  {} + {} + {}",
        #                 value, self.level_I, index0, value0, occupied0, vacancy0, value_current, ONE, vacancy)
                    
        #             # Call next layer
        #             if next_elements:
        #                 # At least one child is valid
        #                 with Condition(~occupied1 | ~occupied2):
        #                     valid_ab = (~occupied2).concat(~occupied1).bitcast(Int(2))
        #                     pred = ((~valid_ab) + Int(2)(1)) & valid_ab
        #                     index_next = pred.select1hot(index1, index2)
        #                     call = next_layer.async_called(action=ONE, index=index_next, value=value_next)
        #                     log("------------------------------------------------------OLD NEXT VALUE:\t value: {}\tindex: {}\t call:{}", value_next, index_next, Int(1)(1))
        #                     call.bind.set_fifo_depth(action=1, index=1, value=1)

        #                 # Two child nodes are both occupied.
        #                 with Condition(occupied1 & occupied2):
        #                     index_next = (vacancy1 < vacancy2).select(index2, index1)
        #                     call = next_layer.async_called(action=ONE, index=index_next, value=value_next)
        #                     log("------------------------------------------------------OLD NEXT VALUE:\t value: {}\tindex: {}\t call:{}", value_next, index_next, Int(1)(1))
        #                     call.bind.set_fifo_depth(action=1, index=1, value=1)

        # # POP
        # with Condition(~action):
        #     # The current element is valid.
        #     with Condition(~occupied0):
        #         log("Pop\t\tPop failed! The heap is empty.")
        #     # The current element is occupied.
        #     with Condition(occupied0):
        #         with Condition(self.level_I == Int(32)(0)):
        #             log("Pop: {}", value0)
        #         with Condition(self.level_I != Int(32)(0)):
        #             log("Pop  {}  \tfrom\tLevel_{}[{}]\tFrom  {} + {} + {}",
        #                 value0, self.level_I, index0, value0, occupied0, vacancy0)
                    
        #         # Call next layer                                        
        #         if next_elements is None:
        #             self.elements[index0] = type0.bundle(value=Int(32)(0), is_occupied=ZERO, vacancy=vacancy0)
                    
        #             log("OLD CURRENT VALUE:\t value: {}\toccupied: {}\tvacancy: {}", Int(32)(0), ZERO, vacancy0)
                    
        #         if next_elements:
        #             # Two child nodes are both occupied.
        #             with Condition(occupied1 & occupied2):
        #                 # value write to current level
        #                 value_update = (value1 < value2).select(value1, value2)
        #                 index_next = (value1 < value2).select(index1, index2)                        
        #                 vacancy = vacancy0 + Int(self.height)(1)
        #                 self.elements[index0] = type0.bundle(value=value_update, is_occupied=ONE, vacancy=vacancy)
                        
        #                 log("OLD CURRENT VALUE:\t value: {}\toccupied: {}\tvacancy: {}", value_update, ONE, vacancy)

        #                 call = next_layer.async_called(action=ZERO, index=index_next, value=Int(32)(0))
        #                 log("------------------------------------------------------OLD NEXT VALUE:\t value: {}\tindex: {}\t call:{}", Int(32)(0), index_next, Int(1)(1))
        #                 call.bind.set_fifo_depth(action=1, index=1, value=1)
                        
        #             # One child is valid, another is occupied.
        #             with Condition(occupied1 ^ occupied2):
        #                 occupied_ab = occupied2.concat(occupied1).bitcast(Int(2))
        #                 pred = ((~occupied_ab) + Int(2)(1)) & occupied_ab
        #                 index_next = pred.select1hot(index1, index2)
        #                 value_update = pred.select1hot(value1, value2)
        #                 vacancy = vacancy0 + Int(self.height)(1)
        #                 self.elements[index0] = type0.bundle(value=value_update, is_occupied=ONE, vacancy=vacancy)
                        
        #                 log("OLD CURRENT VALUE:\t value: {}\toccupied: {}\tvacancy: {}", value_update, ONE, vacancy)

        #                 call = next_layer.async_called(action=ZERO, index=index_next, value=Int(32)(0))
        #                 log("------------------------------------------------------OLD NEXT VALUE:\t value: {}\tindex: {}\t call:{}", Int(32)(0), index_next, Int(1)(1))
        #                 call.bind.set_fifo_depth(action=1, index=1, value=1)

        #             # Two child nodes are both valid.
        #             with Condition(~occupied1 & ~occupied2):
        #                 self.elements[index0] = type0.bundle(value=Int(32)(0), is_occupied=ZERO, vacancy=vacancy0)
                        
        #                 log("OLD CURRENT VALUE:\t value: {}\toccupied: {}\tvacancy: {}", Int(32)(0), ZERO, vacancy0)

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
    
def check_c(raw):
    new_array = []
    old_array = []
    
    for line in raw.splitlines():
        if 'NEW CURRENT VALUE:' in line:
            toks = line.split()
            a = toks[-1]
            b = toks[-3]
            c = toks[-5]
            new_array.append([a, b, c])
        
        if 'OLD CURRENT VALUE:' in line:
            toks = line.split()
            a = toks[-1]
            b = toks[-3]
            c = toks[-5]
            old_array.append([a, b, c])
    
    # 检查数组是否长度相等
    assert len(new_array) == len(old_array), "Arrays have different lengths."
    
    # 逐项比较两个数组
    for i in range(len(new_array)):
        for j in range(len(new_array[i])):
            assert new_array[i][j] == old_array[i][j], f"Mismatch at row {i}, column {j}: {new_array[i][j]} != {old_array[i][j]}"
            
def check_n(raw):
    new_array = []
    old_array = []
    
    for line in raw.splitlines():
        if 'NEW NEXT VALUE:' in line:
            toks = line.split()
            a = toks[-3]
            b = toks[-5]
            new_array.append([a, b])
        
        if 'OLD NEXT VALUE:' in line:
            toks = line.split()
            a = toks[-3]
            b = toks[-5]
            old_array.append([a, b])
    
    # 检查数组是否长度相等
    assert len(new_array) == len(old_array), "Arrays have different lengths."
    
    # 逐项比较两个数组
    for i in range(len(new_array)):
        for j in range(len(new_array[i])):
            assert new_array[i][j] == old_array[i][j], f"Mismatch at row {i}, column {j}: {new_array[i][j]} != {old_array[i][j]}"


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
    print(raw)
    check(raw, heap_height=heap_height)
    # check_c(raw)
    # check_n(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw, heap_height=heap_height)        
        
    print(f"Seed is {current_seed}.") # For reproducing when problems occur
    
if __name__ == '__main__':
    priority_queue(heap_height=3)
