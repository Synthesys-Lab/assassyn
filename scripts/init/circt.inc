# CIRCT installation targets
# Converted from scripts/init/circt.sh

.PHONY: install-circt clean-circt patch-circt

# Patch target for circt
patch-circt: 3rd-party/circt/.patch-applied

# Marker file to track patch application
3rd-party/circt/.patch-applied:
	@echo "Checking circt patch status..."
	@cd 3rd-party/circt && \
		if bash ../../scripts/patch-apply.sh check ../../scripts/init/patches/circt.patch 2>/dev/null; then \
			echo "CIRCT patch already applied — skipping patch step."; \
		else \
			echo "Applying CIRCT patch..."; \
			bash ../../scripts/patch-apply.sh apply ../../scripts/init/patches/circt.patch; \
		fi && \
		touch $(CURDIR)/3rd-party/circt/.patch-applied

install-circt: 3rd-party/circt/.circt-built

# Marker file to track build completion
3rd-party/circt/.circt-built: patch-circt
	@if [ -f "$@" ]; then \
		echo "CIRCT already built — skipping build step."; \
	else \
		echo "Installing PyCDE..."; \
		echo "Step 1: Checking if PyCDE is already installed..."; \
		if . .assassyn-venv/bin/activate && python -c "import pycde" 2>/dev/null; then \
			echo "PyCDE is already installed — skipping installation."; \
		else \
			echo "PyCDE not found. Proceeding with installation..."; \
			echo "Step 2: Attempting to install PyCDE via pip..."; \
			if . .assassyn-venv/bin/activate && pip install pycde; then \
				echo "PyCDE installed successfully via pip."; \
			else \
				echo "Failed to install PyCDE via pip. Proceeding to source installation..."; \
				echo "Step 3: Building PyCDE from source..."; \
				cd 3rd-party/circt/frontends/PyCDE && \
				CIRCT_DIRECTORY="$$(pwd)/../../" CIRCT_EXTRA_CMAKE_ARGS="-DESI_RUNTIME=OFF -DZ3_DISABLE=ON -DOR_TOOLS_DISABLE=ON" . $$(pwd)/../../../../.assassyn-venv/bin/activate && python -m build && \
				echo "Installing PyCDE wheel..."; \
				if . $$(pwd)/../../../../.assassyn-venv/bin/activate && pip install ./dist/*.whl; then \
					echo "PyCDE built and installed successfully from source."; \
				else \
					echo "Failed to install PyCDE from source. Please check the build logs."; \
					exit 1; \
				fi; \
			fi; \
		fi; \
		echo "CIRCT installation completed."; \
		touch $(CURDIR)/$@; \
	fi

clean-circt:
	@echo "Cleaning CIRCT build artifacts and uninstalling PyCDE..."
	@if [ -d ".assassyn-venv" ]; then \
		. .assassyn-venv/bin/activate && pip uninstall -y pycde 2>/dev/null || true; \
	fi
	@cd 3rd-party/circt && \
		rm -rf dist build *.egg-info 2>/dev/null || true && \
		bash ../../scripts/patch-apply.sh reverse ../../scripts/init/patches/circt.patch 2>/dev/null || true && \
		rm -f .patch-applied .circt-built
	@echo "CIRCT clean completed."
