#!/bin/bash
set -e

# Phase 0 Demo: Foundations & Dev Environment
echo "=== Phase 0: Foundations & Dev Environment ==="

# 1. Verify Dev Container (simulated by checking if running in container or just checking paths)
if [ -f "/.dockerenv" ]; then
    echo "✅ Running inside a container."
else
    echo "⚠️ Not running inside a container (expected for local demo)."
fi

# 2. Build AXIAL CLI
echo "Building axial-cli..."
cargo build --package axial-cli

# 3. Run Doctor
echo "Running axial doctor..."
./target/debug/axial doctor --all

# 4. Verify Monorepo Structure
echo "Verifying monorepo structure..."
crates=(
    "axial-core"
    "axial-ledger"
    "axial-router"
    "axial-shield"
    "axial-pty"
    "axial-git"
    "axial-bytebot"
    "axial-cli-harness"
    "axial-cli"
)

for crate in "${crates[@]}"; do
    if [ -d "crates/$crate" ]; then
        echo "✅ crate: $crate exists."
    else
        echo "❌ crate: $crate MISSING!"
        exit 1
    fi
done

echo "=== Phase 0 Demo Complete ==="
