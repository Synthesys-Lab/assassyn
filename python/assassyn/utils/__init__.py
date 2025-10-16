"""Utility functions and decorators for Assassyn."""

# Import new type enforcement utilities
from .enforce_type import enforce_type, validate_arguments, check_type

# Import existing utilities from the main utils.py file
# We use a different approach to avoid circular imports
import importlib.util
import os

# Load the main utils.py file
utils_path = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'utils.py')
spec = importlib.util.spec_from_file_location("utils", utils_path)
utils_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(utils_module)

# Export all the functions
identifierize = utils_module.identifierize
unwrap_operand = utils_module.unwrap_operand
repo_path = utils_module.repo_path
package_path = utils_module.package_path
patch_fifo = utils_module.patch_fifo
run_simulator = utils_module.run_simulator
run_verilator = utils_module.run_verilator
parse_verilator_cycle = utils_module.parse_verilator_cycle
parse_simulator_cycle = utils_module.parse_simulator_cycle
has_verilator = utils_module.has_verilator
create_dir = utils_module.create_dir
namify = utils_module.namify

__all__ = [
    # Existing utilities
    'identifierize', 'unwrap_operand', 'repo_path', 'package_path',
    'patch_fifo', 'run_simulator', 'run_verilator', 'parse_verilator_cycle',
    'parse_simulator_cycle', 'has_verilator', 'create_dir', 'namify',
    # New type enforcement utilities
    'enforce_type', 'validate_arguments', 'check_type'
]
