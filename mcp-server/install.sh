#!/bin/bash

###
### Devilbox MCP Server - Automated Installer
###
### Automatically configures Claude Code to use the Devilbox MCP server
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

# Detect OS and find Claude Code config
detect_config_path() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        echo "$HOME/Library/Application Support/Claude/claude_desktop_config.json"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        echo "$HOME/.config/Claude/claude_desktop_config.json"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
        # Windows
        echo "$APPDATA/Claude/claude_desktop_config.json"
    else
        echo ""
    fi
}

CONFIG_PATH=$(detect_config_path)

if [ -z "$CONFIG_PATH" ]; then
    echo -e "${RED}✗ Could not detect Claude Code configuration path for OS: $OSTYPE${NC}"
    exit 1
fi

echo -e "${BLUE}→ Detected Claude Code config: $CONFIG_PATH${NC}"

# Step 1: Install dependencies
echo ""
echo -e "${YELLOW}[1/4] Installing npm dependencies...${NC}"
cd "$SCRIPT_DIR"

if ! command -v npm &> /dev/null; then
    echo -e "${RED}✗ npm not found. Please install Node.js first.${NC}"
    exit 1
fi

npm install --silent

echo -e "${GREEN}✓ Dependencies installed (91 packages)${NC}"

# Step 2: Make executable
echo ""
echo -e "${YELLOW}[2/4] Making index.js executable...${NC}"
chmod +x "$SCRIPT_DIR/index.js"
echo -e "${GREEN}✓ Made executable${NC}"

# Step 3: Create/update Claude Code config
echo ""
echo -e "${YELLOW}[3/4] Configuring Claude Code...${NC}"

# Create config directory if it doesn't exist
CONFIG_DIR=$(dirname "$CONFIG_PATH")
mkdir -p "$CONFIG_DIR"

# Check if config file exists
if [ ! -f "$CONFIG_PATH" ]; then
    echo -e "${BLUE}→ Creating new Claude Code configuration${NC}"
    cat > "$CONFIG_PATH" <<EOF
{
  "mcpServers": {
    "devilbox": {
      "command": "node",
      "args": [
        "$SCRIPT_DIR/index.js"
      ]
    }
  }
}
EOF
    echo -e "${GREEN}✓ Created new configuration${NC}"
else
    echo -e "${BLUE}→ Updating existing Claude Code configuration${NC}"

    # Check if devilbox entry already exists
    if grep -q '"devilbox"' "$CONFIG_PATH" 2>/dev/null; then
        echo -e "${YELLOW}! Devilbox MCP server already configured${NC}"
        echo -e "${BLUE}→ Updating path to: $SCRIPT_DIR/index.js${NC}"
    fi

    # Use Node.js to safely update JSON
    node <<EOF
const fs = require('fs');
const configPath = '$CONFIG_PATH';

let config = {};
try {
    const content = fs.readFileSync(configPath, 'utf8');
    config = JSON.parse(content);
} catch (e) {
    // File exists but might be empty or invalid JSON
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
console.log('Configuration updated successfully');
EOF

    echo -e "${GREEN}✓ Configuration updated${NC}"
fi

# Step 4: Verify installation
echo ""
echo -e "${YELLOW}[4/4] Verifying installation...${NC}"

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo -e "${YELLOW}⚠ Docker is not running. Please start Docker Desktop.${NC}"
else
    echo -e "${GREEN}✓ Docker is running${NC}"
fi

# Check if Devilbox is running
cd "$DEVILBOX_PATH"
if docker-compose ps | grep -q "Up"; then
    echo -e "${GREEN}✓ Devilbox is running${NC}"
else
    echo -e "${YELLOW}⚠ Devilbox is not running. Start with: docker-compose up -d${NC}"
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
echo -e "${BLUE}Configuration:${NC}"
echo -e "  MCP Config: ${GREEN}$CONFIG_PATH${NC}"
echo -e "  Server Path: ${GREEN}$SCRIPT_DIR/index.js${NC}"
echo ""
echo -e "${BLUE}Troubleshooting:${NC}"
echo -e "  If Claude Code doesn't detect the server:"
echo -e "  - Ensure Claude Code is completely restarted"
echo -e "  - Check config file: ${YELLOW}cat \"$CONFIG_PATH\"${NC}"
echo -e "  - View logs in Claude Code's MCP settings"
echo ""

exit 0
