# Python package installation targets
# Converted from scripts/init/py-package.sh

.PHONY: install-py-package clean-python

install-py-package:
	@echo "Setting up Python virtual environment..."
	@if [ ! -d "$(VENV_DIR)" ]; then \
		echo "Creating virtual environment..."; \
		python -m venv $(VENV_DIR); \
	else \
		echo "Virtual environment already exists."; \
	fi
	@echo "Installing dependencies using virtual environment Python..."
	@. $(VENV_DIR)/bin/activate && pip install --upgrade pip
	@. $(VENV_DIR)/bin/activate && pip install build 
	@. $(VENV_DIR)/bin/activate && pip install -r python/requirements.txt
	@echo "Python dependencies installed successfully in virtual environment."

clean-python:
	@echo "Cleaning Python virtual environment..."
	@rm -rf $(VENV_DIR)
	@echo "Python virtual environment cleaned successfully."
