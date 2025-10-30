

Assassyn can be run using [Apptainer](https://apptainer.org/) (formerly Singularity) for containerized execution. This approach provides a portable, reproducible environment with all dependencies pre-installed.

From the root of the Assassyn repository:

```sh
apptainer build assassyn.sif assassyn.def
```

This will create a Singularity Image File (`.sif`) containing the complete Assassyn development environment.


### Execute Commands
```sh
apptainer exec --no-home assassyn.sif python main.py
```
By default, Apptainer automatically binds the $HOME directory to the container. `--no-home` disables this binding. 

Use `--bind` to bind a specific directory to the container.

## Notes

- The container is based on Rust 1.82 and includes all necessary dependencies
- The environment is pre-configured with all required environment variables
