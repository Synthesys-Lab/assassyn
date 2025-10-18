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

# Submodules to cache
SUBMODULES := ramulator2 circt verilator

# Compute hash for each submodule
RAMULATOR2_HASH := $(shell git -C 3rd-party/ramulator2 rev-parse --short HEAD 2>/dev/null || echo "unknown")
CIRCT_HASH := $(shell git -C 3rd-party/circt rev-parse --short HEAD 2>/dev/null || echo "unknown")
VERILATOR_HASH := $(shell git -C 3rd-party/verilator rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Cached archive paths
RAMULATOR2_ARCHIVE := $(APPTAINER_OUTPUT_DIR)/ramulator2-$(RAMULATOR2_HASH).tar.gz
CIRCT_ARCHIVE := $(APPTAINER_OUTPUT_DIR)/circt-$(CIRCT_HASH).tar.gz
VERILATOR_ARCHIVE := $(APPTAINER_OUTPUT_DIR)/verilator-$(VERILATOR_HASH).tar.gz

# All submodule archives
SUBMODULE_ARCHIVES := $(RAMULATOR2_ARCHIVE) $(CIRCT_ARCHIVE) $(VERILATOR_ARCHIVE)

# Phase 1: Build base container with system dependencies
$(APPTAINER_BASE_IMAGE): $(APPTAINER_BASE_DEF)
	@echo "=== Building Base Container ==="
	@echo "Building base container with system dependencies..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	@cd vm/apptainer && apptainer build $(APPTAINER_OUTPUT_DIR)/base.sif base.def
	@echo "=== Base container build completed successfully! ==="
	@echo "Container: $(APPTAINER_BASE_IMAGE)"

# Pattern rule to archive any submodule with hash-based caching
$(APPTAINER_OUTPUT_DIR)/ramulator2-%.tar.gz: 3rd-party/ramulator2
	@echo "=== Archiving ramulator2 ($*) ==="
	@mkdir -p $(APPTAINER_OUTPUT_DIR)
	@tar -czf $@ -C 3rd-party ramulator2
	@echo "Archive created: $@"

$(APPTAINER_OUTPUT_DIR)/circt-%.tar.gz: 3rd-party/circt
	@echo "=== Archiving circt ($*) ==="
	@mkdir -p $(APPTAINER_OUTPUT_DIR)
	@tar -czf $@ -C 3rd-party circt
	@echo "Archive created: $@"

$(APPTAINER_OUTPUT_DIR)/verilator-%.tar.gz: 3rd-party/verilator
	@echo "=== Archiving verilator ($*) ==="
	@mkdir -p $(APPTAINER_OUTPUT_DIR)
	@tar -czf $@ -C 3rd-party verilator
	@echo "Archive created: $@"

# Archive the repository with cached submodules
$(APPTAINER_OUTPUT_DIR)/repo.tar.gz: $(SUBMODULE_ARCHIVES)
	@echo "=== Creating Repository Archive ==="
	@echo "Assembling repository with cached submodules..."
	@mkdir -p $(APPTAINER_OUTPUT_DIR)/temp_archive/app/3rd-party
	@rsync -a --exclude='3rd-party' . $(APPTAINER_OUTPUT_DIR)/temp_archive/app/
	@cd $(APPTAINER_OUTPUT_DIR)/temp_archive/app/3rd-party && \
		tar -xzf $(RAMULATOR2_ARCHIVE) && \
		tar -xzf $(CIRCT_ARCHIVE) && \
		tar -xzf $(VERILATOR_ARCHIVE)
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
	@rm -f $(APPTAINER_OUTPUT_DIR)/base.sif $(APPTAINER_OUTPUT_DIR)/repo.sif \
		$(APPTAINER_OUTPUT_DIR)/assassyn.sif $(APPTAINER_OUTPUT_DIR)/repo.tar.gz && \
		echo "All Apptainer containers and main archive cleaned."

ultraclean-apptainer: clean-apptainer
	@echo "Cleaning cached submodule archives..."
	@rm -f $(APPTAINER_OUTPUT_DIR)/ramulator2-*.tar.gz \
		$(APPTAINER_OUTPUT_DIR)/circt-*.tar.gz \
		$(APPTAINER_OUTPUT_DIR)/verilator-*.tar.gz && \
		echo "All cached submodule archives cleaned."