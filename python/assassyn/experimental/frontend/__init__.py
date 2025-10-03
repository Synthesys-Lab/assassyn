"""New experimental frontend for Assassyn.

This frontend provides a more function-like programming style as a wrapper
to the old frontend. It uses the @factory decorator to create modules with
automatic AST building.

Key components:
- factory: Universal factory decorator for creating modules
- Factory: Generic wrapper class for module instances
- Module: Standard sequential module type
- Downstream: Combinational convergent module type
- if_: Wrapper to Condition for conditional blocks
- module: Module-specific utilities (pop_all)
- pin: Function to expose combinational pins
"""

from assassyn.ir.block import Condition
from assassyn.ir.module import Module
from assassyn.ir.module.downstream import Downstream
from assassyn.ir.value import Value

from .factory import Factory, factory, pin, this
from . import module
from . import downstream

# Wrapper to Condition for better readability
if_ = Condition

__all__ = [
    'factory',
    'Factory',
    'Module',
    'Downstream',
    'Value',
    'module',
    'pin',
    'this',
    'if_',
]
