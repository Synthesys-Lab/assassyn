# Python package installation targets
# Converted from scripts/init/py-package.sh

.PHONY: install-py-package

install-py-package:
	@echo "Installing Python dependencies..."
	@pip install -r python/requirements.txt
	@echo "Python dependencies installed successfully."
