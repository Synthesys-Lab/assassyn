# Verilator build targets
# Converted from scripts/init/verilator.sh

.PHONY: build-verilator clean-verilator patch-verilator

# Patch target for verilator
patch-verilator: 3rd-party/verilator/.patch-applied

# Marker file to track patch application
3rd-party/verilator/.patch-applied:
	@echo "Checking verilator patch status..."
	@cd 3rd-party/verilator && \
		if git apply --reverse --check ../../scripts/init/patches/verilator.patch 2>/dev/null; then \
			echo "Verilator patch already applied — skipping patch step."; \
		else \
			echo "Applying Verilator patch..."; \
			git apply ../../scripts/init/patches/verilator.patch 2>/dev/null; \
		fi && \
		touch .patch-applied

build-verilator: patch-verilator
	@echo "Building Verilator..."
	@echo "Installing Verilator by building it from source..."
	@cd 3rd-party/verilator && \
		autoconf && \
		./configure && \
		make -j
	@echo "Verilator build completed successfully."

clean-verilator:
	@echo "Cleaning Verilator build artifacts..."
	@cd 3rd-party/verilator && \
		make clean 2>/dev/null || true && \
		rm -rf build 2>/dev/null || true && \
		git apply --reverse ../../scripts/init/patches/verilator.patch 2>/dev/null || true && \
		rm -f .patch-applied
	@echo "Verilator clean completed."
