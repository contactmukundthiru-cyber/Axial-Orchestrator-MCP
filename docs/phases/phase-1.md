# Phase 1: Schemas & Ledger

## Overview
Phase 1 implements the data core of AXIAL. This includes versioned JSON schemas for orchestration and a hash-chained ledger that ensures all actions are deterministic, verifiable, and permanent.

## Deliverables

### 1. Versioned Schemas
Located in `axial-core`, these schemas define the "Task Algebra" used by the command center:
- `PlanPacket`: The top-level execution plan.
- `TaskGraph`: A directed graph of tasks.
- `Invariant`: Constraints that must hold during or after execution.
- `ApprovalGate`: Manual intervention points.
- `Artifact`: Structured outputs.

### 2. Hash-Chained Ledger
Located in `axial-ledger`:
- **JSONL Sink**: All entries are written to a linear JSONL file for easy parsing and recovery.
- **SQLite Index**: Provides fast query capabilities for the UI and CLI.
- **SHA-256 Chaining**: Each entry contains the hash of the index, previous hash, payload, and timestamp.
- **Verification**: `axial ledger verify` checks the integrity of the entire chain.

### 3. Runpack
- A self-contained export of a run, including ledger segments and artifacts.

## Verification
1. `axial run --demo plan_examples/hello.json`
2. `axial ledger verify`
