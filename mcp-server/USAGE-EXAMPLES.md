# Devilbox MCP Server - Usage Examples

Comprehensive guide showing real-world usage examples with Claude Code.

## Quick Start

### Installation

**One-line automated installation:**

```bash
cd /path/to/Devilbox-Boost/mcp-server && ./install.sh
```

The installer automatically:
1. ✅ Installs npm dependencies
2. ✅ Makes index.js executable
3. ✅ Creates `.mcp.json` in project root (Claude Code CLI)
4. ✅ Updates Claude Desktop config (if applicable)
5. ✅ Verifies Docker and Devilbox are running

**Then restart Claude Code** and start using natural language!

### What Gets Configured

The installer creates a `.mcp.json` in the Devilbox-Boost project root (for Claude Code CLI) and updates the Claude Desktop config:

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

**Note:** The installer automatically detects and uses your actual path.

**Config locations:**
- **Claude Code CLI**: `.mcp.json` in the Devilbox-Boost project root
- **Claude Desktop (macOS)**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Claude Desktop (Linux)**: `~/.config/Claude/claude_desktop_config.json`
- **Claude Desktop (Windows)**: `%APPDATA%/Claude/claude_desktop_config.json`

---

## Tool Examples

### 1. Check Service Status

**Natural language:** "What's the status of my Devilbox environment?"

**Tool:** `devilbox_status`

**Example Response:**
```
NAME               IMAGE                              COMMAND                  SERVICE   CREATED          STATUS          PORTS
devilbox-bind-1    cytopia/bind:alpine-0.35           "/docker-entrypoint.…"   bind      3 hours ago      Up 3 hours      0.0.0.0:1053->53/tcp
devilbox-httpd-1   devilbox/apache-2.4:alpine-1.2     "/docker-entrypoint.…"   httpd     3 hours ago      Up 32 minutes   0.0.0.0:80->80/tcp
devilbox-mysql-1   devilbox/mysql:mariadb-10.6-0.21   "/docker-entrypoint.…"   mysql     3 hours ago      Up 3 hours      0.0.0.0:3306->3306/tcp
devilbox-php-1     devilbox-php-8.4:work              "/usr/local/bin/dock…"   php       33 minutes ago   Up 33 minutes   9000/tcp
...
```

---

### 2. List Projects

**Natural language:** "What projects do I have and what are their URLs?"

**Tool:** `devilbox_projects`

**Example Response:**
```
Found 11 projects in data/www (TLD: .local):

- tapthetable
    URL:       http://tapthetable.local
    Framework: Laravel
    Vhost:     auto-detected
- fansframe
    URL:       http://fansframe.local
    Framework: Laravel
    Vhost:     custom template (.devilbox/)
...
```

Framework detection recognizes Laravel (`artisan`), WordPress (`wp-config.php` / `wp-content`), Symfony (`bin/console`), generic composer projects, plain PHP and static sites.

---

### 3. Run Artisan Commands

**Natural language:** "Run the pending migrations for tapthetable"

**Tool:** `devilbox_artisan` with `{ "project": "tapthetable", "args": "migrate" }`

Runs `php artisan migrate` as user `devilbox` inside `/shared/httpd/tapthetable` — no permission-mangled files.

**More examples:**

| You say | Tool call |
|---------|-----------|
| "Show me all routes in tapthetable" | `{ "project": "tapthetable", "args": "route:list" }` |
| "Clear the caches" | `{ "project": "tapthetable", "args": "optimize:clear" }` |
| "Make a model with migration" | `{ "project": "tapthetable", "args": "make:model Order -m" }` |
| "Run one queued job" | `{ "project": "tapthetable", "args": "queue:work --once" }` |
| "Roll back the last migration" | `{ "project": "tapthetable", "args": "migrate:rollback --step=1" }` |

If the project isn't a Laravel app, the tool tells you so and points at `devilbox_projects`.

---

### 4. Run Composer

**Natural language:** "Install the composer dependencies for fansframe"

**Tool:** `devilbox_composer` with `{ "project": "fansframe", "args": "install" }`

