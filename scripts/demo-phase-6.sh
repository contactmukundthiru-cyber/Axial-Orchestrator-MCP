#!/bin/bash
set -e

echo "--- AXIAL PHASE 6 DEMO: NEURAL GIT ---"

# 1. Initialize a git repo if not present
if [ ! -d ".git" ]; then
    git init
    git config user.email "axial@example.com"
    git config user.name "AXIAL Agent"
    git commit --allow-empty -m "Initial commit"
fi

# 2. Fork a session
echo "Testing session fork..."
cargo run --bin axial -- git fork "session-alpha-123"

# 3. Test structured merge
echo "Testing JSON patch merge..."
BASE_JSON='{"status": "pending", "nodes": []}'
PATCH_JSON='[{"op": "replace", "path": "/status", "value": "completed"}]'

# We escape the JSON for the shell
cargo run --bin axial -- git merge "$BASE_JSON" "$PATCH_JSON"

echo "Phase 6 Verification Complete."
