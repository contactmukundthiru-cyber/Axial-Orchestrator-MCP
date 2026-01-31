# AXIAL Developer Guide

## Prerequisite
- Docker + VS Code Dev Containers extension.

## Setup
1. Clone the repo.
2. Open in VS Code.
3. Choose a profile when prompted:
   - **Profile A (Base)**: Core development.
   - **Profile B (Docker)**: With Docker support.
   - **Profile C (GPU)**: With NVIDIA GPU support.

## Running Axial
```bash
# Run doctor
cargo run --package axial-cli -- doctor --all

# Build workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run Phase 0 Demo
./scripts/demo-phase-0.sh
```

## Structure
- `crates/`: Rust workspace members (Core, Ledger, Router, Shield, PTY, Git, Bytebot, Tool Harness).
- `apps/`: UI (Tauri) and VS Code extension (Ghost).
- `docs/phases/`: Detailed implementation plans per subsystem.
