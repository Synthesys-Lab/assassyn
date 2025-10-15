# Apptainer container build targets
# Creates a containerized snapshot of the current Assassyn repository

.PHONY: build-apptainer-base build-apptainer clean-apptainer

# Default values for Apptainer builds
APPTAINER_BASE_IMAGE ?= vm/apptainer/assassyn-base.sif
APPTAINER_IMAGE ?= vm/apptainer/assassyn.sif
APPTAINER_BASE_DEF ?= vm/apptainer/assassyn-base.def
APPTAINER_DEF ?= vm/apptainer/assassyn.def

$(APPTAINER_BASE_IMAGE): $(APPTAINER_BASE_DEF)
	@echo "=== Building Assassyn Base Container ==="
	@echo "Building base image (this may take a while)..."
	@cd vm/apptainer && apptainer build --force assassyn-base.sif assassyn-base.def
	@echo "Base image built successfully!"

$(APPTAINER_IMAGE): $(APPTAINER_BASE_IMAGE) $(APPTAINER_DEF)
	@echo "=== Building Assassyn Container ==="
	@echo "Creating clean git archive snapshot..."
	@git archive --format=tar.gz --prefix=assassyn-$(shell git describe --tags --always)/ HEAD > /tmp/assassyn-source.tar.gz
	@echo "Creating containerized snapshot of current repository..."
	@cd vm/apptainer && apptainer build --force --bind /tmp/assassyn-source.tar.gz:/tmp/assassyn-source.tar.gz assassyn.sif assassyn.def
	@rm -f /tmp/assassyn-source.tar.gz
	@echo "=== Container build completed successfully! ==="
	@echo "Container: $(APPTAINER_IMAGE)"
	@echo "Usage: apptainer exec --no-home $(APPTAINER_IMAGE) python your_script.py"

build-apptainer-base: $(APPTAINER_BASE_IMAGE)

build-apptainer: $(APPTAINER_IMAGE)

clean-apptainer:
	@echo "Cleaning Apptainer containers..."
	@cd vm/apptainer && \
		rm -f assassyn-base.sif assassyn.sif && \
		echo "Apptainer containers cleaned."