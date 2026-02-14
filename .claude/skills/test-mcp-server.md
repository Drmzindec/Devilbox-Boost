# Test MCP Server

Test the Devilbox MCP server functionality to ensure all tools work correctly.

## Usage

```bash
cd mcp-server
node test-mcp.js
```

## What It Tests

1. **Server Initialization** - MCP protocol handshake
2. **Tools List** - Verifies all 10 tools are available
3. **devilbox_status** - Container status detection
4. **devilbox_vhosts** - Project listing
5. **Command execution** - Docker operations
6. **Database queries** - MySQL connectivity

## Expected Output

```
Testing Devilbox MCP Server...

→ Test 1: Initializing server...
→ Test 2: Listing tools...
→ Test 3: Calling devilbox_status...
→ Test 4: Calling devilbox_vhosts...

============================================================
Test Results:
============================================================
✓ Initialization: OK
✓ Tools list: 10 tools found
  Tools: devilbox_status, devilbox_start, devilbox_stop, devilbox_restart, devilbox_logs, devilbox_exec, devilbox_vhosts, devilbox_config, devilbox_databases, devilbox_health
✓ devilbox_status: Working
  Containers detected: 8
✓ devilbox_vhosts: Working
  Projects detected: 13

============================================================
Summary:
============================================================
✅ All tests passed!
✅ MCP server is working correctly
```

## Installation Test

```bash
# Remove existing configuration
node -e "
const fs = require('fs');
const path = '~/Library/Application Support/Claude/claude_desktop_config.json';
const config = JSON.parse(fs.readFileSync(path, 'utf8'));
delete config.mcpServers.devilbox;
fs.writeFileSync(path, JSON.stringify(config, null, 2));
"

# Run installer
cd mcp-server
./install.sh

# Verify configuration
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json | jq '.mcpServers.devilbox'
```

## Manual Testing with Claude Code

After installation and restarting Claude Code, test natural language:

1. "What services are running on my Devilbox?"
2. "Show me the last 50 PHP logs"
3. "List all databases"
4. "What projects do I have?"
5. "Is Devilbox healthy?"

## Troubleshooting

**Tests fail:**
- Ensure Docker is running: `docker info`
- Start Devilbox: `docker-compose up -d`
- Check container names: `docker ps`

**MCP not detected in Claude Code:**
- Verify config file path
- Completely restart Claude Code
- Check installer output for errors

**Tool returns errors:**
- Check Docker Compose version
- Verify container naming (devilbox-php-1, etc.)
- Check file permissions on index.js
