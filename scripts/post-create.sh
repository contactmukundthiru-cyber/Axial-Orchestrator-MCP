#!/bin/bash
set -e

echo "Starting AXIAL post-create setup..."

# Install dependencies for apps/axial-command-center if it exists
if [ -d "apps/axial-command-center" ]; then
    cd apps/axial-command-center && npm install && cd ../..
fi

# Build axial-cli
cargo build --package axial-cli

# Run doctor
./target/debug/axial doctor --all
