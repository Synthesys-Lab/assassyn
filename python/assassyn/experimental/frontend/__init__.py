"""New experimental frontend for Assassyn.

This frontend provides a more function-like programming style as a wrapper
to the old frontend. It uses the @pipeline.factory decorator to create
pipeline stages with automatic AST building.

Key components:
- pipeline: Module containing the factory decorator and utilities
- Stage: Wrapper class for Module objects with convenient calling interface
"""

from . import pipeline
from .stage import Stage

__all__ = ['pipeline', 'Stage']
