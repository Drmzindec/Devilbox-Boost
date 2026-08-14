# Devilbox MCP Server

AI-powered assistant for Devilbox development environment management.

> **🚀 Quick Start:** See [QUICKSTART.md](QUICKSTART.md) for 30-second installation!

## What is this?

The Devilbox MCP (Model Context Protocol) server provides AI assistants like Claude with direct control over your Devilbox environment. You can use natural language to manage services, run artisan and composer, query databases, toggle Xdebug, tail PHP error logs, and more!

## Features

### 🚀 Service Management
- **Start/Stop/Restart** services with natural language
- **Check status** of all running containers
- **View logs** from any service

### 🐘 PHP / Laravel Workflow
- **Run artisan** commands in any project (`devilbox_artisan`)
- **Run composer** in any project (`devilbox_composer`)
- **Tail PHP-FPM error logs** straight from the host (`devilbox_php_errors`)
- **Toggle Xdebug** on/off with a config drop-in + PHP-FPM reload (`devilbox_xdebug`)

### 💾 Database Operations
- **Run SQL queries** against MySQL/MariaDB or PostgreSQL (`devilbox_sql`)
- **List databases** across both engines (`devilbox_databases`)
- Credentials are read from `.env` — nothing is hardcoded

### 🌐 Project & Vhost Management
- **List all projects** with framework detection and vhost URLs (`devilbox_projects`)
- **Vhost configuration status** per project (`devilbox_vhosts`)

### 📊 Monitoring & Health
- **Health checks** for all services
- **Connectivity tests** from the PHP container to MySQL, PostgreSQL, Redis, Memcached and MongoDB
- **Disk space** and Docker info

### ⚙️ Configuration
- **Read and update** `.env` settings (passwords are redacted in full listings)
- **Execute commands** inside containers (`devilbox_exec`)

## Security Model

This server was rewritten to be injection-safe:

- **No host shell, ever.** Every process is spawned with `child_process.spawn` and argument arrays — tool input can never inject into your host shell.
- **Container commands are explicit.** `devilbox_exec`, `devilbox_artisan` and `devilbox_composer` run their command string via `docker compose exec -T <service> sh -c <command>`, where the command is a single argument. The shell that interprets it lives *inside* the container — that is the tool's purpose.
- **Service names are allowlisted** against the services actually defined in `docker-compose.yml` and `docker-compose.override.yml`. Unknown names are rejected with the list of valid ones.
- **Project names are validated** (`[A-Za-z0-9._-]`, must exist in `data/www`) — no path traversal.
- **Credentials come from `.env`** (`MYSQL_ROOT_PASSWORD`, `PGSQL_ROOT_USER`, `PGSQL_ROOT_PASSWORD`) and are passed via environment variables, never on a shell command line.
- **Compose commands run with `cwd` set to the Devilbox root**, so `docker-compose.override.yml` merging always works.

## Installation

### Automated Installation (Recommended)

Run the installer to automatically configure both Claude Code CLI and Claude Desktop:

```bash
cd /path/to/Devilbox-Boost/mcp-server
./install.sh
```

The installer will:
1. ✅ Install npm dependencies
2. ✅ Make index.js executable
3. ✅ Create `.mcp.json` in the project root (for Claude Code CLI)
4. ✅ Update `claude_desktop_config.json` (for Claude Desktop app)
5. ✅ Verify Docker and Devilbox are running

**Then restart Claude Code** and start using natural language!

### Manual Installation

If you prefer manual setup:

1. **Install Dependencies:**
```bash
cd /path/to/Devilbox-Boost/mcp-server
npm install
```

2. **Make executable:**
```bash
chmod +x index.js
```

3. **Configure your Claude client:**

#### Claude Code CLI (recommended)

Create a `.mcp.json` file in the Devilbox-Boost project root:

```json
{
  "mcpServers": {
    "devilbox": {
      "command": "node",
      "args": [
        "/path/to/Devilbox-Boost/mcp-server/index.js"
      ]
    }
  }
}
```

The MCP server activates automatically when Claude Code is opened in the Devilbox-Boost directory.

#### Claude Desktop App

Edit your Claude Desktop configuration file:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

Add the same `mcpServers` block shown above.

4. **Restart Claude Code**

## Available Tools

