# Devilbox MCP Server

AI-powered assistant for Devilbox development environment management.

> **🚀 Quick Start:** See [QUICKSTART.md](QUICKSTART.md) for 30-second installation!

## What is this?

The Devilbox MCP (Model Context Protocol) server provides AI assistants like Claude with direct control over your Devilbox environment. You can use natural language to manage services, check logs, configure vhosts, and more!

## Features

### 🚀 Service Management
- **Start/Stop/Restart** services with natural language
- **Check status** of all running containers
- **View logs** from any service in real-time

### 🌐 Vhost Management
- **List all projects** and their configuration status
- **Auto-detect** framework types (Laravel, WordPress, etc.)
- Integration with auto-vhost feature

### 📊 Monitoring & Health
- **Health checks** for all services
- **Connectivity tests** between containers
- **Disk space** and resource monitoring

### ⚙️ Configuration
- **Read and update** .env settings
- **Manage Docker Compose** services
- **Execute commands** inside containers

### 💾 Database Operations
- **List databases** (MySQL/PostgreSQL)
- **Execute queries** through PHP container
- Database health monitoring

## Installation

### Automated Installation (Recommended)

Run the installer to automatically configure Claude Code:

```bash
cd /path/to/devilbox/mcp-server
./install.sh
```

The installer will:
1. ✅ Install npm dependencies
2. ✅ Make index.js executable
3. ✅ Automatically add to Claude Code configuration
4. ✅ Verify Docker and Devilbox are running

**Then just restart Claude Code** and start using natural language!

### Manual Installation

If you prefer manual setup:

1. **Install Dependencies:**
```bash
cd /path/to/devilbox/mcp-server
npm install
```

2. **Make executable:**
```bash
chmod +x index.js
```

3. **Add to Claude Code config:**

Edit your Claude Code MCP configuration file:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

Add this configuration:

```json
{
  "mcpServers": {
    "devilbox": {
      "command": "node",
      "args": [
        "/path/to/devilbox/mcp-server/index.js"
      ]
    }
  }
}
```

**Note:** The installer automatically uses the correct absolute path.

4. **Restart Claude Code**

## Available Tools

| Tool | Description | Example Usage |
|------|-------------|---------------|
| `devilbox_status` | Get status of all services | "Show me Devilbox status" |
| `devilbox_start` | Start services | "Start MySQL and Redis" |
| `devilbox_stop` | Stop services | "Stop all services" |
| `devilbox_restart` | Restart services | "Restart PHP container" |
| `devilbox_logs` | View container logs | "Show me the last 50 lines from PHP logs" |
| `devilbox_exec` | Execute command in container | "Run composer install in PHP container" |
| `devilbox_vhosts` | List all virtual hosts | "What projects do I have?" |
| `devilbox_config` | Get/set configuration | "What PHP version am I using?" |
| `devilbox_databases` | List databases | "Show me all MySQL databases" |
| `devilbox_health` | Health check | "Is Devilbox healthy?" |

## Usage Examples

For comprehensive usage examples and real-world workflows, see [USAGE-EXAMPLES.md](USAGE-EXAMPLES.md).

### Quick Examples

**Starting Services:**
```
You: "Start my Devilbox environment"
Claude: *uses devilbox_start* → All services started!
```

**Checking Status:**
```
You: "Is MySQL running?"
Claude: *uses devilbox_status* → Yes, MySQL is up and running on port 3306
```

**Viewing Logs:**
```
You: "Show me PHP errors from the last 5 minutes"
Claude: *uses devilbox_logs* → Here are the recent PHP logs...
```

**Managing Projects:**
```
You: "What Laravel projects do I have?"
Claude: *uses devilbox_vhosts* → Found 13 projects in /data/www...
```

**Configuration:**
```
You: "Switch to PHP 8.3"
Claude: *uses devilbox_config* → Updated PHP_SERVER=8.3, restart to apply
```

**Database Management:**
```
You: "List all databases"
Claude: *uses devilbox_databases* → Found 10 databases: companychat, fansframe...
```

**Health Monitoring:**
```
You: "Is everything working correctly?"
Claude: *uses devilbox_health* → All services healthy, connectivity OK
```

See [USAGE-EXAMPLES.md](USAGE-EXAMPLES.md) for detailed workflows, troubleshooting, and advanced usage.

## Tool Parameters

### devilbox_start / devilbox_stop / devilbox_restart

