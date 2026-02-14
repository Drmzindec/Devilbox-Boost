# Devilbox MCP Server - Quick Start

## Installation (30 seconds)

```bash
cd /path/to/devilbox/mcp-server && ./install.sh
```

Then **restart Claude Code**.

That's it! 🎉

---

## Usage Examples

Just talk to Claude naturally:

### Service Management
```
"What's the status of my Devilbox?"
"Start MySQL and Redis"
"Restart the PHP container"
```

### Viewing Logs
```
"Show me the last 50 PHP logs"
"What errors are in the httpd logs?"
"View MySQL logs"
```

### Project Management
```
"What projects do I have?"
"List all Laravel projects"
"Show me all virtual hosts"
```

### Database Operations
```
"List all databases"
"Show me all MySQL databases"
"List PostgreSQL databases"
```

### Configuration
```
"What PHP version am I using?"
"Switch to PHP 8.3"
"Show me the current configuration"
```

### Command Execution
```
"Run php -v in the PHP container"
"Execute composer --version"
"Run artisan migrate in companychat"
```

### Health Checks
```
"Is everything working correctly?"
"Check Devilbox health"
"Are all services running?"
```

---

## What You Get

- ✅ **10 AI-powered tools** for Devilbox management
- ✅ **Natural language interface** - no commands to memorize
- ✅ **Automatic Claude Code integration** - zero manual config
- ✅ **Real-time service monitoring** - status, logs, health
- ✅ **Smart project detection** - lists all your vhosts
- ✅ **Database operations** - list databases, run queries
- ✅ **Configuration management** - read/write .env settings

---

## Troubleshooting

### "MCP server not showing up in Claude Code"

1. Verify installation:
```bash
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

2. Look for the `devilbox` entry under `mcpServers`

3. Completely restart Claude Code (quit and reopen)

### "Docker not running"

```bash
# Start Docker Desktop, then verify:
docker info
```

### "Devilbox not running"

```bash
cd /path/to/devilbox
docker-compose up -d
```

---

## Need More Help?

- **Full examples**: See [USAGE-EXAMPLES.md](USAGE-EXAMPLES.md)
- **Tool reference**: See [README.md](README.md)
- **Configuration**: See installer output for your config path
- **Server path**: Automatically detected by installer

---

**Version**: 1.0.0
**Last Updated**: February 14, 2026
