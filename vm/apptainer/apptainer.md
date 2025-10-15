# Assassyn Apptainer Container

Assassyn can be run using [Apptainer](https://apptainer.org/) (formerly Singularity) for containerized execution. This approach provides a portable, reproducible environment with all dependencies pre-installed.

## Quick Start

From the `vm/apptainer` directory:

```sh
# Build for a specific branch
./build-assassyn.sh <branch-name>

# Build for main branch (default)
./build-assassyn.sh
```

This creates a containerized Assassyn environment for the specified branch.

## Manual Build Process

If you prefer manual control over the build process:

```sh
# Step 1: Build base image (one-time setup)
apptainer build assassyn-base.sif assassyn-base.def

# Step 2: Build branch-specific image
apptainer build --build-arg ASSASSYN_BRANCH=<branch-name> assassyn-<branch-name>.sif assassyn-branch.def
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
- `build-assassyn.sh`: Automated build script
- `assassyn-base.sif`: Generated base image (cached for reuse)
- `assassyn-<branch-name>.sif`: Generated branch-specific images

## Environment Variables and Defaults

The container system uses environment variables with default fallbacks:

- `ASSASSYN_REPO`: Git repository URL (default: `https://github.com/Synthesys-Lab/assassyn.git`)
- `ASSASSYN_BRANCH`: Git branch name (default: `main`)

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
- Build scripts can be easily integrated into GitHub Actions
- Containers can be built for any branch automatically

### Usage Flexibility
The container system supports multiple use cases:
- **Development**: Interactive shells for development work
- **Testing**: Automated test execution in isolated environments
- **Deployment**: Consistent runtime environments across different systems
- **Distribution**: Portable environments that work across different platforms
