# Apptainer container build targets
# Converted from vm/apptainer/build-assassyn.sh

.PHONY: build-apptainer-base build-apptainer-branch build-apptainer clean-apptainer

# Default values for Apptainer builds
APPTAINER_BRANCH ?= master
APPTAINER_REPO ?= https://github.com/Synthesys-Lab/assassyn.git
APPTAINER_BASE_IMAGE = vm/apptainer/assassyn-base.sif
APPTAINER_BRANCH_IMAGE = vm/apptainer/assassyn-$(APPTAINER_BRANCH).sif
APPTAINER_BASE_DEF = vm/apptainer/assassyn-base.def
APPTAINER_BRANCH_DEF = vm/apptainer/assassyn-branch.def

build-apptainer-base:
	@echo "=== Building Assassyn Base Container ==="
	@if [ ! -f "$(APPTAINER_BASE_DEF)" ]; then \
		echo "Error: $(APPTAINER_BASE_DEF) not found!"; \
		exit 1; \
	fi
	@if [ ! -f "$(APPTAINER_BASE_IMAGE)" ]; then \
		echo "Building base image (this may take a while)..."; \
		cd vm/apptainer && apptainer build assassyn-base.sif assassyn-base.def; \
		echo "Base image built successfully!"; \
	else \
		echo "Base image already exists, skipping..."; \
	fi

build-apptainer-branch: build-apptainer-base
	@echo "=== Building Assassyn Branch Container for branch: $(APPTAINER_BRANCH) ==="
	@if [ ! -f "$(APPTAINER_BRANCH_DEF)" ]; then \
		echo "Error: $(APPTAINER_BRANCH_DEF) not found!"; \
		exit 1; \
	fi
	@echo "Building branch image for $(APPTAINER_BRANCH)..."
	@cd vm/apptainer && apptainer build --build-arg ASSASSYN_REPO=$(APPTAINER_REPO) --build-arg ASSASSYN_BRANCH=$(APPTAINER_BRANCH) assassyn-$(APPTAINER_BRANCH).sif assassyn-branch.def
	@echo "=== Branch container build completed successfully! ==="
	@echo "Container: $(APPTAINER_BRANCH_IMAGE)"
	@echo "Usage: apptainer exec --no-home $(APPTAINER_BRANCH_IMAGE) python your_script.py"

# Alias for backward compatibility
build-apptainer: build-apptainer-branch

clean-apptainer:
	@echo "Cleaning Apptainer containers..."
	@cd vm/apptainer && \
		rm -f assassyn-*.sif && \
		echo "Apptainer containers cleaned."
