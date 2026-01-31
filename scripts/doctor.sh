#!/bin/bash
set -e
cargo run --package axial-cli -- doctor "$@"
