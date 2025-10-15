# Build System Architecture

## Overview

The Assassyn build system provides a unified interface for building, testing, and cleaning the project components. The system supports both simulation and RTL generation through a modular architecture that handles third-party dependencies and custom patches.

## Architecture Components

### 1. Physical Machine vs VM Build Separation

The build system operates in two distinct environments:

- **Physical Machine**: Where patches are applied and build artifacts are cached
- **VM Environment**: Where the actual build process runs inside containers

This separation ensures that:
- Patches are applied with correct relative paths on the physical machine
- Build cache is shared between physical machine and VM
- VM environment remains isolated and reproducible

### 2. Patch Management System

#### Patch Application Strategy

Patches are applied on the physical machine before VM initialization to avoid path mismatch issues:

```
Physical Machine → Apply Patches → Build Container → VM Build Process
```

#### Patch Targets Structure

Each component has a dedicated patch target following the pattern:
```makefile
patch-<component>: 3rd-party/<component>/.patch-applied
```

The `.patch-applied` marker file ensures:
- Patches are only applied once
- Build system can track patch status
- Clean targets can properly reverse patches

#### Patch Status Verification

Before applying patches, the system verifies current patch status:
```bash
git apply --reverse --check <patch-file>
```

This prevents:
- Double application of patches
- Patch conflicts
- Build failures due to incorrect patch state

### 3. Build Target Dependencies

The build system follows a clear dependency chain:

```
build-<component> → patch-<component> → .patch-applied marker
```

This ensures:
- Patches are applied before building
- Build targets are idempotent
- Clean targets properly reverse all changes

### 4. Cache Management

The build cache includes:
- Patch files (`scripts/init/patches/*.patch`)
- Build artifacts
- Submodule hashes
- Makefile and build script changes

Cache keys are designed to invalidate when:
- Patches are modified
- Submodules are updated
- Build scripts change

## Implementation Details

### Patch-All Target

The `patch-all` target applies all necessary patches on the physical machine:

```makefile
patch-all: patch-ramulator2 patch-circt patch-verilator
```

This target:
- Runs before VM initialization
- Ensures all patches are applied with correct paths
- Provides unified patch management interface

### Component-Specific Patch Targets

Each component has its own patch target that:
- Checks patch status before application
- Applies patches only if needed
- Creates marker files for tracking
- Handles patch reversal for clean operations

### Error Handling

The patch system includes comprehensive error handling:
- Patch application failures are caught and reported
- Rollback procedures are documented
- Build process fails fast on patch errors
- Clean targets reverse patches safely

## Benefits

1. **Reliability**: Patches applied on physical machine with correct paths
2. **Performance**: Avoid re-applying patches in VM environment
3. **Consistency**: Unified patch application process
4. **Debugging**: Easier to debug patch issues on physical machine
5. **Maintainability**: Centralized patch management
6. **CI/CD Stability**: Eliminates VM-specific patch path issues

## Future Considerations

- Patch conflict detection and resolution
- Automated patch validation
- Patch dependency management
- Integration with submodule update process
