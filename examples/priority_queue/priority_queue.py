from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Layer(Module):
    """
    Initialize the Layer class.

    :param height: The total height of the heap. Must be a positive integer.
    :param level: The current level number. Must be between 0 and height-1.
                    If level is 0, it represents the top layer, and if level equals height, it represents the bottom layer.
    :raises ValueError: If height is no more than 0, or if level is out of the valid range.
    """
    @module.constructor
    def __init__(self, height:int, level:int, elements:RegArray):
        super().__init__(disable_arbiter_rewrite=True)
        if height <= 0:
            raise ValueError(f"Height must be a positive integer, got {height}")        
        if level < 0 or level >= height:
            raise ValueError(f"Level must be between 0 and {height-1}, got {level}")
            
        self.height = height-level+1    # Not the heap height, but the layer height
        self.level = level
        self.level_I = Int(32)(level)
        self.name = f"level_{level}"
        self.elements = elements
        
        self.action = Port(Int(1))      # 0 means push, 1 means pop
        self.index = Port(Int(32))
        self.value = Port(Int(32))

    @module.combinational
    def build(self, next_layer: 'Layer' = None, next_elements: RegArray = None):        
        index_bit = max(self.level, 1)        
        index0 = self.index[0:index_bit-1].bitcast(Int(index_bit))

        # PUSH
        with Condition(~self.action):
            v = self.elements[index0]
            # The current element is valid.
            with Condition(~v[self.height:self.height]):
                vv = self.value.concat(Int(1)(1)).concat(v[0:self.height-1])
                self.elements[index0] = vv
                log("Push {}  \tin\tLevel_{}[{}]\tFrom  {} + {} + {}\tto  {} + {} + {}",
                    self.value, self.level_I, index0,
                    self.elements[index0][self.height+1:self.height+32],
                    self.elements[index0][self.height:self.height],
                    self.elements[index0][0:self.height-1], 
                    vv[self.height+1:self.height+32],
                    vv[self.height:self.height],
                    vv[0:self.height-1])
                
            # The current element is occupied.
            with Condition(v[self.height:self.height]):                                            
                # There is no vacancy on the subtree.
                with Condition(v[0:self.height-1] == UInt(self.height)(0)):
                    log("Push {}  \tPush failed, There is no vacancy!", self.value)
                    
                # There is vacancy on the subtree.
                with Condition(v[0:self.height-1] > UInt(self.height)(0)):
                    vacancy = v[0:self.height-1].bitcast(Int(self.height))-Int(self.height)(1)    
                    # value write to current level                
                    value_current = (self.value > v[self.height+1:self.height+32].bitcast(Int(32))).select(v[self.height+1:self.height+32].bitcast(Int(32)), self.value)
                    # value write to next level
                    value_next = (self.value > v[self.height+1:self.height+32].bitcast(Int(32))).select(self.value, v[self.height+1:self.height+32].bitcast(Int(32)))
                    vv = value_current.concat(Int(1)(1)).concat(vacancy)
                    self.elements[index0] = vv
                    log("Push {}  \tin\tLevel_{}[{}]\tFrom  {} + {} + {}\tto  {} + {} + {}",
                        self.value, self.level_I, index0,
                        self.elements[index0][self.height+1:self.height+32],
                        self.elements[index0][self.height:self.height],
                        self.elements[index0][0:self.height-1],
                        vv[self.height+1:self.height+32],
                        vv[self.height:self.height],
                        vv[0:self.height-1])
                    
                    # Call next layer
                    if next_elements:
                        # Two child nodes derived from the same parent node.
                        index1 = (index0 * Int(32)(2))[0:31].bitcast(Int(32))
                        index2 = index1 + Int(32)(1)
                        v1 = next_elements[index1]
                        v2 = next_elements[index2]
                        
                        # At least one child is valid
                        with Condition(~v1[self.height-1:self.height-1] | ~v2[self.height-1:self.height-1]):
                            valid_ab = (~v2[self.height-1:self.height-1]).concat(~v1[self.height-1:self.height-1]).bitcast(Int(2))
                            pred = ((~valid_ab) + Int(2)(1)) & valid_ab
                            index_next = pred.select1hot(index1, index2)
                            next_layer.async_called(action=Int(1)(0), index=index_next, value=value_next)

                        # Two child nodes are both occupied.
                        with Condition(v1[self.height-1:self.height-1] & v2[self.height-1:self.height-1]):
                            index_next = (v1[0:self.height-2] < v2[0:self.height-2]).select(index2, index1)
                            next_layer.async_called(action=Int(1)(0), index=index_next, value=value_next)

        # POP
        with Condition(self.action):
            v = self.elements[index0]         
            # The current element is valid.
            with Condition(~v[self.height:self.height]):
                log("Pop\t\tPop failed! The heap is empty.")
            # The current element is occupied.
            with Condition(v[self.height:self.height]):
                with Condition(self.level_I == Int(32)(0)):
                    log("Pop: {}", self.elements[index0][self.height+1:self.height+32])
                with Condition(self.level_I != Int(32)(0)):
                    log("Pop  {}  \tfrom\tLevel_{}[{}]\tFrom  {} + {} + {}",
                        self.elements[index0][self.height+1:self.height+32],
                        self.level_I, index0,
                        self.elements[index0][self.height+1:self.height+32],
                        self.elements[index0][self.height:self.height],
                        self.elements[index0][0:self.height-1])
                    
                # Call next layer                                        
                if next_elements is None:
                    vacancy = v[0:self.height-1].bitcast(Int(self.height))+Int(self.height)(1)
                    vv = Int(32)(0).concat(Int(1)(0)).concat(v[0:self.height-1])
                    self.elements[index0] = vv
                    
                if next_elements:
                    # Two child nodes derived from the same parent node.
                    index1 = (index0 * Int(32)(2))[0:31].bitcast(Int(32))
                    index2 = index1 + Int(32)(1)
                    v1 = next_elements[index1]
                    v2 = next_elements[index2]
                    
                    # Two child nodes are both occupied.
                    with Condition(v1[self.height-1:self.height-1] & v2[self.height-1:self.height-1]):
                        # value write to current level
                        value_update = (v1[self.height:self.height+31] < v2[self.height:self.height+31]).select(v1[self.height:self.height+31], v2[self.height:self.height+31])
                        index_next = (v1[self.height:self.height+31] < v2[self.height:self.height+31]).select(index1, index2)                        
                        vacancy = v[0:self.height-1].bitcast(Int(self.height))+Int(self.height)(1)
                        vv = value_update.concat(Int(1)(1)).concat(vacancy)
                        self.elements[index0] = vv

                        next_layer.async_called(action=Int(1)(1), index=index_next, value=Int(32)(0))
                        
                    # One child is valid, another is occupied.
                    with Condition(v1[self.height-1:self.height-1] ^ v2[self.height-1:self.height-1]):
                        occupied_ab = v2[self.height-1:self.height-1].concat(v1[self.height-1:self.height-1]).bitcast(Int(2))
                        pred = ((~occupied_ab) + Int(2)(1)) & occupied_ab
                        index_next = pred.select1hot(index1, index2)
                        value_update = pred.select1hot(v1[self.height:self.height+31], v2[self.height:self.height+31])
                        vacancy = v[0:self.height-1].bitcast(Int(self.height))+Int(self.height)(1)
                        vv = value_update.concat(Int(1)(1)).concat(vacancy)
                        self.elements[index0] = vv

                        next_layer.async_called(action=Int(1)(1), index=index_next, value=Int(32)(0))

                    # Two child nodes are both valid.
                    with Condition(~v1[self.height-1:self.height-1] & ~v2[self.height-1:self.height-1]):
                        vv = Int(32)(0).concat(Int(1)(0)).concat(v[0:self.height-1])
                        self.elements[index0] = vv

