# Assassyn Apptainer Container

Assassyn can be run using [Apptainer](https://apptainer.org/) (formerly Singularity) for containerized execution. This approach provides a portable, reproducible environment that creates a snapshot of your current repository with all dependencies pre-installed.

## Quick Start

From the project root directory:

```sh
# Build container (snapshots current repository state)
make build-apptainer

# Clean all containers
make clean-apptainer
```

This creates a containerized snapshot of your current Assassyn repository state.

## Available Makefile Targets

The Apptainer build system provides several targets for different use cases:

### Primary Target
- `make build-apptainer`: Build container with current repository snapshot

### Additional Targets
- `make build-apptainer-base`: Build only the base container (system dependencies)
- `make clean-apptainer`: Remove all generated container files

**Examples:**
```sh
# Build container with current repo snapshot
make build-apptainer

# Build only the base container (rarely needed)
make build-apptainer-base

# Clean up all containers
make clean-apptainer
```

## Execute Commands

```sh
# Run Python scripts
apptainer exec --no-home assassyn.sif python main.py

# Run with specific directory binding
apptainer exec --bind /path/to/your/code assassyn.sif python /path/to/your/code/main.py

# Interactive shell
apptainer shell assassyn.sif
```

By default, Apptainer automatically binds the `$HOME` directory to the container. Use `--no-home` to disable this binding.

## Container Files

- `assassyn-base.def`: Base container with system dependencies (Rust, Python, build tools)
- `assassyn.def`: Main container definition that snapshots the current repository
- `scripts/init/apptainer.inc`: Makefile include with Apptainer build targets
- `assassyn-base.sif`: Generated base image (shared across all builds)
- `assassyn.sif`: Generated container image (contains current repository snapshot)

## Environment Variables and Defaults

The container system uses the current local directory and includes these pre-configured environment variables:
- `ASSASSYN_HOME`: Assassyn installation directory
- `VERILATOR_ROOT`: Verilator installation path
- `PYTHONPATH`: Python module search path
- `RUSTC_WRAPPER`: Rust compiler wrapper for caching
- `CC`/`CXX`: Compiler settings with ccache

## Design Purpose

This multi-stage container design addresses several key requirements:

### Repository Snapshotting
The container system creates a snapshot of your current repository state, enabling:
- **Reproducible Builds**: Exact same environment across different systems
- **Version Control**: Container reflects the exact state of your working directory
- **Portability**: Move containers between systems with identical behavior

### Build Optimization
The two-stage approach optimizes build times:
- **Base Image**: Contains all system dependencies (Rust, Python, build tools) - shared across all builds
- **Repository Image**: Contains only the Assassyn code from your current working directory - rebuilt as needed

This separation means:
- Dependencies are cached in the base image
- Only code changes require rebuilding
- Significantly faster iteration for development

### Reproducible Environments
Each container build creates a reproducible environment ensuring:
- Consistent build environments across different systems
- Isolated dependencies per repository state
- Easy deployment and testing

### CI/CD Integration
The design supports automated workflows:
- Environment variables control repository state
- Makefile targets can be easily integrated into GitHub Actions
- Containers can be built for any repository state automatically
- Unified build system with other project components

### Integration with Main Build System
The Apptainer build system is fully integrated with the main project Makefile:
- **Unified Interface**: All build targets accessible via `make` commands
- **Consistent Pattern**: Follows the same structure as other component builds (Verilator, Ramulator2, etc.)
- **Dependency Management**: Automatically handles base container dependencies
- **Error Handling**: Includes proper validation and error checking
- **Clean Targets**: Provides cleanup functionality consistent with other components

### Usage Flexibility
The container system supports multiple use cases:
- **Development**: Interactive shells for development work
- **Testing**: Automated test execution in isolated environments
- **Deployment**: Consistent runtime environments across different systems
- **Distribution**: Portable environments that work across different platforms
