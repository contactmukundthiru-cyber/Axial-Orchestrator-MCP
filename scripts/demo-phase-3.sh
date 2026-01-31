#!/bin/bash
set -e
echo "Running Phase 3 Demo..."
./scripts/doctor.sh tools probe
./scripts/doctor.sh tools run cursor "add logging to main.rs" --dry-run
