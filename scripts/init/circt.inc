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
	@pip install pycde || \
		(echo "Failed to install CIRCT via pip. Fall back to building from source using PyCDE setup." && \
		cd 3rd-party/circt/frontends/PyCDE && \
		CIRCT_DIRECTORY="$$(pwd)/../../" CIRCT_EXTRA_CMAKE_ARGS="-DESI_RUNTIME=OFF -DZ3_DISABLE=ON -DOR_TOOLS_DISABLE=ON" python -m build && \
		pip install ./dist/*.whl && \
		echo "PyCDE built and installed successfully to local directory.")
	@echo "CIRCT installation completed."

clean-circt:
	@echo "Cleaning CIRCT build artifacts and uninstalling PyCDE..."
	@pip uninstall -y pycde 2>/dev/null || true
	@cd 3rd-party/circt && \
		rm -rf dist build *.egg-info 2>/dev/null || true && \
		git apply --reverse ../../scripts/init/patches/circt.patch 2>/dev/null || true && \
		rm -f .patch-applied
	@echo "CIRCT clean completed."
