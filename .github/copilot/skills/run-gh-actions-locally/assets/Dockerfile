# Custom Docker image for running VstKit CI locally with `act`
# Build: docker build -t vstkit-ci:latest .github/act/
# Use:   act -P ubuntu-latest=vstkit-ci:latest --container-architecture linux/amd64

FROM ubuntu:22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install all system dependencies, Node.js, and Rust in optimized layers
RUN apt-get update && apt-get install -y \
    build-essential \
    ca-certificates \
    curl \
    git \
    gnupg \
    libayatana-appindicator3-dev \
    libgtk-3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    libx11-xcb-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxdo-dev \
    pkg-config \
    sudo \
    wget \
    && curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && /root/.cargo/bin/rustup component add rustfmt clippy

# Add cargo to PATH for all users
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify installations
RUN node --version && npm --version && rustc --version && cargo --version

# Set working directory
WORKDIR /workspace

# Label for documentation
LABEL org.opencontainers.image.source="https://github.com/RonHouben/vstkit"
LABEL org.opencontainers.image.description="CI image for VstKit local testing with act"
