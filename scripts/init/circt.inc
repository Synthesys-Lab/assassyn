# CIRCT installation targets
# Converted from scripts/init/circt.sh

.PHONY: install-circt clean-circt

install-circt:
	@echo "Installing PyCDE..."
	@pip install --user pycde --break-system-packages || \
		(echo "Failed to install CIRCT via pip. Fall back to building from source using PyCDE setup." && \
		cd 3rd-party/circt/frontends/PyCDE && \
		CIRCT_DIRECTORY="$$(pwd)/../../" CIRCT_EXTRA_CMAKE_ARGS="-DESI_RUNTIME=OFF -DZ3_DISABLE=ON -DOR_TOOLS_DISABLE=ON" python -m build && \
		pip install ./dist/*.whl && \
		echo "PyCDE built and installed successfully to local directory.")
	@echo "CIRCT installation completed."

clean-circt:
	@echo "Cleaning CIRCT build artifacts and uninstalling PyCDE..."
	@pip uninstall -y pycde 2>/dev/null || true
	@cd 3rd-party/circt/frontends/PyCDE && \
		rm -rf dist build *.egg-info 2>/dev/null || true
	@echo "CIRCT clean completed."
