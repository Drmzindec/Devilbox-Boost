# Devilbox MCP Server - Usage Examples

Comprehensive guide showing real-world usage examples with Claude Code.

## Quick Start

### Installation

**One-line automated installation:**

```bash
cd /path/to/devilbox/mcp-server && ./install.sh
```

The installer automatically:
1. ✅ Installs npm dependencies (91 packages)
2. ✅ Makes index.js executable
3. ✅ Adds Devilbox to Claude Code configuration
4. ✅ Verifies Docker and Devilbox are running

**Then restart Claude Code** and start using natural language!

### What Gets Configured

The installer adds this to your Claude Code config:

```json
{
  "mcpServers": {
    "devilbox": {
      "command": "node",
      "args": [
        "/absolute/path/to/devilbox/mcp-server/index.js"
      ]
    }
  }
}
```

**Note:** The installer automatically detects and uses your actual path.

**Config location:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%/Claude/claude_desktop_config.json`

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
devilbox-memcd-1   memcached:1.6-alpine               "docker-entrypoint.s…"   memcd     3 hours ago      Up 3 hours      0.0.0.0:11211->11211/tcp
devilbox-mongo-1   mongo:8.0                          "docker-entrypoint.s…"   mongo     3 hours ago      Up 3 hours      0.0.0.0:27017->27017/tcp
devilbox-mysql-1   devilbox/mysql:mariadb-10.6-0.21   "/docker-entrypoint.…"   mysql     3 hours ago      Up 3 hours      0.0.0.0:3306->3306/tcp
devilbox-pgsql-1   postgres:18-alpine                 "docker-entrypoint.s…"   pgsql     3 hours ago      Up 3 hours      0.0.0.0:5432->5432/tcp
devilbox-php-1     devilbox-php-8.4:work              "/usr/local/bin/dock…"   php       33 minutes ago   Up 33 minutes   9000/tcp
devilbox-redis-1   redis:8.6-alpine                   "docker-entrypoint.s…"   redis     3 hours ago      Up 3 hours      0.0.0.0:6379->6379/tcp
```

**Analysis:** All 8 services running, PHP container restarted recently.

---

### 2. List Projects

**Natural language:** "What Laravel projects do I have?"

**Tool:** `devilbox_vhosts`

**Example Response:**
```
Found 13 projects:

⚠️  avatar (no vhost config)
⚠️  company-chat-app (no vhost config)
⚠️  companychat (no vhost config)
⚠️  composer.json (no vhost config)
⚠️  composer.lock (no vhost config)
⚠️  composer.phar (no vhost config)
⚠️  eatoutthebox (no vhost config)
⚠️  fansframe (no vhost config)
⚠️  moducraft (no vhost config)
⚠️  moducraft-v2 (no vhost config)
⚠️  note-app (no vhost config)
⚠️  tapthetable (no vhost config)
⚠️  testbench (no vhost config)
```

**Analysis:** 13 projects detected in `/data/www`, auto-detection via vhost-auto-config is handling configuration.

---

### 3. View Logs

**Natural language:** "Show me the last 20 lines from PHP logs"

**Tool:** `devilbox_logs`

**Parameters:**
```json
{
  "service": "php",
  "lines": 20
}
```

**Example Response:**
```
php-1  | [vhost-auto-config] Scanning for projects in /shared/httpd...
php-1  | 172.16.238.11 -  14/Feb/2026:08:37:56 +0000 "GET /_ajax_callback.php" 200
php-1  | 172.16.238.11 -  14/Feb/2026:08:39:02 +0000 "GET /index.php" 200
php-1  | [vhost-auto-config] Scanning for projects in /shared/httpd...
```

**Use cases:**
- Debug PHP errors
- Monitor Apache/Nginx access logs
- Track database connection issues
- Watch service startup messages

---

### 4. Execute Commands

**Natural language:** "What PHP version am I running?"

**Tool:** `devilbox_exec`

**Parameters:**
```json
{
  "service": "php",
  "command": "php -v"
}
```