**More examples:**
- `{ "project": "fansframe", "args": "require laravel/pint --dev" }`
- `{ "project": "fansframe", "args": "update --with-all-dependencies" }`
- `{ "project": "fansframe", "args": "dump-autoload -o" }`

Composer runs as user `devilbox` so `vendor/` stays owned by you.

---

### 5. Run SQL Queries

**Natural language:** "How many users signed up this week in the tapthetable database?"

**Tool:** `devilbox_sql` with:
```json
{
  "engine": "mysql",
  "database": "tapthetable",
  "query": "SELECT COUNT(*) FROM users WHERE created_at >= NOW() - INTERVAL 7 DAY;"
}
```

**Example Response:**
```
+----------+
| COUNT(*) |
+----------+
|       42 |
+----------+
```

**PostgreSQL:** `{ "engine": "pgsql", "query": "SELECT version();" }`

Notes:
- The query is piped to the client binary **inside the database container** via stdin — nothing touches your host shell.
- Credentials come from `.env` (`MYSQL_ROOT_PASSWORD`, `PGSQL_ROOT_USER`, `PGSQL_ROOT_PASSWORD`) and are passed as environment variables.

---

### 6. List Databases

**Natural language:** "List all my databases"

**Tool:** `devilbox_databases` (default `type: "all"` covers both engines)

**Example Response:**
```
=== MySQL/MariaDB ===
+--------------------+
| Database           |
+--------------------+
| tapthetable        |
| fansframe          |
| information_schema |
...

=== PostgreSQL ===
List of databases ...
```

---

### 7. Tail PHP Errors

**Natural language:** "Something 500s — show me the last PHP errors"

**Tool:** `devilbox_php_errors` with `{ "lines": 50 }`

**Example Response:**
```
Last 50 lines of log/php-fpm-8.4/php-errors.log:

[13-Aug-2026 22:14:03 UTC] PHP Fatal error:  Uncaught Error: Class "App\Services\Missing" not found in ...
```

Reads the log file directly from the host — works even when containers are down. Pass `php_version` (e.g. `"8.2"`) to inspect a different version's log.

---

### 8. Toggle Xdebug

**Natural language:** "Enable Xdebug so I can step through this request"

**Tool:** `devilbox_xdebug` with `{ "action": "on" }`

**Example Response:**
```
Xdebug enabled (mode=debug,develop, port=9003, start_with_request=yes).
Wrote cfg/php-ini-8.4/zzz-mcp-xdebug.ini.
PHP-FPM reloaded (kill -USR2 1). If the change does not apply, restart the php container: devilbox_restart ["php"].
```

**Options:**
- `{ "action": "on", "trigger": true }` — only debug requests carrying `XDEBUG_TRIGGER` (faster for everything else)
- `{ "action": "on", "port": 9000 }` — for IDEs listening on the legacy port
- `{ "action": "off" }` — remove the drop-in and reload
- `{ "action": "status" }` — show drop-in state and what PHP reports

The drop-in file sorts after `custom.ini`, so it cleanly overrides the performance-first `xdebug.mode = off` default without editing your own config.

---

### 9. View Container Logs

**Natural language:** "Show me the last 200 lines of MySQL logs"

**Tool:** `devilbox_logs` with `{ "service": "mysql", "lines": 200 }`

---

### 10. Execute Commands in Containers

**Natural language:** "What PHP extensions are loaded?"

**Tool:** `devilbox_exec` with `{ "service": "php", "command": "php -m" }`

**More examples:**
- `{ "service": "php", "command": "node -v && npm -v" }`
- `{ "service": "php", "command": "npm run build", "user": "devilbox", "workdir": "/shared/httpd/tapthetable" }`
- `{ "service": "redis", "command": "redis-cli INFO memory" }`
- `{ "service": "mysql", "command": "mysqladmin status -uroot -p\"$MYSQL_ROOT_PASSWORD\"" }`

The command runs via `sh -c` **inside** the chosen container. The service name is allowlisted; an invalid name returns:

```
Error: Service 'foo' not known. Valid services: bind, httpd, mailpit, meilisearch, memcd, mongo, mysql, pgsql, php, rabbit, redis, rustfs
```

---

### 11. Manage Services

