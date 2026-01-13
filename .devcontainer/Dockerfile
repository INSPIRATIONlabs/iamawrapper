# Development container for imawrapper
# Lightweight Rust development environment

FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Install useful utilities for development
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        # Useful utilities
        unzip \
        p7zip-full \
        xxd \
        bsdmainutils \
        # For XML validation
        libxml2-utils \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install additional Rust components
RUN rustup component add clippy rustfmt \
    && rustup target add x86_64-unknown-linux-gnu \
    && rustup target add aarch64-unknown-linux-gnu \
    && cargo install cargo-audit cargo-watch

# Set up workspace
WORKDIR /workspaces/iamawrapper
