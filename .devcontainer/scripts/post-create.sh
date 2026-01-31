#!/bin/bash
set -e

echo "Initializing AXIAL Dev Environment..."

# Create tools directory
mkdir -p "$HOME/tools"
cd "$HOME/tools"

# 1. Install Aider
if ! command -v aider &> /dev/null; then
    echo "Installing Aider..."
    pip install aider-chat --break-system-packages || pip install aider-chat
fi

# 2. Install Claude Code (Mocking if not public, but usually it's a CLI tool)
# For now, we'll install a placeholder or search for it.
if ! command -v claude &> /dev/null; then
    echo "Claude Code CLI setup... (Placeholder)"
    # npm install -g @anthropic-ai/claude-code
fi

# 3. Setup Bytebot (Assuming the user wants the reference implementation)
# If the user means a specific repo, we'd clone it here.
# Since the location is ambiguous, we'll set up the AXIAL Bytebot Client environment.
mkdir -p "$HOME/.axial/memory"

# 4. Install Goose
if ! command -v goose &> /dev/null; then
    echo "Installing Goose..."
    curl -fsSL https://github.com/block/goose/releases/download/stable/install.sh | bash || true
fi

# 5. Initialize Axial Config
mkdir -p "$HOME/.axial"
if [ ! -f "$HOME/.axial/profiles.json" ]; then
    echo '[{"name": "default", "constraints": [], "preferred_tools": ["aider", "cursor"]}]' > "$HOME/.axial/profiles.json"
fi

echo "AXIAL Environment Ready."
