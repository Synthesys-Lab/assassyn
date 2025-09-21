# Docker

It is also recommended to use [Docker](https://www.docker.com/) to
automatically manage the dependences. We decide to adopt a hybrid style
of coding, tooling, and development, where this repo is located
in your _physical_ machine, while the execution is in the docker _virtual_
machine (VM).

Still, before doing anything, make sure you have this repo fully initialized:

````sh
git submodule update --init --recursive
````

## Build the Docker Image

Assuming you are at the root of this source tree:

```sh
docker build -t assassyn:latest .
```

## Run the Docker Container

At least read point 3 below before typing your command!!!

1. `-v <src>:<dst>` mounts a physical source directory into the VM destination.
2. `--name` specifies the name of this VM container, which avoids a random `<adj>_<noun>` name.
3. `-m` specifies the memory limit of a container. However, this flag only works on Linux.
    - If you are using Docker client on Windows or Mac OS with UI, memory should be tuned in the client.
    - Click the bottom bar of the client <img src="./imag/resource-bar.png" width=75%>.
    - A resource slide bar will pop up as shown <img src="./imag/resource-bar.png" width=75%>.
    - Tune the memory usage **before** starting your VM container.

```sh
docker run --rm -tid -v `pwd`:/app --user $(id -u):$(id -g) \
  -m 32g \
  --name assassyn assassyn:latest
```

## Build Initialization

If it is the first time, the repo should be initialized using the command below.

```sh
docker exec -it ./init.sh
```

LLVM linkage is super memory consuming, while `ninja` build can only use fixed #thread
parallelization, which may overwhelm the memory. Feel free to tune your own parameters to have
a balance between performance and machine limit. If you see something like `g++ is killed Signal 9`
or `truncated file`, with high probability it is caused by out of memory (OOM).
Feel free to give a different parameter and run again.