# Verilator build targets
# Converted from scripts/init/verilator.sh

.PHONY: build-verilator clean-verilator patch-verilator

# Patch target for verilator
patch-verilator: 3rd-party/verilator/.patch-applied

# Marker file to track patch application
3rd-party/verilator/.patch-applied:
	@echo "Checking verilator patch status..."
	@cd 3rd-party/verilator && \
		if bash ../../scripts/patch-apply.sh check ../../scripts/init/patches/verilator.patch 2>/dev/null; then \
			echo "Verilator patch already applied — skipping patch step."; \
		else \
			echo "Applying Verilator patch..."; \
			bash ../../scripts/patch-apply.sh apply ../../scripts/init/patches/verilator.patch; \
		fi && \
		touch $(CURDIR)/3rd-party/verilator/.patch-applied

build-verilator: 3rd-party/verilator/.verilator-built

# Marker file to track build completion
3rd-party/verilator/.verilator-built: patch-verilator
	@if [ -f "$@" ]; then \
		echo "Verilator already built — skipping build step."; \
	else \
		echo "Building Verilator..."; \
		echo "Installing Verilator by building it from source..."; \
		cd 3rd-party/verilator && \
			autoconf && \
			./configure && \
			make -j; \
		echo "Verilator build completed successfully."; \
		touch $(CURDIR)/$@; \
	fi

clean-verilator:
	@echo "Cleaning Verilator build artifacts..."
	@cd 3rd-party/verilator && \
		make clean 2>/dev/null || true && \
		rm -rf build 2>/dev/null || true && \
		bash ../../scripts/patch-apply.sh reverse ../../scripts/init/patches/verilator.patch 2>/dev/null || true && \
		rm -f .patch-applied .verilator-built
	@echo "Verilator clean completed."
