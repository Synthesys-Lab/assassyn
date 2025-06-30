.PHONY: verilator main
verilator:
	$(VERILATOR_ROOT)/bin/verilator --cc {}.sv --exe main.cpp --top tb --timing --trace
main: verilator
	make -C obj_dir -f Vtb.mk