| Tool | Description | Example Usage |
|------|-------------|---------------|
| `devilbox_status` | Get status of all services | "Show me Devilbox status" |
| `devilbox_start` | Start services | "Start MySQL and Redis" |
| `devilbox_stop` | Stop services | "Stop all services" |
| `devilbox_restart` | Restart services | "Restart PHP container" |
| `devilbox_logs` | View container logs | "Show me the last 50 lines from PHP logs" |
| `devilbox_exec` | Execute command in container | "Run `php -v` in the PHP container" |
| `devilbox_artisan` | Run artisan in a project | "Run migrations for tapthetable" |
| `devilbox_composer` | Run composer in a project | "Run composer install in fansframe" |
| `devilbox_sql` | Run a SQL query | "How many users are in the tapthetable database?" |
| `devilbox_databases` | List databases | "Show me all databases" |
| `devilbox_php_errors` | Tail PHP-FPM error log | "Show me recent PHP errors" |
| `devilbox_xdebug` | Toggle/inspect Xdebug | "Turn on Xdebug" |
| `devilbox_projects` | List projects with URLs & frameworks | "What projects do I have and what are their URLs?" |
| `devilbox_vhosts` | List vhost config status | "Which projects have custom vhost templates?" |
| `devilbox_config` | Get/set `.env` configuration | "What PHP version am I using?" |
| `devilbox_health` | Health check | "Is Devilbox healthy?" |

## Usage Examples

For comprehensive usage examples and real-world workflows, see [USAGE-EXAMPLES.md](USAGE-EXAMPLES.md).

### Quick Examples

**Starting Services:**
```
You: "Start my Devilbox environment"
Claude: *uses devilbox_start* → All services started!
```

**Laravel Development:**
```
You: "Run the migrations for tapthetable and seed the database"
Claude: *uses devilbox_artisan (migrate --seed)* → Migrations complete!
```

**Database Queries:**
```
You: "How many rows are in the users table of the fansframe database?"
Claude: *uses devilbox_sql* → 1,284 users
```

**Debugging:**
```
You: "Something is broken, show me the last PHP errors"
Claude: *uses devilbox_php_errors* → Here's the stack trace from php-errors.log...

You: "Enable Xdebug so I can step through it"
Claude: *uses devilbox_xdebug (on)* → Xdebug enabled on port 9003, PHP-FPM reloaded
```

**Managing Projects:**
```
You: "What Laravel projects do I have?"
Claude: *uses devilbox_projects* → Found 11 projects, 6 are Laravel...
```

**Configuration:**
```
You: "Switch to PHP 8.3"
Claude: *uses devilbox_config* → Updated PHP_SERVER=8.3, restart to apply
```

## Tool Parameters

### devilbox_start / devilbox_stop / devilbox_restart

```typescript
{
  services?: string[]  // Optional: ['php', 'mysql', 'redis']
                       // If omitted, affects all services
}
```

Service names are validated against the compose files. Valid services:
`bind, httpd, mailpit, meilisearch, memcd, mongo, mysql, pgsql, php, rabbit, redis, rustfs`

### devilbox_logs

```typescript
{
  service: string,     // Required: 'php', 'httpd', 'mysql', etc.
  lines?: number       // Optional: number of lines (default: 100, max: 10000)
}
```

### devilbox_exec

```typescript
{
  service: string,     // Required: container to execute in (allowlisted)
  command: string,     // Required: command to run (via sh -c INSIDE the container)
  user?: string,       // Optional: e.g. 'devilbox' for write-safe file operations
  workdir?: string     // Optional: absolute path inside the container
}
```

**Examples:**
- `{ "service": "php", "command": "php -v" }`
- `{ "service": "php", "command": "npm run build", "user": "devilbox", "workdir": "/shared/httpd/myapp" }`

### devilbox_artisan

```typescript
{
  project: string,     // Required: directory name under data/www
  args?: string        // Optional: artisan arguments (default: "list")
}
```

Runs `php artisan <args>` as user `devilbox` in `/shared/httpd/<project>`.

**Examples:**
- `{ "project": "tapthetable", "args": "migrate --seed" }`
- `{ "project": "tapthetable", "args": "route:list" }`
- `{ "project": "tapthetable", "args": "queue:work --once" }`

### devilbox_composer

