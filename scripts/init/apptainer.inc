# Apptainer container build targets
# Creates a containerized snapshot of the Assassyn repository
# Note: The repository URL and branch/tag are configured in vm/apptainer/repo.def

# Default values for Apptainer builds
# Always use /tmp as the output directory
APPTAINER_OUTPUT_DIR ?= /tmp
APPTAINER_BASE_IMAGE ?= $(APPTAINER_OUTPUT_DIR)/base.sif
APPTAINER_REPO_IMAGE ?= $(APPTAINER_OUTPUT_DIR)/repo.sif
APPTAINER_IMAGE ?= $(APPTAINER_OUTPUT_DIR)/assassyn.sif
APPTAINER_BASE_DEF ?= vm/apptainer/base.def
APPTAINER_REPO_DEF ?= vm/apptainer/repo.def
APPTAINER_DEF ?= vm/apptainer/assassyn.def

# Phase 1: Build base container with system dependencies
$(APPTAINER_BASE_IMAGE): $(APPTAINER_BASE_DEF)
	@echo "=== Building Base Container ==="
	@echo "Building base container with system dependencies..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	@cd vm/apptainer && apptainer build $(APPTAINER_OUTPUT_DIR)/base.sif base.def
	@echo "=== Base container build completed successfully! ==="
	@echo "Container: $(APPTAINER_BASE_IMAGE)"

# Phase 2: Build repo container with cloned repository
$(APPTAINER_REPO_IMAGE): $(APPTAINER_BASE_IMAGE) $(APPTAINER_REPO_DEF)
	@echo "=== Building Repository Container ==="
	@echo "Building repository container with cloned repo and submodules..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	@REPO_URL=$$(git remote get-url origin) && \
	 REPO_REF=$$(git describe --tags --exact-match HEAD 2>/dev/null || git rev-parse --abbrev-ref HEAD) && \
	 cd vm/apptainer && apptainer build --build-arg REPO_URL="$$REPO_URL" --build-arg REPO_REF="$$REPO_REF" $(APPTAINER_OUTPUT_DIR)/repo.sif repo.def
	@echo "=== Repository container build completed successfully! ==="
	@echo "Container: $(APPTAINER_REPO_IMAGE)"

# Phase 3: Build final container with everything built
$(APPTAINER_IMAGE): $(APPTAINER_REPO_IMAGE) $(APPTAINER_DEF)
	@echo "=== Building Assassyn Container ==="
	@echo "Building final container (this may take a while)..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	@cd vm/apptainer && apptainer build $(APPTAINER_OUTPUT_DIR)/assassyn.sif assassyn.def
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
	@rm -f $(APPTAINER_OUTPUT_DIR)/base.sif && \
		echo "Base Apptainer container cleaned."

clean-apptainer-repo:
	@echo "Cleaning repository Apptainer container..."
	@rm -f $(APPTAINER_OUTPUT_DIR)/repo.sif && \
		echo "Repository Apptainer container cleaned."

clean-apptainer:
	@echo "Cleaning all Apptainer containers..."
	@rm -f $(APPTAINER_OUTPUT_DIR)/base.sif $(APPTAINER_OUTPUT_DIR)/repo.sif $(APPTAINER_OUTPUT_DIR)/assassyn.sif && \
		echo "All Apptainer containers cleaned."