**Natural language:** "Restart PHP and the webserver"

**Tool:** `devilbox_restart` with `{ "services": ["php", "httpd"] }`

Also: `devilbox_start` (uses `docker compose up -d`, so override merging and dependency ordering work) and `devilbox_stop`.

---

### 12. Configuration

**Natural language:** "What PHP version am I running?"

**Tool:** `devilbox_config` with `{ "action": "get", "key": "PHP_SERVER" }`

**Natural language:** "Switch me to PHP 8.2"

**Tool:** `devilbox_config` with `{ "action": "set", "key": "PHP_SERVER", "value": "8.2" }`

Then: "Restart everything" → `devilbox_restart`.

**Note:** `{ "action": "get" }` without a key lists the whole `.env`, with `*PASSWORD*`/`*SECRET*`/`*TOKEN*` values redacted. Request a specific key to see its real value.

---

### 13. Health Check

**Natural language:** "Is everything working correctly?"

**Tool:** `devilbox_health`

**Example Response:**
```
=== Service Status ===
(docker compose ps output)

=== Docker Info ===
Docker Desktop | Docker 27.4.0

=== Disk Space ===
Filesystem     Size   Used  Avail Capacity  Mounted on
...

=== Connectivity (from php container) ===
[OK]   php -> mysql:3306
[OK]   php -> pgsql:5432
[OK]   php -> redis:6379
[OK]   php -> memcd:11211
[OK]   php -> mongo:27017
```

---

## Real-World Workflows

### Morning startup

```
You: "Start devilbox and check everything is healthy"
Claude: devilbox_start → devilbox_health → "All services up, connectivity OK"
```

### Debugging a broken page

```
You: "tapthetable.local is throwing a 500, figure out why"
Claude: devilbox_php_errors → finds the stack trace
        devilbox_artisan {"project":"tapthetable","args":"optimize:clear"} → cache cleared
        "The error was a stale config cache referencing a removed provider. Fixed."
```

### Setting up a freshly cloned project

```
You: "I just cloned fansframe into data/www — get it running"
Claude: devilbox_composer {"project":"fansframe","args":"install"}
        devilbox_sql {"query":"CREATE DATABASE IF NOT EXISTS fansframe;"}
        devilbox_artisan {"project":"fansframe","args":"migrate --seed"}
        devilbox_projects → "Ready at http://fansframe.local"
```

### Step-debugging session

```
You: "I need to step through the checkout flow"
Claude: devilbox_xdebug {"action":"on","trigger":true}
        "Xdebug is on (trigger mode). Add XDEBUG_TRIGGER to your request or use the browser extension."
...later...
You: "Done debugging"
Claude: devilbox_xdebug {"action":"off"} → "Xdebug off, PHP-FPM reloaded"
```

### Database inspection

```
You: "Which tables in tapthetable are biggest?"
Claude: devilbox_sql {"database":"information_schema","query":"SELECT table_name, ROUND((data_length+index_length)/1024/1024,1) AS mb FROM tables WHERE table_schema='tapthetable' ORDER BY mb DESC LIMIT 10;"}
```

---

## Error Messages You Might See

| Message | Meaning / Fix |
|---------|---------------|
| `Service 'x' not known. Valid services: ...` | Typo or non-existent service; pick from the list |
| `Project 'x' not found in data/www. Available projects: ...` | Project directory doesn't exist |
| `Project 'x' has no artisan file` | Not a Laravel project — use `devilbox_exec` or `devilbox_composer` |
| `PHP error log not found at log/php-fpm-X.Y/...` | Wrong version; check `PHP_SERVER` in `.env` or pass `php_version` |
| `Command timed out after 120s` | Long-running command; run it in a terminal instead |
| `Cannot connect to the Docker daemon` | Start Docker Desktop |

---

## Testing the Server

```bash
cd /path/to/Devilbox-Boost/mcp-server
npm test
```

The test suite initializes the server over stdio, verifies all 16 tools are registered, exercises the filesystem-backed tools (`devilbox_vhosts`, `devilbox_projects`, `devilbox_config`) and confirms the service allowlist rejects hostile input like `php; rm -rf /`. It does not require Docker to be running.
