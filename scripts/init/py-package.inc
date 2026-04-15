# Python package installation targets
# Converted from scripts/init/py-package.sh

PYTHON_VENV_BIN ?= $(shell command -v python3.12 2>/dev/null || command -v python3 2>/dev/null || command -v python 2>/dev/null)

.PHONY: install-py-package clean-python

install-py-package:
	@echo "Setting up Python virtual environment..."
	@if [ -z "$(PYTHON_VENV_BIN)" ]; then \
		echo "No suitable Python interpreter found. Tried python3.12, python3, and python." >&2; \
		exit 1; \
	fi
	@if [ ! -d "$(VENV_DIR)" ]; then \
		echo "Creating virtual environment with $(PYTHON_VENV_BIN)..."; \
		"$(PYTHON_VENV_BIN)" -m venv $(VENV_DIR); \
	else \
		echo "Virtual environment already exists."; \
	fi
	@echo "Installing dependencies using virtual environment Python..."
	@$(VENV_DIR)/bin/python -m pip install --upgrade pip
	@$(VENV_DIR)/bin/python -m pip install build
	@$(VENV_DIR)/bin/python -m pip install -r python/requirements.txt
	@echo "Python dependencies installed successfully in virtual environment."

clean-python:
	@echo "Cleaning Python virtual environment..."
	@rm -rf $(VENV_DIR)
	@echo "Python virtual environment cleaned successfully."
