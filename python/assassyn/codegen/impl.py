'''The module to generate the assassyn IR builder for the given system'''

from . import simulator
from . import verilog
from ..builder import SysBuilder

def codegen(sys: SysBuilder, **kwargs):
    '''
    The help function to generate the assassyn IR builder for the given system

    Args:
        sys (SysBuilder): The system to generate the builder for
        simulator: Whether to generate a simulator
        verilog: Verilog simulator target (if any)
        idle_threshold: Idle threshold for the simulator
        sim_threshold: Simulation threshold
        random: Whether to randomize module execution order
        resource_base: Path to resource files
        fifo_depth: Default FIFO depth
    '''
    # Create a CodeGen object but exclude simulator generation flag
    # We'll handle simulator generation separately using the Python implementation

    simulator_manifest = None
    # If simulator flag is set, use the Python implementation to generate it
    if kwargs['simulator']:
        print('Start simulator in-python elaboration')
        simulator_manifest = simulator.elaborate(sys, **kwargs)

    verilog_path = None
    if kwargs.get('verilog'):
        print('Start verilog elaboration')
        verilog_path = verilog.elaborate(sys, **kwargs)

    return simulator_manifest, verilog_path
