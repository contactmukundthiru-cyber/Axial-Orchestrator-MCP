# Phase 0: Foundations & Dev Environment

## Overview
Phase 0 establishes the AXIAL monorepo structure, build system, and developer environment using Dev Containers. This ensures all developers and CI systems have a consistent, "v1-max" foundation.

## Deliverables

### 1. Monorepo Structure
- Rust workspace for core logic, PTY, Git, and orchestration.
- `apps/` for UI and VS Code extension.
- `.devcontainer/` for multi-profile development.
- `scripts/` for automation and maintenance.

### 2. Multi-Profile Dev Containers
- **Base**: Ubuntu 24.04, Rust, Node.js, Tauri deps, Git, SQLite, CRIU.
- **Docker**: Inherits Base, adds Docker-in-Docker support for containerized tools (Bytebot).
- **GPU**: Inherits Base, adds NVIDIA Container Toolkit for local LLM acceleration.

### 3. AXIAL CLI (Initial)
- `axial doctor`: Robust environment validation.
- Verifies: Rust, Node, Docker, CRIU, Tauri deps, Ollama.

### 4. CI/CD
- GitHub Actions workflow for linting, testing, and devcontainer smoke checks.

## Verification
1. Run `scripts/post-create.sh` inside the devcontainer.
2. Run `axial doctor --all` to verify the environment.
3. `cargo test --workspace` should pass.
