# Master Makefile for Assassyn project
# This Makefile provides a unified interface for building, testing, and cleaning the project

.PHONY: all env env-source build-all test-all clean-all clean-built install-py-package clean-python build-verilator clean-verilator build-ramulator2 build-wrapper clean-ramulator2 clean-wrapper install-circt clean-circt rust-lint pylint build-apptainer-base build-apptainer-repo build-apptainer clean-apptainer-base clean-apptainer-repo clean-apptainer patch-all patch-ramulator2 patch-circt patch-verilator

# Virtual environment directory (shared across all Python-related targets)
VENV_DIR := .assassyn-venv

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

# Apply all patches on physical machine before VM build
patch-all: patch-ramulator2 patch-circt patch-verilator

# Build all components
build-all: install-py-package build-verilator build-ramulator2 build-wrapper install-circt

# Test all components
test-all:
	@echo "Running all tests..."
	@pytest -n 8 python/unit-tests
	@pytest -n 8 python/ci-tests

# Clean all components
clean-all: clean-python clean-verilator clean-ramulator2 clean-wrapper clean-circt

# Clean all build marker files (.xxx-built)
clean-built:
	@echo "Cleaning all build marker files..."
	@rm -f 3rd-party/verilator/.verilator-built
	@rm -f 3rd-party/ramulator2/.ramulator2-built
	@rm -f tools/c-ramulator2-wrapper/.wrapper-built
	@rm -f 3rd-party/circt/.circt-built
	@echo "All build marker files cleaned."

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
include scripts/init/apptainer.inc
include scripts/codex.inc
