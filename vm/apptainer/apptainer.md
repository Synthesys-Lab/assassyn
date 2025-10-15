# Assassyn Apptainer Container

Assassyn can be run using [Apptainer](https://apptainer.org/) (formerly Singularity) for containerized execution. This approach provides a portable, reproducible environment that clones the Assassyn repository from a specified URL and tag with all dependencies pre-installed.

## Quick Start

From the project root directory:

```sh
# Build container with default repository (https://github.com/Synthesys-Lab/assassyn at master)
make build-apptainer

# Build container with custom repository and tag
make build-apptainer REPO_URL=https://github.com/your-fork/assassyn REPO_TAG=your-branch

# Clean all containers
make clean-apptainer
```

This creates a containerized environment by cloning the specified Assassyn repository and tag.

## Available Makefile Targets

The Apptainer build system provides several targets for different use cases:

### Primary Target
- `make build-apptainer`: Build container with specified repository and tag

### Additional Targets
- `make build-apptainer-base`: Build only the base container (system dependencies)
- `make clean-apptainer`: Remove all generated container files

### Repository Configuration
The build system accepts two optional parameters:
- `REPO_URL`: Repository URL (default: `https://github.com/Synthesys-Lab/assassyn`)
- `REPO_TAG`: Git tag or branch (default: `master`)

**Examples:**
```sh
# Build container with default repository (master branch)
make build-apptainer

# Build container with specific branch
make build-apptainer REPO_TAG=develop

# Build container with custom fork
make build-apptainer REPO_URL=https://github.com/your-fork/assassyn REPO_TAG=feature-branch

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
- `assassyn.def`: Main container definition that clones the specified repository
- `scripts/init/apptainer.inc`: Makefile include with Apptainer build targets
- `assassyn-base.sif`: Generated base image (shared across all builds)
- `assassyn.sif`: Generated container image (contains cloned repository)

## Environment Variables and Defaults

The container system clones the specified repository and includes these pre-configured environment variables:
- `ASSASSYN_HOME`: Assassyn installation directory
- `VERILATOR_ROOT`: Verilator installation path
- `PYTHONPATH`: Python module search path
- `RUSTC_WRAPPER`: Rust compiler wrapper for caching
- `CC`/`CXX`: Compiler settings with ccache

## Design Purpose

This multi-stage container design addresses several key requirements:

### Repository Cloning
The container system clones the specified repository and tag, enabling:
- **Flexible Source Control**: Use any repository URL and tag/branch combination
- **Version Control**: Container reflects the exact state of the specified repository tag
- **Portability**: Move containers between systems with identical behavior
- **Fork Support**: Easy testing of forks and custom branches

### Build Optimization
The two-stage approach optimizes build times:
- **Base Image**: Contains all system dependencies (Rust, Python, build tools) - shared across all builds
- **Repository Image**: Contains only the Assassyn code from the specified repository - rebuilt as needed

This separation means:
- Dependencies are cached in the base image
- Only repository changes require rebuilding
- Significantly faster iteration for development

### Reproducible Environments
Each container build creates a reproducible environment ensuring:
- Consistent build environments across different systems
- Isolated dependencies per repository state
- Easy deployment and testing
- Deterministic builds from any repository tag

### CI/CD Integration
The design supports automated workflows:
- Repository URL and tag parameters control source code
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
