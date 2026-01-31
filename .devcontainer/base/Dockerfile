FROM ubuntu:24.04

# Non-interactive frontend for apt
ENV DEBIAN_FRONTEND=noninteractive

# Install core build tools and dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    libssl-dev \
    pkg-config \
    ca-certificates \
    software-properties-common \
    sqlite3 \
    libsqlite3-dev \
    # Tauri build dependencies
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libvips-dev \
    # PTY / CRIU / Systems deps
    criu \
    telnet \
    clang \
    # Node.js build deps
    python3 \
    make \
    g++ \
    # WebRTC build deps
    libsoup-3.0-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install Rust stable pinned
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
    && rustup component add rustfmt clippy

# Install Node LTS (pinned version)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g npm@latest

# Create a non-root user
ARG USERNAME=axial
ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && apt-get update \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME

USER $USERNAME
WORKDIR /workspace