```typescript
{
  project: string,     // Required: directory name under data/www
  args: string         // Required: composer arguments
}
```

**Examples:**
- `{ "project": "fansframe", "args": "install" }`
- `{ "project": "fansframe", "args": "require laravel/pint --dev" }`
- `{ "project": "fansframe", "args": "dump-autoload" }`

### devilbox_sql

```typescript
{
  engine?: 'mysql' | 'pgsql',  // Default: 'mysql'
  query: string,               // Required: the SQL to run
  database?: string            // Optional: database to run it against
}
```

The query is piped to the client binary inside the database container via stdin. Credentials come from `.env` (`MYSQL_ROOT_PASSWORD`, `PGSQL_ROOT_USER`, `PGSQL_ROOT_PASSWORD`).

**Examples:**
- `{ "query": "SHOW TABLES;", "database": "tapthetable" }`
- `{ "engine": "pgsql", "query": "SELECT version();" }`

### devilbox_databases

```typescript
{
  type?: 'mysql' | 'pgsql' | 'all'  // Default: 'all'
}
```

### devilbox_php_errors

```typescript
{
  lines?: number,       // Default: 50
  php_version?: string  // Default: PHP_SERVER from .env (e.g. "8.4")
}
```

Reads `log/php-fpm-<version>/php-errors.log` directly from the host filesystem — no containers involved.

### devilbox_xdebug

```typescript
{
  action: 'status' | 'on' | 'off',  // Required
  mode?: string,                    // Default: "debug,develop"
  port?: number,                    // Default: 9003
  trigger?: boolean                 // Default: false (debug every request)
}
```

`on` writes `cfg/php-ini-<version>/zzz-mcp-xdebug.ini` (sorted last, so it overrides `custom.ini`) and reloads PHP-FPM with `kill -USR2 1`. `off` removes the drop-in and reloads. If a reload doesn't apply the change, restart the php container.

### devilbox_config

```typescript
{
  action: 'get' | 'set',  // Required
  key?: string,           // Optional for get, required for set
  value?: string          // Required for set
}
```

**Examples:**
- Get all (passwords redacted): `{ "action": "get" }`
- Get one: `{ "action": "get", "key": "PHP_SERVER" }`
- Set value: `{ "action": "set", "key": "PHP_SERVER", "value": "8.4" }`

## Testing

```bash
# Offline-friendly test suite (protocol, tools list, filesystem tools, allowlist)
npm test

# Start the server manually
node index.js

# Explore interactively with the MCP inspector
npx @modelcontextprotocol/inspector node index.js
```

## Troubleshooting

### "Service 'foo' not known"

Tool arguments for service names are allowlisted. The error message lists all valid services — pick one of those.

### "Command not found: docker compose"

Make sure Docker Desktop is running and `docker compose` is available (Docker Desktop includes it by default).

### "Project 'foo' not found in data/www"

Project names must match a directory in `data/www`. The error lists available projects.

### "Module not found: @modelcontextprotocol/sdk"

Run: `npm install` in the mcp-server directory

For more troubleshooting tips, see [USAGE-EXAMPLES.md](USAGE-EXAMPLES.md).

## Development

### Adding New Tools

1. Add tool definition to `tools` array
2. Implement handler in `toolHandlers`
3. Use `run()` / `dockerCompose()` / `composeExec()` / `execInContainer()` — never build shell strings
4. Validate service names with `validateService()` and projects with `validateProject()`
5. Add a test in `test-mcp.js` and run `npm test`

## Architecture

```
Claude Code
    ↓
MCP Protocol (stdio)
    ↓
Devilbox MCP Server (Node.js, spawn + argv arrays — no host shell)
    ↓
docker compose (cwd = Devilbox root, override merging intact)
    ↓
Devilbox Containers
```

## Security

- MCP server runs with your user permissions
- No network exposure (stdio only)
- No host shell — all processes spawned with argument arrays
- Service and project names validated against real compose services / directories
- Credentials read from `.env`, passed via environment variables, redacted in config listings

## Credits

- **Devilbox**: https://github.com/cytopia/devilbox
- **MCP Protocol**: https://modelcontextprotocol.io
- **Built for**: Claude Code

## License

MIT License - Same as Devilbox

---

**Version**: 2.0.0
**Author**: Devilbox Boost Contributors
