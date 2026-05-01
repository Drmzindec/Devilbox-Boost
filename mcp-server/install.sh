#!/usr/bin/env bash

###
### Devilbox MCP Server - Automated Installer
###
### Configures Claude Code (CLI or Desktop) to use the Devilbox MCP server
###

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEVILBOX_PATH="$(cd "$SCRIPT_DIR/.." && pwd)"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                                                            ║"
echo "║        Devilbox MCP Server Installer                      ║"
echo "║        AI-Powered Devilbox Management                     ║"
echo "║                                                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Detect Claude Desktop config path
detect_desktop_config_path() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "$HOME/Library/Application Support/Claude/claude_desktop_config.json"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "$HOME/.config/Claude/claude_desktop_config.json"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
        echo "$APPDATA/Claude/claude_desktop_config.json"
    else
        echo ""
    fi
}

# Step 1: Install dependencies
echo -e "${YELLOW}[1/4] Installing npm dependencies...${NC}"
cd "$SCRIPT_DIR"

if ! command -v npm &> /dev/null; then
    echo -e "${RED}✗ npm not found. Please install Node.js first.${NC}"
    exit 1
fi

npm install --silent
echo -e "${GREEN}✓ Dependencies installed${NC}"

# Step 2: Make executable
echo ""
echo -e "${YELLOW}[2/4] Making index.js executable...${NC}"
chmod +x "$SCRIPT_DIR/index.js"
echo -e "${GREEN}✓ Made executable${NC}"

# Step 3: Configure MCP server
echo ""
echo -e "${YELLOW}[3/4] Configuring MCP server...${NC}"

# --- Claude Code CLI: create .mcp.json in project root ---
MCP_JSON="$DEVILBOX_PATH/.mcp.json"
echo -e "${BLUE}→ Claude Code CLI: Creating $MCP_JSON${NC}"

node <<EOF
const fs = require('fs');
const path = '$MCP_JSON';

let config = {};
try {
    const content = fs.readFileSync(path, 'utf8');
    config = JSON.parse(content);
} catch (e) {
    config = {};
}

if (!config.mcpServers) {
    config.mcpServers = {};
}

config.mcpServers.devilbox = {
    command: "node",
    args: ["$SCRIPT_DIR/index.js"]
};

fs.writeFileSync(path, JSON.stringify(config, null, 2) + '\n');
EOF

echo -e "${GREEN}✓ Claude Code CLI configured (.mcp.json)${NC}"

# --- Claude Desktop app: update claude_desktop_config.json ---
DESKTOP_CONFIG=$(detect_desktop_config_path)

if [ -n "$DESKTOP_CONFIG" ]; then
    CONFIG_DIR=$(dirname "$DESKTOP_CONFIG")
    mkdir -p "$CONFIG_DIR"

    echo -e "${BLUE}→ Claude Desktop: Updating $DESKTOP_CONFIG${NC}"

    node <<EOF
const fs = require('fs');
const configPath = '$DESKTOP_CONFIG';

let config = {};
try {
    const content = fs.readFileSync(configPath, 'utf8');
    config = JSON.parse(content);
} catch (e) {
    config = {};
}

if (!config.mcpServers) {
    config.mcpServers = {};
}

config.mcpServers.devilbox = {
    command: "node",
    args: ["$SCRIPT_DIR/index.js"]
};

fs.writeFileSync(configPath, JSON.stringify(config, null, 2) + '\n');
EOF

    echo -e "${GREEN}✓ Claude Desktop configured${NC}"
else
    echo -e "${YELLOW}⚠ Could not detect Claude Desktop config path (OS: $OSTYPE)${NC}"
    echo -e "${BLUE}→ Claude Code CLI config was still created successfully${NC}"
fi

# Step 4: Verify installation
echo ""
echo -e "${YELLOW}[4/4] Verifying installation...${NC}"

if ! docker info &> /dev/null; then
    echo -e "${YELLOW}⚠ Docker is not running. Please start Docker Desktop.${NC}"
else
    echo -e "${GREEN}✓ Docker is running${NC}"
fi

cd "$DEVILBOX_PATH"
if docker compose ps 2>/dev/null | grep -q "Up\|running"; then
    echo -e "${GREEN}✓ Devilbox is running${NC}"
else
    echo -e "${YELLOW}⚠ Devilbox is not running. Start with: docker compose up -d${NC}"
fi

# Summary
echo ""
echo -e "${GREEN}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                                                            ║"
echo "║        ✓ Installation Complete!                           ║"
echo "║                                                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

echo ""
echo -e "${BLUE}What was configured:${NC}"
echo ""
echo -e "  Claude Code CLI:  ${GREEN}$MCP_JSON${NC}"
if [ -n "$DESKTOP_CONFIG" ]; then
echo -e "  Claude Desktop:   ${GREEN}$DESKTOP_CONFIG${NC}"
fi
echo -e "  Server path:      ${GREEN}$SCRIPT_DIR/index.js${NC}"

echo ""
echo -e "${BLUE}Next Steps:${NC}"
echo ""
echo -e "  1. ${YELLOW}Restart Claude Code${NC} to load the MCP server"
echo ""
echo -e "  2. Start using natural language with Claude:"
echo -e "     ${GREEN}\"What's the status of my Devilbox?\"${NC}"
echo -e "     ${GREEN}\"Show me PHP logs\"${NC}"
echo -e "     ${GREEN}\"List all databases\"${NC}"
echo -e "     ${GREEN}\"Switch to PHP 8.3\"${NC}"
echo ""
echo -e "  3. See ${BLUE}USAGE-EXAMPLES.md${NC} for more examples"
echo ""

exit 0
