# Phase 8: Command Center UI + VS Code Embed + VSIX (MAX)

## Status: In Progress

## Objectives
- Deliver a unified, local-first UI for AXIAL via Tauri 2.0.
- Deeply integrate with VS Code through the "AXIAL Ghost" extension.
- Provide real-time monitoring and control of agents and computer-use sessions.

## UI Layout (3-Pane)
1. **Left (Command & Workforce)**: Active agents, task queue, and performance metrics.
2. **Center (Editor/Workspace)**: Embedded VS Code view or active file editor.
3. **Right (Timeline & Shield)**: Hash-chained ledger view, PTY replay, and real-time security logs.

## VS Code Embed Strategy
We use the **VS Code Server** approach:
- Launch a lightweight VS Code Server inside the devcontainer.
- Embed the web-UI into a Tauri webview.
- Communicate via the AXIAL Ghost extension for file-system and terminal synchronization.

## Features
- **Bytebot Mirror**: Real-time WebRTC stream of the agent's computer-use session with overlay takeover controls.
- **Onboarding Wizard**: runs `axial doctor` and guides the user through devcontainer and API profile setup.
