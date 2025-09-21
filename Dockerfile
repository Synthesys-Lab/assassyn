# Use the Rust image as the base
FROM rust:1.82

# Install system packages and set up Python symlink
RUN apt-get update && apt-get install -y --no-install-recommends \
    zsh \
    python3-pip \
    python3-dev \
    python3-venv \
    pybind11-dev \
    python-is-python3 \
    git \
    autoconf \
    build-essential \
    flex \
    bison \
    libfl2 \
    libfl-dev \
    libexpat1-dev \
    gettext \
    perl \
    ccache \
    libgoogle-perftools-dev \
    numactl \
    perl-doc \
    help2man \
    cmake \
    ninja-build \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Set default shell to zsh
SHELL ["/bin/zsh", "-c"]

# You can use the following command instead
RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

# Set build environment variables
ENV CC="ccache gcc"
ENV CXX="ccache g++"
ENV CCACHE_DIR="/tmp/ccache"
ENV PYTHONUSERBASE="/tmp/.local"

# Set up Assassyn related environment variables
ENV ASSASSYN_HOME="/app"
ENV VERILATOR_ROOT="/app/3rd-party/verilator"
ENV PYTHONPATH="/app/python"

# Set working directory
WORKDIR /app

# Ensure setup.sh is sourced on shell startup if it exists
RUN echo '[ -f /app/setup.sh ] && source /app/setup.sh' >> /root/.zshrc

# Define the default command
CMD ["/bin/zsh"]
