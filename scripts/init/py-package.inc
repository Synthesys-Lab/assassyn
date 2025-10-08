# Python package installation targets
# Converted from scripts/init/py-package.sh

.PHONY: install-py-package

install-py-package:
	@echo "Installing Python dependencies..."
	@pip install --user -r python/requirements.txt --break-system-packages
	@if [ $$? -ne 0 ]; then \
		echo "Failed to install Python dependencies. Please check the requirements."; \
		exit 1; \
	else \
		echo "Python dependencies installed successfully."; \
	fi
