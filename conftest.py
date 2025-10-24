"""
Pytest configuration for Assassyn tests.
"""

import pytest
from assassyn.builder import Singleton


@pytest.fixture(autouse=True)
def reset_builder_singleton():
    """Reset builder singleton before each test to ensure clean state."""
    # Reset before test
    Singleton.builder = None
    Singleton.line_expression_tracker = None
    Singleton.naming_manager = None
    
    yield
    
    # Reset after test
    Singleton.builder = None
    Singleton.line_expression_tracker = None
    Singleton.naming_manager = None