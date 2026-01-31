# Phase 3: CLI Harness

## Overview
Phase 3 integrates best-in-class IDE agents (Cursor, Codex, Claude Code, Aider) as first-class citizens in the AXIAL ecosystem. Instead of just calling an LLM, AXIAL can delegate repo-wide edits to specialized CLI tools.

## Deliverables

### 1. Unified Tool Interface
- `ToolAdapter` trait in `axial-cli-harness` normalizes disparate CLI interfaces.
- `probe()`: Automates tool discovery and auth state check.
- `run()`: Executes complex multi-file edits with structured results.

### 2. High-Fidelity Adapters
- **Cursor CLI**: Supports `repo-wide` context and `patch` application.
- **Codex CLI**: Specialized for targeted generation and test writing.
- **Scaffolded**: Adapters for Aider, Claude Code, and Cline are present and ready for extension.

### 3. Arbitrage Integration
- The Router can now "prefer" a CLI tool over a raw LLM call for tasks like "multi-file refactor".
- Tool execution is logged to the ledger, including diffs and transcripts.

## Verification
1. `axial tools probe`
2. `axial tools run cursor --task "implement a logger" --dry-run`
