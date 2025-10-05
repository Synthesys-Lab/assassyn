"""
Unit tests for UniqueNameCache.
"""

import pytest
from assassyn.builder.unique_name import UniqueNameCache


class TestUniqueNameCache:
    """Test cases for UniqueNameCache."""

    def test_init(self):
        """Test that UniqueNameCache can be initialized."""
        cache = UniqueNameCache()
        assert cache is not None

    def test_first_name_returns_prefix(self):
        """Test that the first unique name with a prefix returns the prefix itself."""
        cache = UniqueNameCache()
        result = cache.get_unique_name("signal")
        assert result == "signal"

    def test_second_same_prefix_appends_number(self):
        """Test that requesting the same prefix twice appends a number."""
        cache = UniqueNameCache()
        first = cache.get_unique_name("signal")
        second = cache.get_unique_name("signal")
        assert first == "signal"
        assert second == "signal_1"

    def test_multiple_same_prefix_increments_number(self):
        """Test that multiple requests with same prefix increment the number."""
        cache = UniqueNameCache()
        names = [cache.get_unique_name("wire") for _ in range(5)]
        assert names == ["wire", "wire_1", "wire_2", "wire_3", "wire_4"]

    def test_different_prefixes_independent(self):
        """Test that different prefixes maintain independent counters."""
        cache = UniqueNameCache()
        signal1 = cache.get_unique_name("signal")
        wire1 = cache.get_unique_name("wire")
        signal2 = cache.get_unique_name("signal")
        wire2 = cache.get_unique_name("wire")

        assert signal1 == "signal"
        assert wire1 == "wire"
        assert signal2 == "signal_1"
        assert wire2 == "wire_1"

    def test_empty_prefix(self):
        """Test behavior with empty string prefix."""
        cache = UniqueNameCache()
        first = cache.get_unique_name("")
        second = cache.get_unique_name("")
        assert first == ""
        assert second == "_1"

    def test_prefix_with_numbers(self):
        """Test that prefixes containing numbers work correctly."""
        cache = UniqueNameCache()
        first = cache.get_unique_name("signal_0")
        second = cache.get_unique_name("signal_0")
        assert first == "signal_0"
        assert second == "signal_0_1"

    def test_prefix_with_underscores(self):
        """Test that prefixes ending with underscores work correctly."""
        cache = UniqueNameCache()
        first = cache.get_unique_name("my_signal_")
        second = cache.get_unique_name("my_signal_")
        assert first == "my_signal_"
        assert second == "my_signal__1"

    def test_cache_isolation(self):
        """Test that different cache instances are independent."""
        cache1 = UniqueNameCache()
        cache2 = UniqueNameCache()

        name1_cache1 = cache1.get_unique_name("test")
        name1_cache2 = cache2.get_unique_name("test")
        name2_cache1 = cache1.get_unique_name("test")

        assert name1_cache1 == "test"
        assert name1_cache2 == "test"
        assert name2_cache1 == "test_1"

    def test_large_number_of_names(self):
        """Test that the cache handles a large number of unique names."""
        cache = UniqueNameCache()
        names = [cache.get_unique_name("node") for _ in range(100)]

        # Check first and last
        assert names[0] == "node"
        assert names[99] == "node_99"

        # Check all are unique
        assert len(set(names)) == 100

    def test_special_characters_in_prefix(self):
        """Test prefixes with special characters."""
        cache = UniqueNameCache()
        first = cache.get_unique_name("signal$")
        second = cache.get_unique_name("signal$")
        assert first == "signal$"
        assert second == "signal$_1"

    def test_unicode_prefix(self):
        """Test that unicode prefixes work correctly."""
        cache = UniqueNameCache()
        first = cache.get_unique_name("信号")
        second = cache.get_unique_name("信号")
        assert first == "信号"
        assert second == "信号_1"

    def test_mixed_usage_pattern(self):
        """Test a realistic mixed usage pattern."""
        cache = UniqueNameCache()

        # Mix different prefixes
        names = []
        names.append(cache.get_unique_name("clk"))
        names.append(cache.get_unique_name("data"))
        names.append(cache.get_unique_name("clk"))
        names.append(cache.get_unique_name("addr"))
        names.append(cache.get_unique_name("data"))
        names.append(cache.get_unique_name("clk"))

        assert names == ["clk", "data", "clk_1", "addr", "data_1", "clk_2"]