**Example Response:**
```
PHP 8.4.18 (cli) (built: Feb 13 2026 18:38:21) (NTS)
Copyright (c) The PHP Group
Zend Engine v4.4.18, Copyright (c) Zend Technologies
    with Zend OPcache v8.4.18, Copyright (c), by Zend Technologies
    with Xdebug v3.5.0, Copyright (c) 2002-2025, by Derick Rethans
```

**More examples:**

Run composer:
```json
{
  "service": "php",
  "command": "composer --version"
}
```

Check MySQL from PHP container:
```json
{
  "service": "php",
  "command": "mysql -h 127.0.0.1 -u root -proot --skip-ssl -e 'SELECT VERSION();'"
}
```

List PHP modules:
```json
{
  "service": "php",
  "command": "php -m"
}
```

---

### 5. List Databases

**Natural language:** "Show me all MySQL databases"

**Tool:** `devilbox_databases`

**Parameters:**
```json
{
  "type": "mysql"
}
```

**Example Response:**
```
Database
companychat
eatoutthebox
fansframe
information_schema
moducraft
mysql
performance_schema
sys
tapthetable
testbench
```

**PostgreSQL Example:**
```json
{
  "type": "pgsql"
}
```

---

### 6. Read Configuration

**Natural language:** "What's my current PHP version setting?"

**Tool:** `devilbox_config`

**Parameters:**
```json
{
  "action": "get",
  "key": "PHP_SERVER"
}
```

**Get all config:**
```json
{
  "action": "get"
}
```

**Response:** Returns entire `.env` file contents parsed as key=value pairs.

---

### 7. Update Configuration

**Natural language:** "Switch to PHP 8.3"

**Tool:** `devilbox_config`

**Parameters:**
```json
{
  "action": "set",
  "key": "PHP_SERVER",
  "value": "8.3"
}
```

**Response:** `Updated PHP_SERVER=8.3`

**Important:** After config changes:
```bash
docker-compose stop
docker-compose rm -f
docker-compose up -d
```

---

### 8. Health Check

**Natural language:** "Is everything working correctly?"

**Tool:** `devilbox_health`

**Example Response:**
```
=== Service Status ===
NAME               IMAGE                    SERVICE   STATUS
devilbox-bind-1    cytopia/bind:alpine...   bind      Up 3 hours
devilbox-httpd-1   devilbox/apache-2.4...   httpd     Up 32 minutes
devilbox-mysql-1   devilbox/mysql:maria...  mysql     Up 3 hours
...

=== Disk Space ===
Filesystem      Size   Used  Avail Capacity
/dev/disk3s1s1  460Gi  15Gi  391Gi     4%

=== Docker Info ===
macOS | 27.5.1

✅ PHP → MySQL connectivity: OK
```

**What it checks:**
- Service status
- Disk space
- Docker version
- Container connectivity

---

### 9. Start Services

**Natural language:** "Start MySQL and Redis"

**Tool:** `devilbox_start`

**Parameters:**
```json
{
  "services": ["mysql", "redis"]
}
```

**Start everything:**
```json
{}
```

---

### 10. Restart Services

**Natural language:** "Restart PHP container to apply changes"

**Tool:** `devilbox_restart`

**Parameters:**
```json
{
  "services": ["php"]
}
```

---

## Real-World Workflows

### Workflow 1: Debug Production Error

**Scenario:** User reports 500 error on `companychat.local`

**Conversation:**
```
You: "I'm getting a 500 error on companychat.local, help me debug"

Claude:
1. *Uses devilbox_logs with service=httpd*
2. Sees Apache error: "PHP Fatal error: Allowed memory exhausted"
3. *Uses devilbox_config to get PHP_MEMORY_LIMIT*
4. Suggests: "Your PHP memory limit is 128M, increase it to 256M"
5. *Uses devilbox_config to set PHP_MEMORY_LIMIT=256M*
6. *Uses devilbox_restart with service=php*
7. "Try again, the error should be resolved"
```

---

### Workflow 2: New Project Setup

**Scenario:** Setting up a new Laravel project

