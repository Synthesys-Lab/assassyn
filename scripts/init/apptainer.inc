# Apptainer container build targets
# Creates a containerized snapshot of the Assassyn repository

# Default values for Apptainer builds
APPTAINER_IMAGE ?= vm/apptainer/assassyn.sif
APPTAINER_DEF ?= vm/apptainer/assassyn.def

$(APPTAINER_IMAGE): $(APPTAINER_DEF)
	@echo "=== Building Assassyn Container ==="
	@echo "Building container (this may take a while)..."
	@cd vm/apptainer && apptainer build assassyn.sif assassyn.def
	@echo "=== Container build completed successfully! ==="
	@echo "Container: $(APPTAINER_IMAGE)"
	@echo "Usage: apptainer exec --no-home $(APPTAINER_IMAGE) python your_script.py"

build-apptainer: $(APPTAINER_IMAGE)

clean-apptainer:
	@echo "Cleaning Apptainer containers..."
	@cd vm/apptainer && \
		rm -f assassyn.sif && \
		echo "Apptainer containers cleaned."