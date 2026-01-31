#!/bin/bash
set -e
echo "Running Phase 5 Demo..."
./scripts/doctor.sh pty new "uname -a"
./scripts/doctor.sh pty new "date"
