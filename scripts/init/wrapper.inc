# Ramulator2 and Wrapper build targets
# Converted from scripts/init/wrapper.sh

.PHONY: build-ramulator2 build-wrapper clean-ramulator2 clean-wrapper patch-ramulator2

# Patch target for ramulator2
patch-ramulator2: 3rd-party/ramulator2/.patch-applied

# Marker file to track patch application
3rd-party/ramulator2/.patch-applied:
	@echo "Checking ramulator2 patch status..."
	@cd 3rd-party/ramulator2 && \
		if git apply --reverse --check ../../scripts/init/patches/ramulator2.patch 2>/dev/null; then \
			echo "Ramulator2 patch already applied — skipping patch step."; \
		else \
			echo "Applying ramulator2 patch..."; \
			git apply ../../scripts/init/patches/ramulator2.patch 2>/dev/null; \
		fi && \
		touch .patch-applied

build-ramulator2: patch-ramulator2
	@echo "Building Ramulator2..."
	@echo "Step 1: Creating build directory..."
	@mkdir -p 3rd-party/ramulator2/build
	@echo "Step 2: Running CMake configuration..."
	@cmake -B 3rd-party/ramulator2/build -S 3rd-party/ramulator2 -DCMAKE_POLICY_VERSION_MINIMUM=3.5
	@echo "Step 3: Building with make..."
	@make -C 3rd-party/ramulator2/build -j
	@echo "Ramulator2 build completed."

build-wrapper: build-ramulator2
	@echo "Building Wrapper..."
	@mkdir -p tools/c-ramulator2-wrapper/build && \
		cd tools/c-ramulator2-wrapper/build && \
		cmake .. && \
		make -j
	@echo "Wrapper build completed."

clean-ramulator2:
	@echo "Cleaning Ramulator2 build artifacts..."
	@cd 3rd-party/ramulator2 && \
		rm -rf build && \
		git apply --reverse ../../scripts/init/patches/ramulator2.patch 2>/dev/null || true && \
		rm -f .patch-applied
	@echo "Ramulator2 clean completed."

clean-wrapper:
	@echo "Cleaning Wrapper build artifacts..."
	@rm -rf tools/c-ramulator2-wrapper/build
	@echo "Wrapper clean completed."