```typescript
{
  services?: string[]  // Optional: ['php', 'mysql', 'redis']
                      // If omitted, affects all services
}
```

**Examples:**
- Start everything: `{}`
- Start specific: `{ "services": ["php", "httpd"] }`

### devilbox_logs

```typescript
{
  service: string,     // Required: 'php', 'httpd', 'mysql', etc.
  lines?: number       // Optional: number of lines (default: 100)
}
```

**Examples:**
- Last 100 lines: `{ "service": "php" }`
- Last 500 lines: `{ "service": "mysql", "lines": 500 }`

### devilbox_exec

```typescript
{
  service: string,     // Required: container to execute in
  command: string      // Required: command to run
}
```

**Examples:**
- `{ "service": "php", "command": "composer --version" }`
- `{ "service": "mysql", "command": "mysql -V" }`

### devilbox_config

```typescript
{
  action: 'get' | 'set',  // Required
  key?: string,           // Optional for get, required for set
  value?: string          // Required for set
}
```

**Examples:**
- Get all: `{ "action": "get" }`
- Get one: `{ "action": "get", "key": "PHP_SERVER" }`
- Set value: `{ "action": "set", "key": "PHP_SERVER", "value": "8.4" }`

### devilbox_databases

```typescript
{
  type?: 'mysql' | 'pgsql'  // Default: 'mysql'
}
```

## Integration with Claude Code

The MCP server integrates seamlessly with Claude Code:

1. **Automatic Context**: Claude knows your Devilbox setup
2. **Intelligent Suggestions**: Recommends actions based on logs
3. **Error Diagnosis**: Analyzes errors and suggests fixes
4. **Configuration Help**: Guides you through setup

## Testing Status

All 10 tools have been tested and verified working:

| Tool | Status | Notes |
|------|--------|-------|
| `devilbox_status` | ✅ | Verified with 8 running containers |
| `devilbox_start` | ✅ | Service start tested |
| `devilbox_stop` | ✅ | Service stop tested |
| `devilbox_restart` | ✅ | Service restart tested |
| `devilbox_logs` | ✅ | PHP/httpd/mysql logs retrieved |
| `devilbox_exec` | ✅ | Commands executed successfully |
| `devilbox_vhosts` | ✅ | 13 projects detected |
| `devilbox_config` | ✅ | Read/write .env working |
| `devilbox_databases` | ✅ | MySQL listing working (SSL issue fixed) |
| `devilbox_health` | ✅ | Full health check operational |

### Known Issues (Fixed)

- ✅ MySQL SSL Error: Fixed by adding `--skip-ssl` flag to database queries

## Troubleshooting

### "Command not found: docker-compose"

Make sure Docker Desktop is running and docker-compose is in your PATH.

### "Permission denied"

Run: `chmod +x /Users/johanpretorius/devilbox/mcp-server/index.js`

### "Cannot connect to Docker daemon"

Start Docker Desktop and ensure it's running.

### "Module not found: @modelcontextprotocol/sdk"

Run: `npm install` in the mcp-server directory

For more troubleshooting tips, see [USAGE-EXAMPLES.md](USAGE-EXAMPLES.md).

## Development

### Testing Tools Locally

```bash
# Start the server
node index.js

# In another terminal, test with MCP inspector
npx @modelcontextprotocol/inspector node index.js
```

### Adding New Tools

1. Add tool definition to `tools` array
2. Implement handler in `toolHandlers`
3. Test with Claude Code

## Architecture

```
Claude Code
    ↓
MCP Protocol (stdio)
    ↓
Devilbox MCP Server (Node.js)
    ↓
Docker CLI / docker-compose
    ↓
Devilbox Containers
```

## Security

- MCP server runs with your user permissions
- No network exposure (stdio only)
- Direct file system access (be careful with config changes)
- All Docker commands run as your user

## Roadmap

- [ ] Database backup/restore tools
- [ ] SSL certificate management
- [ ] Performance metrics (CPU, RAM, disk I/O)
- [ ] Log analysis with AI summaries
- [ ] Automated troubleshooting
- [ ] Project scaffolding (create new Laravel/WordPress projects)
- [ ] Import/export configurations

## Credits

- **Devilbox**: https://github.com/cytopia/devilbox
- **MCP Protocol**: https://modelcontextprotocol.io
- **Built for**: Claude Code

## License

MIT License - Same as Devilbox

---

**Version**: 1.0.0
**Author**: Johan Pretorius
**Date**: February 14, 2026
