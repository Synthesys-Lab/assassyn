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

# Compute hash for main repo
ASSASSYN_HASH := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Compute hash for each submodule
RAMULATOR2_HASH := $(shell git -C 3rd-party/ramulator2 rev-parse --short HEAD 2>/dev/null || echo "unknown")
CIRCT_HASH := $(shell git -C 3rd-party/circt rev-parse --short HEAD 2>/dev/null || echo "unknown")
VERILATOR_HASH := $(shell git -C 3rd-party/verilator rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Main repo archive
ASSASSYN_ARCHIVE := $(APPTAINER_OUTPUT_DIR)/assassyn-$(ASSASSYN_HASH).tar.gz

# Cached archive paths
RAMULATOR2_ARCHIVE := $(APPTAINER_OUTPUT_DIR)/ramulator2-$(RAMULATOR2_HASH).tar.gz
CIRCT_ARCHIVE := $(APPTAINER_OUTPUT_DIR)/circt-$(CIRCT_HASH).tar.gz
VERILATOR_ARCHIVE := $(APPTAINER_OUTPUT_DIR)/verilator-$(VERILATOR_HASH).tar.gz

# All submodule archives
SUBMODULE_ARCHIVES := $(RAMULATOR2_ARCHIVE) $(CIRCT_ARCHIVE) $(VERILATOR_ARCHIVE)

# All archives (main repo + submodules)
ALL_ARCHIVES := $(ASSASSYN_ARCHIVE) $(SUBMODULE_ARCHIVES)

# Phase 1: Build base container with system dependencies
$(APPTAINER_BASE_IMAGE): $(APPTAINER_BASE_DEF)
	@echo "=== Building Base Container ==="
	@echo "Building base container with system dependencies..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	@cd vm/apptainer && apptainer build $(APPTAINER_OUTPUT_DIR)/base.sif base.def
	@echo "=== Base container build completed successfully! ==="
	@echo "Container: $(APPTAINER_BASE_IMAGE)"

# Create assassyn main repo archive
$(ASSASSYN_ARCHIVE):
	@echo "=== Archiving assassyn main repo ($(ASSASSYN_HASH)) ==="
	@mkdir -p $(APPTAINER_OUTPUT_DIR)
	@git archive --format=tar.gz --prefix=assassyn/ HEAD > $@
	@echo "Archive created: $@"

# Pattern rule to archive any submodule with hash-based caching using git archive (recursive)
$(APPTAINER_OUTPUT_DIR)/ramulator2-%.tar.gz: 3rd-party/ramulator2
	@echo "=== Archiving ramulator2 ($*) ==="
	@mkdir -p $(APPTAINER_OUTPUT_DIR)
	@git -C 3rd-party/ramulator2 archive --format=tar.gz --prefix=ramulator2/ HEAD > $@
	@echo "Archive created: $@"

$(APPTAINER_OUTPUT_DIR)/circt-%.tar.gz: 3rd-party/circt
	@echo "=== Archiving circt ($*) ==="
	@mkdir -p $(APPTAINER_OUTPUT_DIR)
	@cd 3rd-party/circt && \
		git archive --format=tar --prefix=circt/ HEAD > /tmp/circt-combined.tar && \
		git submodule foreach 'git archive --format=tar --prefix=circt/$$sm_path/ HEAD >> /tmp/circt-combined.tar' && \
		gzip -c /tmp/circt-combined.tar > $@ && \
		rm -f /tmp/circt-combined.tar
	@echo "Archive created: $@"

$(APPTAINER_OUTPUT_DIR)/verilator-%.tar.gz: 3rd-party/verilator
	@echo "=== Archiving verilator ($*) ==="
	@mkdir -p $(APPTAINER_OUTPUT_DIR)
	@git -C 3rd-party/verilator archive --format=tar.gz --prefix=verilator/ HEAD > $@
	@echo "Archive created: $@"

# Archive the repository with cached submodules
$(APPTAINER_OUTPUT_DIR)/repo.tar.gz: $(ASSASSYN_ARCHIVE) $(RAMULATOR2_ARCHIVE) $(CIRCT_ARCHIVE) $(VERILATOR_ARCHIVE)
	@echo "=== Creating Repository Archive Bundle ==="
	@echo "Bundling repository archives with hash-based naming..."
	@mkdir -p $(APPTAINER_OUTPUT_DIR)/temp_bundle/3rd-party
	@echo "Copying archives..."
	@cp $(ASSASSYN_ARCHIVE) $(APPTAINER_OUTPUT_DIR)/temp_bundle/
	@cp $(RAMULATOR2_ARCHIVE) $(APPTAINER_OUTPUT_DIR)/temp_bundle/3rd-party/
	@cp $(CIRCT_ARCHIVE) $(APPTAINER_OUTPUT_DIR)/temp_bundle/3rd-party/
	@cp $(VERILATOR_ARCHIVE) $(APPTAINER_OUTPUT_DIR)/temp_bundle/3rd-party/
	@echo "Creating final bundle..."
	@cd $(APPTAINER_OUTPUT_DIR)/temp_bundle && \
		tar -czf $(APPTAINER_OUTPUT_DIR)/repo.tar.gz .
	@rm -rf $(APPTAINER_OUTPUT_DIR)/temp_bundle
	@echo "Archive bundle created: $(APPTAINER_OUTPUT_DIR)/repo.tar.gz"

# Phase 2: Build repo container with archived repository
$(APPTAINER_REPO_IMAGE): $(APPTAINER_BASE_IMAGE) $(APPTAINER_REPO_DEF) $(APPTAINER_OUTPUT_DIR)/repo.tar.gz
	@echo "=== Building Repository Container ==="
	@echo "Building repository container with archived repo and submodules..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	@cd vm/apptainer && apptainer build --force $(APPTAINER_OUTPUT_DIR)/repo.sif repo.def
	@echo "=== Repository container build completed successfully! ==="
	@echo "Container: $(APPTAINER_REPO_IMAGE)"

# Phase 3: Build final container with everything built
$(APPTAINER_IMAGE): $(APPTAINER_REPO_IMAGE) $(APPTAINER_DEF)
	@echo "=== Building Assassyn Container ==="
	@echo "Building final container (this may take a while)..."
	@echo "Output directory: $(APPTAINER_OUTPUT_DIR)"
	cd vm/apptainer && apptainer build --force $(APPTAINER_OUTPUT_DIR)/assassyn.sif assassyn.def
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
	@rm -f $(APPTAINER_OUTPUT_DIR)/repo.sif $(APPTAINER_OUTPUT_DIR)/repo.tar.gz $(ASSASSYN_ARCHIVE) && \
		echo "Repository Apptainer container and archives cleaned."

clean-apptainer:
	@echo "Cleaning all Apptainer containers..."
	@rm -f $(APPTAINER_OUTPUT_DIR)/base.sif $(APPTAINER_OUTPUT_DIR)/repo.sif \
		$(APPTAINER_OUTPUT_DIR)/assassyn.sif $(APPTAINER_OUTPUT_DIR)/repo.tar.gz $(ASSASSYN_ARCHIVE) && \
		echo "All Apptainer containers and main archive cleaned."

ultraclean-apptainer: clean-apptainer
	@echo "Cleaning cached submodule archives..."
	@rm -f $(APPTAINER_OUTPUT_DIR)/ramulator2-*.tar.gz \
		$(APPTAINER_OUTPUT_DIR)/circt-*.tar.gz \
		$(APPTAINER_OUTPUT_DIR)/verilator-*.tar.gz && \
		echo "All cached submodule archives cleaned."