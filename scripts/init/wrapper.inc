# Ramulator2 and Wrapper build targets
# Converted from scripts/init/wrapper.sh

.PHONY: build-ramulator2 build-wrapper clean-ramulator2 clean-wrapper patch-ramulator2

# Patch target for ramulator2
patch-ramulator2: 3rd-party/ramulator2/.patch-applied

# Marker file to track patch application
3rd-party/ramulator2/.patch-applied:
	@echo "Checking ramulator2 patch status..."
	@cd 3rd-party/ramulator2 && \
		if bash ../../scripts/patch-apply.sh check ../../scripts/init/patches/ramulator2.patch 2>/dev/null; then \
			echo "Ramulator2 patch already applied — skipping patch step."; \
		else \
			echo "Applying ramulator2 patch..."; \
			bash ../../scripts/patch-apply.sh apply ../../scripts/init/patches/ramulator2.patch; \
		fi && \
		touch $(CURDIR)/3rd-party/ramulator2/.patch-applied

build-ramulator2: 3rd-party/ramulator2/.ramulator2-built

# Marker file to track build completion
3rd-party/ramulator2/.ramulator2-built: patch-ramulator2
	@if [ -f "$@" ]; then \
		echo "Ramulator2 already built — skipping build step."; \
	else \
		echo "Building Ramulator2..."; \
		echo "Step 1: Creating build directory..."; \
		mkdir -p 3rd-party/ramulator2/build; \
		echo "Step 2: Running CMake configuration..."; \
		cmake -B 3rd-party/ramulator2/build -S 3rd-party/ramulator2 -DCMAKE_POLICY_VERSION_MINIMUM=3.5; \
		echo "Step 3: Building with make..."; \
		make -C 3rd-party/ramulator2/build -j; \
		echo "Ramulator2 build completed."; \
		touch $(CURDIR)/$@; \
	fi

build-wrapper: tools/c-ramulator2-wrapper/.wrapper-built

# Marker file to track wrapper build completion
tools/c-ramulator2-wrapper/.wrapper-built: build-ramulator2
	@if [ -f "$@" ]; then \
		echo "Wrapper already built — skipping build step."; \
	else \
		echo "Building Wrapper..."; \
		mkdir -p tools/c-ramulator2-wrapper/build && \
			cd tools/c-ramulator2-wrapper/build && \
			cmake .. && \
			make -j; \
		echo "Wrapper build completed."; \
		touch $(CURDIR)/$@; \
	fi

clean-ramulator2:
	@echo "Cleaning Ramulator2 build artifacts..."
	@cd 3rd-party/ramulator2 && \
		rm -rf build && \
		bash ../../scripts/patch-apply.sh reverse ../../scripts/init/patches/ramulator2.patch 2>/dev/null || true && \
		rm -f .patch-applied .ramulator2-built
	@echo "Ramulator2 clean completed."

clean-wrapper:
	@echo "Cleaning Wrapper build artifacts..."
	@rm -rf tools/c-ramulator2-wrapper/build
	@rm -f tools/c-ramulator2-wrapper/.wrapper-built
	@echo "Wrapper clean completed."
