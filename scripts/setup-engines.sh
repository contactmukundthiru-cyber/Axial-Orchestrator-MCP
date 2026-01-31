#!/bin/bash
set -e

WORKSPACE_ROOT=$(pwd)
TOOLS_DIR="$WORKSPACE_ROOT/tools"
mkdir -p "$TOOLS_DIR"

echo "Cloning Mandatory AXIAL Agent Engines..."

# 1. Aider
if [ ! -d "$TOOLS_DIR/aider" ]; then
    git clone https://github.com/paul-gauthier/aider.git "$TOOLS_DIR/aider"
fi

# 2. OpenHands (formerly OpenDevin)
if [ ! -d "$TOOLS_DIR/OpenHands" ]; then
    git clone https://github.com/All-Hands-AI/OpenHands.git "$TOOLS_DIR/OpenHands"
fi

# 3. LangGraph (Python)
if [ ! -d "$TOOLS_DIR/langgraph" ]; then
    git clone https://github.com/langchain-ai/langgraph.git "$TOOLS_DIR/langgraph"
fi

# 4. AutoGen
if [ ! -d "$TOOLS_DIR/autogen" ]; then
    git clone https://github.com/microsoft/autogen.git "$TOOLS_DIR/autogen"
fi

echo "All mandatory open-source engines cloned to $TOOLS_DIR"
echo "Initializing local virtual environment for agents..."
cd "$TOOLS_DIR"
python3 -m venv venv
source venv/bin/activate
pip install -e ./aider
pip install autogen langgraph
deactivate
echo "AXIAL Agent Substrate Ready."
