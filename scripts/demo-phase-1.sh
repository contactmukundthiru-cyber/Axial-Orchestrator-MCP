#!/bin/bash
set -e

# Phase 1 Demo: Schemas & Ledger
echo "=== Phase 1: Schemas & Ledger ==="

# 1. Build
echo "Building project..."
cargo build --package axial-cli

# 2. Add entries via axial run --demo
echo "Running demo plan..."
./target/debug/axial run --demo plan_examples/hello.json --local-only

# 3. Verify Integrity
echo "Verifying ledger integrity..."
./target/debug/axial ledger verify

# 4. Export Runpack
echo "Exporting runpack..."
rm -rf demo_runpack
./target/debug/axial ledger export --id demo

echo "=== Phase 1 Demo Complete ==="
