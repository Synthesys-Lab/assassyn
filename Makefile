# Master Makefile for Assassyn project
# This Makefile provides a unified interface for building, testing, and cleaning the project

.PHONY: all env env-source build-all test-all clean-all install-py-package build-verilator clean-verilator build-ramulator2 build-wrapper clean-ramulator2 clean-wrapper install-circt clean-circt

# Default target
all: build-all test-all

# Environment setup target
env:
	@echo "Setting up environment..."
	@echo "Run the following command to set up your shell environment:"
	@echo "source setup.sh"
	@echo ""
	@echo "Or run this single command:"
	@echo "eval \$$(make env-source)"

# Environment source target - outputs commands to set up environment
env-source:
	@echo "source setup.sh"

# Build all components
build-all: install-py-package build-verilator build-ramulator2 build-wrapper install-circt

# Test all components
test-all: build-all
	@echo "Running all tests..."
	@pytest -n 8 -x python/unit-tests
	@pytest -n 8 -x python/ci-tests

# Clean all components
clean-all: clean-verilator clean-ramulator2 clean-wrapper clean-circt

# Include component-specific Makefiles
include scripts/init/py-package.inc
include scripts/init/verilator.inc
include scripts/init/wrapper.inc
include scripts/init/circt.inc
