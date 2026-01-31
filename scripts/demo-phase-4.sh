#!/bin/bash
set -e
echo "Running Phase 4 Demo..."
./scripts/doctor.sh shield scan --input "Contact me at 123-45-6789 or sk-abcdefghijklmnopqrstuvwxyz012345"
./scripts/doctor.sh shield check api.openai.com
./scripts/doctor.sh shield check malicious-site.com