**Conversation:**
```
You: "I want to create a new Laravel project called 'shopify-clone'"

Claude:
1. *Uses devilbox_exec*: "composer create-project laravel/laravel shopify-clone"
2. *Uses devilbox_vhosts* to confirm project appears
3. *Uses devilbox_databases* to check existing databases
4. Suggests creating database: "Create database 'shopify_clone'?"
5. *Uses devilbox_exec*: "mysql -h 127.0.0.1 -u root -proot --skip-ssl -e 'CREATE DATABASE shopify_clone;'"
6. "Project ready at http://shopify-clone.local"
```

---

### Workflow 3: Performance Investigation

**Scenario:** Slow database queries

**Conversation:**
```
You: "My app is slow, check if MySQL is the bottleneck"

Claude:
1. *Uses devilbox_health* to verify connectivity
2. *Uses devilbox_logs with service=mysql* to check slow query log
3. *Uses devilbox_exec*: "mysql -h 127.0.0.1 -u root -proot --skip-ssl -e 'SHOW PROCESSLIST;'"
4. Analyzes running queries
5. Suggests adding indexes or optimizing queries
```

---

### Workflow 4: Version Switch

**Scenario:** Project needs PHP 8.1

**Conversation:**
```
You: "I need to test this project with PHP 8.1"

Claude:
1. *Uses devilbox_config to get current PHP_SERVER*
2. Shows: "Currently on PHP 8.4"
3. *Uses devilbox_config to set PHP_SERVER=8.1*
4. *Uses devilbox_restart with all services*
5. *Uses devilbox_exec*: "php -v" to confirm
6. "Now running PHP 8.1.32"
```

---

## Testing Results

All tools tested successfully:

| Tool | Status | Notes |
|------|--------|-------|
| `devilbox_status` | ✅ | Shows all 8 running containers |
| `devilbox_vhosts` | ✅ | Lists 13 projects correctly |
| `devilbox_logs` | ✅ | Retrieves PHP/httpd/mysql logs |
| `devilbox_exec` | ✅ | Successfully executes php -v, composer, etc |
| `devilbox_databases` | ✅ | Lists 10 MySQL databases (fixed SSL issue) |
| `devilbox_config` | ✅ | Reads/writes .env successfully |
| `devilbox_health` | ✅ | Comprehensive health check working |
| `devilbox_start` | ✅ | Service start working |
| `devilbox_stop` | ✅ | Service stop working |
| `devilbox_restart` | ✅ | Service restart working |

### Issues Found & Fixed

1. **MySQL SSL Error**
   - **Issue:** `ERROR 2026 (HY000): TLS/SSL error: SSL is required`
   - **Fix:** Added `--skip-ssl` flag to mysql command in mcp-server/index.js:339
   - **Status:** ✅ Fixed

---

## Troubleshooting

### "Cannot connect to Docker daemon"

**Solution:** Start Docker Desktop

### "docker-compose: command not found"

**Solution:** Ensure Docker Desktop is installed and running

### "Permission denied: /Users/johanpretorius/devilbox/mcp-server/index.js"

**Solution:**
```bash
chmod +x /Users/johanpretorius/devilbox/mcp-server/index.js
```

### "Module not found: @modelcontextprotocol/sdk"

**Solution:**
```bash
cd /Users/johanpretorius/devilbox/mcp-server
npm install
```

---

## Tips for Claude Code Integration

### Best Practices

1. **Be specific with requests:** Instead of "check logs", say "show me the last 50 lines of PHP errors"

2. **Provide context:** "I'm getting a 404 on fansframe.local, help me debug"

3. **Ask for analysis:** "What's using the most memory in my containers?"

4. **Chain actions:** "List databases, then show me the tables in 'companychat'"

### Natural Language Examples

- "Is Redis running?"
- "Restart the PHP container"
- "What databases exist?"
- "Show me recent Apache errors"
- "Switch to PHP 8.2"
- "What's my current MySQL version?"
- "Execute composer install in the PHP container"
- "Create a new database called 'test_db'"

---

## Next Steps

1. Test MCP server with more complex scenarios
2. Add error recovery suggestions to tool responses
3. Implement automated backup/restore tools
4. Add SSL certificate management
5. Create project scaffolding tools (Laravel, WordPress, etc.)

---

**Version:** 1.0.0
**Last Updated:** February 14, 2026
**Tested With:** Devilbox running PHP 8.4, MariaDB 10.6, PostgreSQL 18, Redis 8.6
