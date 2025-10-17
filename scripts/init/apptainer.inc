# Apptainer container build targets
# Creates a containerized snapshot of the Assassyn repository from specified URL and tag

# Default values for Apptainer builds
APPTAINER_BASE_IMAGE ?= vm/apptainer/assassyn-base.sif
APPTAINER_IMAGE ?= vm/apptainer/assassyn.sif
APPTAINER_BASE_DEF ?= vm/apptainer/assassyn-base.def
APPTAINER_DEF ?= vm/apptainer/assassyn.def
APPTAINER_BUNDLE ?= vm/apptainer/assassyn.bundle

$(APPTAINER_BUNDLE):
	@echo "Creating git bundle..."
	@git bundle create $(APPTAINER_BUNDLE) --all
	@echo "Git bundle created: $(APPTAINER_BUNDLE)"

$(APPTAINER_BASE_IMAGE): $(APPTAINER_BASE_DEF)
	@echo "=== Building Assassyn Base Container ==="
	@echo "Building base image (this may take a while)..."
	@cd vm/apptainer && apptainer build assassyn-base.sif assassyn-base.def
	@echo "Base image built successfully!"

$(APPTAINER_IMAGE): $(APPTAINER_BASE_IMAGE) $(APPTAINER_DEF) $(APPTAINER_BUNDLE)
	@echo "=== Building Assassyn Container ==="
	@echo "Building container with local git bundle"
	@cd vm/apptainer && apptainer build assassyn.sif assassyn.def
	@echo "=== Container build completed successfully! ==="
	@echo "Container: $(APPTAINER_IMAGE)"
	@echo "Usage: apptainer exec --no-home $(APPTAINER_IMAGE) python your_script.py"

build-apptainer-base: $(APPTAINER_BASE_IMAGE)

build-apptainer-bundle: $(APPTAINER_BUNDLE)

build-apptainer: $(APPTAINER_IMAGE)

clean-apptainer:
	@echo "Cleaning Apptainer containers..."
	@cd vm/apptainer && \
		rm -f assassyn-base.sif assassyn.sif assassyn.bundle && \
		echo "Apptainer containers cleaned."