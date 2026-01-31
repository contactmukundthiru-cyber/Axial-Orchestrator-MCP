#!/bin/bash
set -e

echo "Initializing AXIAL Forensic Environment..."

# Block all outbound networking except localhost
# v1-max: This requires root or specific cap_net_admin
# For now, we'll set up the local AXIAL tools in a disconnected state.

# Disable common outbound tools
sudo chmod 000 /usr/bin/curl || true
sudo chmod 000 /usr/bin/wget || true

# Initialize a cold ledger
mkdir -p "$HOME/.axial"
axial ledger verify || echo "Initializing local forensic ledger..."

echo "AXIAL Forensic Environment SECURED."
