# Phase 6: Neural Git (MAX)

## Overview
Phase 6 implements "Neural Git," which moves from simple version control to a structured timeline of cognitive activity. It provides the ability to fork development sessions, merge non-text artifacts (like plans or tool outputs) using JSON patches, and links every commit to PTY sessions and ledger events.

## Components
1.  **Session Forking**:
    -   `axial git fork <id>`: Creates a new project branch representing a specific cognitive branch/thought process.
2.  **Structured Merging**:
    -   Leverages `json-patch` to merge `PlanPacket` and `TaskGraph` updates deterministically.
3.  **Cross-Linking**:
    -   Each commit message contains metadata linking back to the `axial-ledger` entry and the `axial-pty` session segment.

## Implementation Details
-   `GitManager` wraps `libgit2` (via `git2` crate) for high-performance repo operations.
-   Handles branching strategies for parallel agent tasking.
-   Integrates with `axial-ledger` to ensure the hash chain includes git commit SHAs.

## Verification
-   Run `axial git fork test-session`.
-   Verify structured merge logic via CLI.
-   Check ledger connectivity records.
