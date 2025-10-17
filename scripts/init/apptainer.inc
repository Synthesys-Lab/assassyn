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

# Archive the repository with all submodules
$(APPTAINER_OUTPUT_DIR)/repo.tar.gz:
	@echo "=== Creating Repository Archive ==="
	@echo "Archiving repository with all submodules..."
	@mkdir -p $(APPTAINER_OUTPUT_DIR)/temp_archive
	@cp -r . $(APPTAINER_OUTPUT_DIR)/temp_archive/app
	@cd $(APPTAINER_OUTPUT_DIR)/temp_archive && \
		tar --exclude='app/.git' --exclude='app/examples' --exclude='app/docs' --exclude='app/tutorials' \
		-czf $(APPTAINER_OUTPUT_DIR)/repo.tar.gz app
	@rm -rf $(APPTAINER_OUTPUT_DIR)/temp_archive
	@echo "Archive created: $(APPTAINER_OUTPUT_DIR)/repo.tar.gz"

# Phase 2: Build repo container with archived repository
$(APPTAINER_REPO_IMAGE): $(APPTAINER_BASE_IMAGE) $(APPTAINER_REPO_DEF) $(APPTAINER_OUTPUT_DIR)/repo.tar.gz
	@echo "=== Building Repository Container ==="
	@echo "Building repository container with archived repo and submodules..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	@cd vm/apptainer && apptainer build $(APPTAINER_OUTPUT_DIR)/repo.sif repo.def
	@echo "=== Repository container build completed successfully! ==="
	@echo "Container: $(APPTAINER_REPO_IMAGE)"

# Phase 3: Build final container with everything built
$(APPTAINER_IMAGE): $(APPTAINER_REPO_IMAGE) $(APPTAINER_DEF)
	@echo "=== Building Assassyn Container ==="
	@echo "Building final container (this may take a while)..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	cd vm/apptainer && apptainer build $(APPTAINER_OUTPUT_DIR)/assassyn.sif assassyn.def
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
	@rm -f $(APPTAINER_OUTPUT_DIR)/repo.sif $(APPTAINER_OUTPUT_DIR)/repo.tar.gz && \
		echo "Repository Apptainer container and archive cleaned."

clean-apptainer:
	@echo "Cleaning all Apptainer containers..."
	@rm -f $(APPTAINER_OUTPUT_DIR)/base.sif $(APPTAINER_OUTPUT_DIR)/repo.sif $(APPTAINER_OUTPUT_DIR)/assassyn.sif $(APPTAINER_OUTPUT_DIR)/repo.tar.gz && \
		echo "All Apptainer containers and archives cleaned."