class Heap_Push(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)
        self.push_value = Port(Int(32))

    @module.combinational
    def build(self, layer: Layer):
        bound = layer.bind(action=Int(1)(0), index=Int(32)(0))
        bound.async_called(value=self.push_value)
        
class Heap_Pop(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)

    @module.combinational
    def build(self, layer: Layer):
        bound = layer.bind(action=Int(1)(1), index=Int(32)(0), value=Int(32)(1))
        bound.async_called()
        
        
class Testbench(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)

    @module.combinational
    def build(self, push: Heap_Push, pop: Heap_Pop):
        with Cycle(1):
            push.async_called(push_value=Int(32)(40))
            
        with Cycle(3):
            push.async_called(push_value=Int(32)(100))
            
        with Cycle(5):
            push.async_called(push_value=Int(32)(93))
            
        with Cycle(7):
            push.async_called(push_value=Int(32)(29))
            
        with Cycle(9):
            pop.async_called()
            
        with Cycle(11):
            push.async_called(push_value=Int(32)(7))
            
        with Cycle(13):
            push.async_called(push_value=Int(32)(87))
            
        with Cycle(15):
            pop.async_called()
            
        with Cycle(17):
            pop.async_called()
            
        with Cycle(19):
            pop.async_called()
            
        with Cycle(21):
            pop.async_called()
            
        with Cycle(23):
            pop.async_called()

def check(raw,heap_height):
    cnt = 0
    if heap_height == 1:
        pops = [40, 7]
    elif heap_height == 2:
        pops = [40, 7, 93, 100]
    else:
        pops = [29, 7, 40, 87, 93, 100]
        
    outputs = []
    for i in raw.split('\n'):
        if f'Pop:' in i:
            line_toks = i.split()
            value = line_toks[-1]
            outputs.append(int(value))
            cnt += 1    
    for i in range(len(pops)):
        assert pops[i] == outputs[i] 
    assert cnt == len(pops), f'cnt: {cnt} != {len(pops)}'

def priority_queue(heap_height=3):    
    # Build a layer with the given heap height and layer level.
    def build_layer(heap_height: int, level: int):
        element_type = Bits(32 + 1 + (heap_height-level+1))  # value + occupied + vacancy
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
        
        heap_push = Heap_Push()
        heap_push.build(layers[0])
        
        heap_pop = Heap_Pop()
        heap_pop.build(layers[0])
        
        testbench = Testbench()
        testbench.build(heap_push, heap_pop)
    
    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())
    
    raw = utils.run_simulator(simulator_path)
    check(raw, heap_height=heap_height)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw, heap_height=heap_height)
    
    
if __name__ == '__main__':
    priority_queue(heap_height=2)
