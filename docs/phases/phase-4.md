# Phase 4: AXIAL Shield Boundary

## Overview
AXIAL Shield is the security boundary that enforces outbound policies and protects sensitive data. It ensures that no PII, API keys, or unauthorized data leaves the local environment without explicit approval or redaction.

## Deliverables

### 1. Outbound Boundary Enforcement
- All adapters (Router, CLI Harness, Bytebot) must route outbound requests through the Shield.
- `validate_request(domain)`: Checks against a strict allowlist.

### 2. Redaction Pipeline
- Multi-stage pipeline:
  - **Regex Pass**: Fast identification of known patterns (SSNs, API keys, emails).
  - **Local Model Pass (Planned)**: Uses a small local model to identify semantic PII that regex misses.
- Customizable placeholders for redacted data.

### 3. Kill Switch Primitives
- Immediate termination of all outbound traffic if a breach is detected or manual kill switch is triggered.

### 4. Audit Reporting
- Every redaction and blocked request is logged to the ledger with a "security" tag.
- Generates periodic audit reports for compliance.

## Verification
1. `axial shield scan --input "My key is sk-12345678901234567890123456789012"`
2. `axial shield check google.com` (should fail)
