# Apptainer container build targets
# Converted from vm/apptainer/build-assassyn.sh

.PHONY: build-apptainer-base build-apptainer-branch build-apptainer clean-apptainer

# Default values for Apptainer builds
APPTAINER_BRANCH ?= master
APPTAINER_REPO ?= https://github.com/Synthesys-Lab/assassyn.git
APPTAINER_BASE_IMAGE ?= vm/apptainer/assassyn-base-$(APPTAINER_BRANCH).sif
APPTAINER_BRANCH_IMAGE ?= vm/apptainer/assassyn-$(APPTAINER_BRANCH).sif
APPTAINER_BASE_DEF ?= vm/apptainer/assassyn-base.def
APPTAINER_BRANCH_DEF ?= vm/apptainer/assassyn-branch.def

$(APPTAINER_BASE_IMAGE): $(APPTAINER_BASE_DEF)
	@echo "=== Building Assassyn Base Container for branch: $(APPTAINER_BRANCH) ==="
	@echo "Building base image (this may take a while)..."
	@cd vm/apptainer && apptainer build assassyn-base-$(APPTAINER_BRANCH).sif assassyn-base.def
	@echo "Base image built successfully!"

$(APPTAINER_BRANCH_IMAGE): $(APPTAINER_BASE_IMAGE) $(APPTAINER_BRANCH_DEF)
	@echo "=== Building Assassyn Branch Container for branch: $(APPTAINER_BRANCH) ==="
	@echo "Building branch image for $(APPTAINER_BRANCH)..."
	@cd vm/apptainer && apptainer build --build-arg ASSASSYN_REPO=$(APPTAINER_REPO) --build-arg ASSASSYN_BRANCH=$(APPTAINER_BRANCH) assassyn-$(APPTAINER_BRANCH).sif assassyn-branch.def
	@echo "=== Branch container build completed successfully! ==="
	@echo "Container: $(APPTAINER_BRANCH_IMAGE)"
	@echo "Usage: apptainer exec --no-home $(APPTAINER_BRANCH_IMAGE) python your_script.py"

build-apptainer-base: $(APPTAINER_BASE_IMAGE)

build-apptainer-branch: $(APPTAINER_BRANCH_IMAGE)

# Alias for backward compatibility
build-apptainer: build-apptainer-branch

clean-apptainer:
	@echo "Cleaning Apptainer containers..."
	@cd vm/apptainer && \
		rm -f assassyn-base-*.sif assassyn-*.sif && \
		echo "Apptainer containers cleaned."
