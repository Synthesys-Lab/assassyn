'''FSM (Finite State Machine) syntax sugar.'''
import math
from ..block import Condition
from ..dtype import Bits
from ..array import Array

class FSM:# pylint: disable=R0903
    '''FSM inside the module.'''
    state_reg: Array  # State register to store the current state
    transition_table: dict  # Dictionary mapping state transitions
    state_bits: int  # Number of bits needed to represent all states
    state_map: dict  # Dictionary mapping state names to bit representations

    def __init__(self, state_reg, transition_table):
        '''FSM constructor.'''
        assert isinstance(state_reg, Array), "Expecting a Array object"
        self.state_reg = state_reg

        self.transition_table = transition_table

        self.state_bits = math.floor(math.log2(len(transition_table)))
        print(f"State bits: {self.state_bits}")


        self.state_map = {}
        i = 0
        for state_name in transition_table:
            self.state_map[state_name] = Bits(self.state_bits)(i)
            i += 1



    def generate(self,func_dict,mux_dict=None):
        '''Build FSM.'''
        state_reg = self.state_reg
        for state_name in self.transition_table:

            print(f"State: {state_name}")
            with Condition( state_reg[0] == self.state_map[state_name]):
                if state_name in func_dict:
                    func_dict[state_name]()
                for condition, next_state in self.transition_table[state_name].items():
                    print(f"Condition: {condition}, Next state: {next_state}")
                    with Condition(condition):
                        state_reg[0] = self.state_map[next_state]
        if mux_dict is not None:
            for value in mux_dict:
                for state_name,right_v in mux_dict[value].items():
                    value = ( state_reg[0] == self.state_map[state_name]).select(right_v, value)
