# Ramulator2 and Wrapper build targets
# Converted from scripts/init/wrapper.sh

.PHONY: build-ramulator2 build-wrapper clean-ramulator2 clean-wrapper

# Marker file to track patch application
3rd-party/ramulator2/.patch-applied:
	@echo "Checking ramulator2 patch status..."
	@cd 3rd-party/ramulator2 && \
		if git apply --reverse --check ../../scripts/ramulator2-template.patch 2>/dev/null; then \
			echo "Ramulator2 patch already applied — skipping patch step."; \
		else \
			echo "Applying ramulator2 patch..."; \
			git apply ../../scripts/ramulator2-template.patch 2>/dev/null; \
			if [ $$? -ne 0 ]; then \
				echo "Failed to apply ramulator2 patch."; \
				exit 1; \
			fi; \
		fi && \
		touch .patch-applied

build-ramulator2: 3rd-party/ramulator2/.patch-applied
	@echo "Building Ramulator2..."
	@cd 3rd-party/ramulator2 && \
		git submodule update --init && \
		mkdir -p build && \
		cd build && \
		cmake .. && \
		make -j
	@if [ $$? -ne 0 ]; then \
		echo "Failed to build Ramulator2."; \
		exit 1; \
	fi
	@echo "Ramulator2 build completed."

build-wrapper: build-ramulator2
	@echo "Building Wrapper..."
	@mkdir -p tools/c-ramulator2-wrapper/build && \
		cd tools/c-ramulator2-wrapper/build && \
		cmake .. && \
		make -j
	@if [ $$? -ne 0 ]; then \
		echo "Failed to build Wrapper."; \
		exit 1; \
	fi
	@echo "Wrapper build completed."

clean-ramulator2:
	@echo "Cleaning Ramulator2 build artifacts..."
	@cd 3rd-party/ramulator2 && \
		rm -rf build && \
		git apply --reverse ../../scripts/ramulator2-template.patch 2>/dev/null || true && \
		rm -f .patch-applied
	@echo "Ramulator2 clean completed."

clean-wrapper:
	@echo "Cleaning Wrapper build artifacts..."
	@rm -rf tools/c-ramulator2-wrapper/build
	@echo "Wrapper clean completed."
