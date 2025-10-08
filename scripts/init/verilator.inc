# Verilator build targets
# Converted from scripts/init/verilator.sh

.PHONY: build-verilator clean-verilator

build-verilator:
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
		rm -rf build 2>/dev/null || true
	@echo "Verilator clean completed."
