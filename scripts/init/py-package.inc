# Python package installation targets
# Converted from scripts/init/py-package.sh

.PHONY: install-py-package

install-py-package:
	@echo "Installing Python dependencies..."
	@pip install --user -r python/requirements.txt --break-system-packages
	@echo "Python dependencies installed successfully."
