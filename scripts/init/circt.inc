# CIRCT installation targets
# Converted from scripts/init/circt.sh

.PHONY: install-circt clean-circt patch-circt

# Patch target for circt
patch-circt: 3rd-party/circt/.patch-applied

# Marker file to track patch application
3rd-party/circt/.patch-applied:
	@echo "Checking circt patch status..."
	@cd 3rd-party/circt && \
		if git apply --reverse --check ../../scripts/init/patches/circt.patch 2>/dev/null; then \
			echo "CIRCT patch already applied — skipping patch step."; \
		else \
			echo "Applying CIRCT patch..."; \
			git apply ../../scripts/init/patches/circt.patch 2>/dev/null; \
		fi && \
		touch .patch-applied

install-circt: patch-circt
	@echo "Installing PyCDE..."
	@echo "Step 1: Checking if PyCDE is already installed..."
	@if python -c "import pycde" 2>/dev/null; then \
		echo "PyCDE is already installed — skipping installation."; \
	else \
		echo "PyCDE not found. Proceeding with installation..."; \
		echo "Step 2: Attempting to install PyCDE via pip..."; \
		if pip install pycde; then \
			echo "PyCDE installed successfully via pip."; \
		else \
			echo "Failed to install PyCDE via pip. Proceeding to source installation..."; \
			echo "Step 3: Building PyCDE from source..."; \
			cd 3rd-party/circt/frontends/PyCDE && \
			CIRCT_DIRECTORY="$$(pwd)/../../" CIRCT_EXTRA_CMAKE_ARGS="-DESI_RUNTIME=OFF -DZ3_DISABLE=ON -DOR_TOOLS_DISABLE=ON" python -m build && \
			echo "Installing PyCDE wheel..."; \
			if pip install ./dist/*.whl; then \
				echo "PyCDE built and installed successfully from source."; \
			else \
				echo "Failed to install PyCDE from source. Please check the build logs."; \
				exit 1; \
			fi; \
		fi; \
	fi
	@echo "CIRCT installation completed."

clean-circt:
	@echo "Cleaning CIRCT build artifacts and uninstalling PyCDE..."
	@pip uninstall -y pycde 2>/dev/null || true
	@cd 3rd-party/circt && \
		rm -rf dist build *.egg-info 2>/dev/null || true && \
		git apply --reverse ../../scripts/init/patches/circt.patch 2>/dev/null || true && \
		rm -f .patch-applied
	@echo "CIRCT clean completed."
