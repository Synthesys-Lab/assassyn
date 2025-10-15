# Apptainer container build targets
# Creates a containerized snapshot of the Assassyn repository from specified URL and tag

# Default values for Apptainer builds
APPTAINER_BASE_IMAGE ?= vm/apptainer/assassyn-base.sif
APPTAINER_IMAGE ?= vm/apptainer/assassyn.sif
APPTAINER_BASE_DEF ?= vm/apptainer/assassyn-base.def
APPTAINER_DEF ?= vm/apptainer/assassyn.def

# Repository configuration with defaults
REPO_URL ?= https://github.com/Synthesys-Lab/assassyn
REPO_TAG ?= master

$(APPTAINER_BASE_IMAGE): $(APPTAINER_BASE_DEF)
	@echo "=== Building Assassyn Base Container ==="
	@echo "Building base image (this may take a while)..."
	@cd vm/apptainer && apptainer build assassyn-base.sif assassyn-base.def
	@echo "Base image built successfully!"

$(APPTAINER_IMAGE): $(APPTAINER_BASE_IMAGE) $(APPTAINER_DEF)
	@echo "=== Building Assassyn Container ==="
	@echo "Building container with repository: $(REPO_URL) at tag: $(REPO_TAG)"
	@cd vm/apptainer && apptainer build --build-arg REPO_URL="$(REPO_URL)" --build-arg REPO_TAG="$(REPO_TAG)" assassyn.sif assassyn.def
	@echo "=== Container build completed successfully! ==="
	@echo "Container: $(APPTAINER_IMAGE)"
	@echo "Repository: $(REPO_URL) at $(REPO_TAG)"
	@echo "Usage: apptainer exec --no-home $(APPTAINER_IMAGE) python your_script.py"

build-apptainer-base: $(APPTAINER_BASE_IMAGE)

build-apptainer: $(APPTAINER_IMAGE)

clean-apptainer:
	@echo "Cleaning Apptainer containers..."
	@cd vm/apptainer && \
		rm -f assassyn-base.sif assassyn.sif && \
		echo "Apptainer containers cleaned."