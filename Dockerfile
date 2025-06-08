# Use the Rust image as the base
FROM rust:1.82

# Install system packages and set up Python symlink
RUN apt-get update && apt-get install -y --no-install-recommends \
    zsh \
    python3-pip \
    python-is-python3 \
    git \
    autoconf \
    g++ \
    flex \
    bison \
    libfl2 \
    libfl-dev \
    libexpat1-dev \
    gettext \
    make \
    perl \
    ccache \
    libgoogle-perftools-dev \
    numactl \
    perl-doc \
    help2man \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Set default shell to zsh
SHELL ["/bin/zsh", "-c"]
# You can use the following command instead
# RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

# Install Python packages, Cargo tools, and Rust components
RUN pip install decorator==5.1.1 pytest==7.4.3 pylint==3.2.3 pytest-xdist==3.6.1 --break-system-packages

# Set environment variables
ENV CC="ccache gcc"
ENV CXX="ccache g++"
ENV PYTHONPATH="/app/python"
ENV ASSASSYN_HOME="/app"
# ENV VERILATOR_ROOT="/usr/local/share/verilator"

# Set working directory
WORKDIR /app

# Clone, build, and install Verilator, then clean up
# RUN set -eux \
#     && git clone https://github.com/verilator/verilator.git /app/verilator \
#     && cd /app/verilator \
#     && git checkout ca4858eb7f6142a0da367e0c299762d0922f1a6c \
#     && autoconf \
#     && ./configure \
#     && make -j$(nproc) \
#     && make install \
#     && verilator --version \
#     && rm -rf /app/verilator

# Ensure setup.sh is sourced on shell startup if it exists
RUN echo '[ -f /app/setup.sh ] && source /app/setup.sh --no-verilator' >> /root/.zshrc

# Define the default command
CMD ["/bin/zsh"]
