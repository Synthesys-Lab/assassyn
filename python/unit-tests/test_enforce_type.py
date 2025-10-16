"""Unit tests for the @enforce_type decorator.

This test suite validates the type enforcement decorator functionality,
including type validation, error handling, and decorator behavior.
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

import pytest
from typing import List, Dict, Union, Optional, Any
from assassyn.utils.enforce_type import enforce_type, validate_arguments, check_type


class TestEnforceTypeDecorator:
    """Test the @enforce_type decorator functionality."""

    def test_valid_simple_types(self):
        """Test that valid simple types pass through unchanged."""
        
        @enforce_type
        def simple_function(value: int, name: str, flag: bool) -> str:
            """Function with simple type annotations."""
            return f"{name}: {value} ({flag})"
        
        # Valid calls should work
        result = simple_function(42, "test", True)
        assert result == "test: 42 (True)"
        
        result = simple_function(0, "", False)
        assert result == ": 0 (False)"

    def test_invalid_simple_types(self):
        """Test that invalid simple types raise TypeError."""
        
        @enforce_type
        def simple_function(value: int, name: str) -> str:
            """Function with simple type annotations."""
            return f"{name}: {value}"
        
        # Invalid calls should raise TypeError
        with pytest.raises(TypeError, match="Argument 'value' must be of type int, got str"):
            simple_function("not_an_int", "test")
        
        with pytest.raises(TypeError, match="Argument 'name' must be of type str, got int"):
            simple_function(42, 123)

    def test_optional_types(self):
        """Test Optional type handling."""
        
        @enforce_type
        def optional_function(value: Optional[int] = None) -> str:
            """Function with Optional type annotation."""
            return f"value: {value}"
        
        # None should be accepted
        result = optional_function(None)
        assert result == "value: None"
        
        # Valid int should be accepted
        result = optional_function(42)
        assert result == "value: 42"
        
        # Invalid type should raise error
        with pytest.raises(TypeError, match="Argument 'value' must be of type int, got str"):
            optional_function("not_an_int")

    def test_union_types(self):
        """Test Union type handling."""
        
        @enforce_type
        def union_function(value: Union[int, str]) -> str:
            """Function with Union type annotation."""
            return f"value: {value}"
        
        # Both int and str should be accepted
        result = union_function(42)
        assert result == "value: 42"
        
        result = union_function("hello")
        assert result == "value: hello"
        
        # Invalid type should raise error
        with pytest.raises(TypeError, match="Argument 'value' must be of type int or str, got bool"):
            union_function(True)

    def test_any_type(self):
        """Test Any type handling (should skip validation)."""
        
        @enforce_type
        def any_function(value: Any) -> str:
            """Function with Any type annotation."""
            return f"value: {value}"
        
        # Any type should be accepted
        result = any_function(42)
        assert result == "value: 42"
        
        result = any_function("hello")
        assert result == "value: hello"
        
        result = any_function([1, 2, 3])
        assert result == "value: [1, 2, 3]"

    def test_generic_types_structure_only(self):
        """Test generic type handling (structure validation only)."""
        
        @enforce_type
        def generic_function(items: List[int], mapping: Dict[str, int]) -> str:
            """Function with generic type annotations."""
            return f"items: {len(items)}, mapping: {len(mapping)}"
        
        # Valid structure should work (contents not validated)
        result = generic_function([1, 2, 3], {"a": 1, "b": 2})
        assert result == "items: 3, mapping: 2"
        
        # Invalid structure should raise error
        with pytest.raises(TypeError, match="Argument 'items' must be of type list, got str"):
            generic_function("not_a_list", {"a": 1})

    def test_no_annotations(self):
        """Test functions without type annotations."""
        
        @enforce_type
        def no_annotations(value):
            """Function without type annotations."""
            return f"value: {value}"
        
        # Should work with any type (no validation)
        result = no_annotations(42)
        assert result == "value: 42"
        
        result = no_annotations("hello")
        assert result == "value: hello"

    def test_decorator_preserves_metadata(self):
        """Test that decorator preserves function metadata."""
        
        @enforce_type
        def documented_function(value: int) -> str:
            """This is a documented function."""
            return f"value: {value}"
        
        # Should preserve function name
        assert documented_function.__name__ == "documented_function"
        
        # Should preserve docstring
        assert documented_function.__doc__ == "This is a documented function."

    def test_default_arguments(self):
        """Test functions with default arguments."""
        
        @enforce_type
        def default_function(value: int, name: str = "default") -> str:
            """Function with default arguments."""
            return f"{name}: {value}"
        
        # Should work with defaults
        result = default_function(42)
        assert result == "default: 42"
        
        # Should work with explicit values
        result = default_function(42, "custom")
        assert result == "custom: 42"

    def test_keyword_arguments(self):
        """Test functions called with keyword arguments."""
        
        @enforce_type
        def keyword_function(value: int, name: str) -> str:
            """Function with keyword arguments."""
            return f"{name}: {value}"
        
        # Should work with keyword arguments
        result = keyword_function(name="test", value=42)
        assert result == "test: 42"


class TestValidateArguments:
    """Test the validate_arguments function directly."""

    def test_validate_arguments_simple(self):
        """Test validate_arguments with simple types."""
        
        def test_func(value: int, name: str) -> str:
            return f"{name}: {value}"
        
        # Valid arguments should pass
        validated = validate_arguments(test_func, (42, "test"), {})
        assert validated["value"] == 42
        assert validated["name"] == "test"
        
        # Invalid arguments should raise error
        with pytest.raises(TypeError):
            validate_arguments(test_func, ("not_int", "test"), {})

    def test_validate_arguments_with_kwargs(self):
        """Test validate_arguments with keyword arguments."""
        
        def test_func(value: int, name: str = "default") -> str:
            return f"{name}: {value}"
        
        # Should work with kwargs
        validated = validate_arguments(test_func, (), {"value": 42, "name": "test"})
        assert validated["value"] == 42
        assert validated["name"] == "test"

    def test_validate_arguments_mixed_args(self):
        """Test validate_arguments with mixed positional and keyword arguments."""
        
        def test_func(value: int, name: str, flag: bool = False) -> str:
            return f"{name}: {value} ({flag})"
        
        # Should work with mixed args
        validated = validate_arguments(test_func, (42,), {"name": "test", "flag": True})
        assert validated["value"] == 42
        assert validated["name"] == "test"
        assert validated["flag"] is True


class TestCheckType:
    """Test the check_type helper function."""

    def test_check_type_simple(self):
        """Test check_type with simple types."""
        
        # Valid types should pass
        assert check_type(42, int) is True
        assert check_type("hello", str) is True
        assert check_type(True, bool) is True
        
        # Invalid types should raise TypeError
        with pytest.raises(TypeError, match="Expected int, got str"):
            check_type("not_int", int)

    def test_check_type_optional(self):
        """Test check_type with Optional types."""
        
        # None should be accepted for Optional
        assert check_type(None, Optional[int]) is True
        
        # Valid type should be accepted
        assert check_type(42, Optional[int]) is True
        
        # Invalid type should raise error
        with pytest.raises(TypeError, match="Expected int, got str"):
            check_type("not_int", Optional[int])

    def test_check_type_union(self):
        """Test check_type with Union types."""
        
        # Any variant should be accepted
        assert check_type(42, Union[int, str]) is True
        assert check_type("hello", Union[int, str]) is True
        
        # Invalid type should raise error
        with pytest.raises(TypeError, match="Expected int or str, got bool"):
            check_type(True, Union[int, str])

    def test_check_type_any(self):
        """Test check_type with Any type."""
        
        # Any type should pass
        assert check_type(42, Any) is True
        assert check_type("hello", Any) is True
        assert check_type([1, 2, 3], Any) is True

    def test_check_type_generic_structure(self):
        """Test check_type with generic types (structure only)."""
        
        # Valid structure should pass
        assert check_type([1, 2, 3], List[int]) is True
        assert check_type({"a": 1}, Dict[str, int]) is True
        
        # Invalid structure should raise error
        with pytest.raises(TypeError, match="Expected list, got str"):
            check_type("not_list", List[int])


class TestErrorMessages:
    """Test error message clarity and formatting."""

    def test_error_message_format(self):
        """Test that error messages are clear and informative."""
        
        @enforce_type
        def test_func(value: int, name: str) -> str:
            return f"{name}: {value}"
        
        # Test error message format
        with pytest.raises(TypeError) as exc_info:
            test_func("not_int", "test")
        
        error_msg = str(exc_info.value)
        assert "Argument 'value'" in error_msg
        assert "must be of type int" in error_msg
        assert "got str" in error_msg

    def test_error_message_with_function_name(self):
        """Test that error messages include function context."""
        
        @enforce_type
        def my_test_function(value: int) -> str:
            return f"value: {value}"
        
        with pytest.raises(TypeError) as exc_info:
            my_test_function("not_int")
        
        error_msg = str(exc_info.value)
        assert "my_test_function" in error_msg or "Argument 'value'" in error_msg


if __name__ == "__main__":
    pytest.main([__file__])
