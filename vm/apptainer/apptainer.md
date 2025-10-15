# Assassyn Apptainer Container

Assassyn can be run using [Apptainer](https://apptainer.org/) (formerly Singularity) for containerized execution. This approach provides a portable, reproducible environment with all dependencies pre-installed.

## Quick Start

From the project root directory:

```sh
# Build container for master branch (default)
make build-apptainer

# Build container for a specific branch
make build-apptainer APPTAINER_BRANCH=<branch-name>

# Build container from a different repository
make build-apptainer APPTAINER_REPO=https://github.com/user/fork.git

# Clean all containers
make clean-apptainer
```

This creates a containerized Assassyn environment for the specified branch.

## Available Makefile Targets

The Apptainer build system provides several targets for different use cases:

### Primary Target
- `make build-apptainer`: Build branch-specific container (alias for `build-apptainer-branch`)

### Additional Targets
- `make build-apptainer-base`: Build only the base container
- `make build-apptainer-branch`: Build branch-specific container (requires base container)
- `make clean-apptainer`: Remove all generated container files

**Advanced Examples:**
```sh
# Build container for current development branch
make build-apptainer-branch APPTAINER_BRANCH=vm-251015

# Build container from a different repository
make build-apptainer-branch APPTAINER_BRANCH=feature-branch APPTAINER_REPO=https://github.com/user/fork.git

# Build only the base container
make build-apptainer-base

# Clean up all containers
make clean-apptainer
```

## Makefile Parameters

The Apptainer build targets support the following parameters:

- `APPTAINER_BRANCH`: Git branch name to build (default: `master`)
- `APPTAINER_REPO`: Git repository URL (default: `https://github.com/Synthesys-Lab/assassyn.git`)

**Parameter Examples:**
```sh
# Override both repository and branch
make build-apptainer APPTAINER_BRANCH=feature-branch APPTAINER_REPO=https://github.com/user/fork.git

# Use default repository with custom branch
make build-apptainer APPTAINER_BRANCH=hotfix-123

# Use custom repository with default branch
make build-apptainer APPTAINER_REPO=https://github.com/user/fork.git
```

## Execute Commands

```sh
# Run Python scripts
apptainer exec --no-home assassyn-<branch-name>.sif python main.py

# Run with specific directory binding
apptainer exec --bind /path/to/your/code assassyn-<branch-name>.sif python /path/to/your/code/main.py

# Interactive shell
apptainer shell assassyn-<branch-name>.sif
```

By default, Apptainer automatically binds the `$HOME` directory to the container. Use `--no-home` to disable this binding.

## Container Files

- `assassyn-base.def`: Base container with system dependencies (Rust, Python, build tools)
- `assassyn-branch.def`: Branch-specific container that builds Assassyn code
- `scripts/init/apptainer.inc`: Makefile include with Apptainer build targets
- `assassyn-base.sif`: Generated base image (cached for reuse)
- `assassyn-<branch-name>.sif`: Generated branch-specific images

## Environment Variables and Defaults

The container system uses environment variables with default fallbacks. These can be controlled via Makefile parameters:

### Makefile Parameters
- `APPTAINER_REPO`: Git repository URL (default: `https://github.com/Synthesys-Lab/assassyn.git`)
- `APPTAINER_BRANCH`: Git branch name (default: `master`)

### Internal Container Variables
- `ASSASSYN_REPO`: Git repository URL (passed from `APPTAINER_REPO`)
- `ASSASSYN_BRANCH`: Git branch name (passed from `APPTAINER_BRANCH`)

The `${VAR:-default}` syntax provides fallback values when environment variables are not set. This allows:
- **Flexible builds**: Override repository or branch without modifying definition files
- **CI/CD integration**: Set variables in build environments
- **Local development**: Use defaults for quick testing

## Pre-configured Environment Variables

The container includes these pre-configured environment variables:
- `ASSASSYN_HOME`: Assassyn installation directory
- `VERILATOR_ROOT`: Verilator installation path
- `PYTHONPATH`: Python module search path
- `RUSTC_WRAPPER`: Rust compiler wrapper for caching
- `CC`/`CXX`: Compiler settings with ccache

## Design Purpose

This multi-stage container design addresses several key requirements:

### Branch Neutrality
The container system supports building images for any Git branch without modifying the definition files. This enables:
- **Testing**: Build containers for feature branches and pull requests
- **Release**: Build containers for stable releases and tags
- **Development**: Build containers for development branches

### Build Optimization
The two-stage approach optimizes build times:
- **Base Image**: Contains all system dependencies (Rust, Python, build tools) - built once
- **Branch Image**: Contains only the Assassyn code for a specific branch - rebuilt as needed

This separation means:
- Dependencies are cached in the base image
- Only code changes require rebuilding
- Significantly faster iteration for testing different branches

### Reproducible Environments
Each branch gets its own containerized environment ensuring:
- Consistent build environments across different systems
- Isolated dependencies per branch
- Easy deployment and testing

### CI/CD Integration
The design supports automated workflows:
- Environment variables control repository and branch selection
- Makefile targets can be easily integrated into GitHub Actions
- Containers can be built for any branch automatically
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
