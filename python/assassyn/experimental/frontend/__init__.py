"""New experimental frontend for Assassyn.

This frontend provides a more function-like programming style as a wrapper
to the old frontend. It uses the @pipeline.factory decorator to create
pipeline stages with automatic AST building.

Key components:
- pipeline: Module containing the factory decorator and utilities
- Stage: Wrapper class for Module objects with convenient calling interface
- if_: Wrapper to Condition for conditional blocks
"""

from assassyn.ir.block import Condition
from . import pipeline
from .stage import Stage
from .converge import converge

# Wrapper to Condition for better readability
if_ = Condition

__all__ = ['pipeline', 'Stage', 'if_', 'converge']
