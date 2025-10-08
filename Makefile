# Master Makefile for Assassyn project
# This Makefile provides a unified interface for building, testing, and cleaning the project

.PHONY: all env env-source build-all test-all clean-all install-py-package build-verilator clean-verilator build-ramulator2 build-wrapper clean-ramulator2 clean-wrapper install-circt clean-circt rust-lint pylint

# Default target
all: build-all test-all

# Environment setup target
env:
	@echo "To apply environment variables to your shell, run:"
	@echo "eval \$$(make env-source)"
	@echo ""
	@echo "Or manually run:"
	@echo "source setup.sh"

# Environment source target - outputs the actual command
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

# Rust linting targets
rust-lint:
	@echo "Running Rust formatting check..."
	cargo fmt --manifest-path tools/rust-sim-runtime/Cargo.toml --all -- --check --config-path rustfmt.toml
	@echo "Running Rust clippy check..."
	cargo clippy --manifest-path tools/rust-sim-runtime/Cargo.toml --all-targets --all-features -- -D warnings

# Python linting target
pylint:
	@echo "Running pylint on assassyn package..."
	pylint --rcfile=python/assassyn/.pylintrc python/assassyn/

# Include component-specific Makefiles
include scripts/init/py-package.inc
include scripts/init/verilator.inc
include scripts/init/wrapper.inc
include scripts/init/circt.inc
