# Phase 7: Bytebot (MAX)

## Overview
Phase 7 introduces "Bytebot," the agent's "hands" in the OS. It provides a standardized interface for computer-use tasks, including UI automation, file manipulation beyond the workspace, and proxying actions to local or containerized environments.

## Components
1.  **Task Client**:
    -   Asynchronous client for communicating with the Bytebot service (implemented as a separate process or container).
2.  **Computer Use Proxy**:
    -   Standardized actions like `click`, `type`, `screenshot`, and `scrape`.
3.  **Harness Integration**:
    -   Allows `axial-cli` to delegate complex environmental interactions to Bytebot.

## Implementation Details
-   `BytebotClient` uses `reqwest` to interact with a REST/gRPC API.
-   Actions are logged in the `axial-ledger` for full traceability of "physical" actions taken by the AI.

## Verification
-   `axial bytebot task "open browser to github"` (mocked).
-   `axial bytebot proxy "mouse_click 100,200"` (mocked).
