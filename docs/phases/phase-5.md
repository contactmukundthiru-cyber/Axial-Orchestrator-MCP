# Phase 5: Neuro-PTY

## Overview
Neuro-PTY is AXIAL's high-fidelity terminal orchestration system. It provides zero-loss capture of all terminal interactions, time-scalable replay, and the ability to checkpoint and restore terminal states.

## Deliverables

### 1. Zero-Loss Capture
- Uses `portable-pty` for cross-platform PTY management.
- Captures all ANSI sequences, signals, and exit codes.
- Streams output to the hash-chained ledger for permanent record.

### 2. Time-Scalable Replay
- Allows the command center UI to "scrub" through terminal history.
- Re-renders ANSI state at any point in time.

### 3. Checkpoint/Restore (MAX)
- **CRIU Path**: On supported Linux systems, uses Checkpoint/Restore In Userspace (CRIU) to freeze and thaw entire process trees.
- **Fallback Path**: Captures environment variables, current working directory, and command history to "dry-restore" a session on other platforms.

### 4. Ledger Linking
- Every PTY event is a first-class entry in the AXIAL ledger, enabling deep search across command history and model outputs.

## Verification
1. `axial pty new "ls -la"`
2. `axial pty replay <session_id>`
