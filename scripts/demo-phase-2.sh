#!/bin/bash
set -e
echo "Running Phase 2 Demo..."
./scripts/doctor.sh route --task "refactor main.rs" --explain
./scripts/doctor.sh route --task "simple echo" --strategy privacy_first --explain
