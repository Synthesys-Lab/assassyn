# CIRCT installation targets
# Converted from scripts/init/circt.sh

.PHONY: install-circt clean-circt

install-circt:
	@echo "Installing PyCDE..."
	@pip install --user pycde --break-system-packages
	@echo "CIRCT installed successfully via pip."

clean-circt:
	@echo "Cleaning CIRCT build artifacts..."
	@cd 3rd-party/circt/frontends/PyCDE && \
		rm -rf dist build *.egg-info 2>/dev/null || true
	@echo "CIRCT clean completed."
