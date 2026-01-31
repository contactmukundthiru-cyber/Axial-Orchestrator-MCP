# Phase 2: Router & Arbitrage Engine

## Overview
Phase 2 implements the intelligent routing layer of AXIAL. This system decides which model or tool-CLI should handle a given task based on capabilities, cost, latency, and privacy requirements.

## Deliverables

### 1. Capability Graph
- Each provider (Ollama, OpenAI, Anthropic, Cursor CLI) defines its capabilities (e.g., `code-editing`, `complex-reasoning`, `local-privacy`) with a score of 1-100.
- The `Router` uses a constraint solver (ranking for now) to select the best provider for a given set of requirements.

### 2. Provider Adapter Framework
- `Provider` trait in `axial-router` ensures a unified interface for all execution engines.
- **Local Adapters**: Ollama implemented. Goose integration scaffolded.
- **Cloud Adapters**: OpenAI and Anthropic scaffolded with full trait compliance.
- **Future-Proofing**: The same trait will be used for Phase 3's CLI-Agent adapters.

### 3. Arbitrage Engine
- Supports strategies like `privacy_first`, `performance`, and `cost_efficient`.
- Quota tracking prevents overage on expensive cloud models.

## Verification
1. `axial route --task "refactor main.rs" --explain`
2. `axial run plan_examples/multi_model.json`
