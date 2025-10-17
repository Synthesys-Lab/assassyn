# Apptainer container build targets
# Creates a containerized snapshot of the Assassyn repository

# Default values for Apptainer builds
APPTAINER_BASE_IMAGE ?= vm/apptainer/base.sif
APPTAINER_REPO_IMAGE ?= vm/apptainer/repo.sif
APPTAINER_IMAGE ?= vm/apptainer/assassyn.sif
APPTAINER_BASE_DEF ?= vm/apptainer/base.def
APPTAINER_REPO_DEF ?= vm/apptainer/repo.def
APPTAINER_DEF ?= vm/apptainer/assassyn.def

# Phase 1: Build base container with system dependencies
$(APPTAINER_BASE_IMAGE): $(APPTAINER_BASE_DEF)
	@echo "=== Building Base Container ==="
	@echo "Building base container with system dependencies..."
	@cd vm/apptainer && apptainer build base.sif base.def
	@echo "=== Base container build completed successfully! ==="
	@echo "Container: $(APPTAINER_BASE_IMAGE)"

# Phase 2: Build repo container with cloned repository
$(APPTAINER_REPO_IMAGE): $(APPTAINER_BASE_IMAGE) $(APPTAINER_REPO_DEF)
	@echo "=== Building Repository Container ==="
	@echo "Building repository container with cloned repo and submodules..."
	@cd vm/apptainer && apptainer build repo.sif repo.def
	@echo "=== Repository container build completed successfully! ==="
	@echo "Container: $(APPTAINER_REPO_IMAGE)"

# Phase 3: Build final container with everything built
$(APPTAINER_IMAGE): $(APPTAINER_REPO_IMAGE) $(APPTAINER_DEF)
	@echo "=== Building Assassyn Container ==="
	@echo "Building final container (this may take a while)..."
	@cd vm/apptainer && apptainer build assassyn.sif assassyn.def
	@echo "=== Container build completed successfully! ==="
	@echo "Container: $(APPTAINER_IMAGE)"
	@echo "Usage: apptainer exec --no-home $(APPTAINER_IMAGE) python your_script.py"

# Individual build targets
build-apptainer-base: $(APPTAINER_BASE_IMAGE)

build-apptainer-repo: $(APPTAINER_REPO_IMAGE)

build-apptainer: $(APPTAINER_IMAGE)

# Clean targets
clean-apptainer-base:
	@echo "Cleaning base Apptainer container..."
	@cd vm/apptainer && \
		rm -f base.sif && \
		echo "Base Apptainer container cleaned."

clean-apptainer-repo:
	@echo "Cleaning repository Apptainer container..."
	@cd vm/apptainer && \
		rm -f repo.sif && \
		echo "Repository Apptainer container cleaned."

clean-apptainer:
	@echo "Cleaning all Apptainer containers..."
	@cd vm/apptainer && \
		rm -f base.sif repo.sif assassyn.sif && \
		echo "All Apptainer containers cleaned."