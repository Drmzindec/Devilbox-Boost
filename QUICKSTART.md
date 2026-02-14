# Devilbox Modern - Quick Start Guide

Get up and running with Devilbox in 10 minutes.

## Prerequisites

- macOS, Linux, or Windows
- Docker Desktop installed and running
- 8GB RAM minimum (16GB recommended)
- 20GB free disk space

---

## Installation

### Option 1: Interactive Setup Wizard (Recommended)

```bash
cd /path/to/devilbox
./setup-devilbox.sh
```

Follow the prompts - takes 10-20 minutes total.

See [SETUP-WIZARD.md](SETUP-WIZARD.md) for detailed walkthrough.

### Option 2: Manual Setup

```bash
# 1. Copy environment file
cp env-example .env

# 2. Edit configuration (optional)
nano .env

# 3. Build custom PHP images
./docker-images/build-php.sh 8.4

# 4. Start Devilbox
docker compose up -d

# 5. Check status
docker compose ps
```

---

## First Project

### Laravel

```bash
# Create project
docker compose exec php laravel new my-blog

# Wait 30 seconds for vhost auto-detection

# Visit
open http://my-blog.local
```

### WordPress

```bash
# Download WordPress
docker compose exec php wp core download --path=my-site

# Configure database
docker compose exec php wp config create \
    --path=my-site \
    --dbname=my_site \
    --dbuser=root \
    --dbpass=root \
    --dbhost=127.0.0.1

# Create database
docker compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl \
    -e "CREATE DATABASE my_site;"

# Install WordPress
docker compose exec php wp core install \
    --path=my-site \
    --url=http://my-site.local \
    --title="My Site" \
    --admin_user=admin \
    --admin_password=admin \
    --admin_email=admin@example.com

# Visit
open http://my-site.local
```

### Custom PHP

```bash
# Create directory
mkdir data/www/my-project
cd data/www/my-project

# Create index.php
echo '<?php phpinfo();' > index.php

# Visit
open http://my-project.local
```

---

## Access Points

| Service | URL |
|---------|-----|
| **Dashboard** | http://localhost |
| **phpMyAdmin** | http://localhost/vendor/phpmyadmin-5.2.3/ |
| **Adminer** | http://localhost/vendor/adminer-5.4.2-devilbox.php |
| **phpCacheAdmin** | http://localhost/vendor/phpcacheadmin-2.4.1/ |
| **phpPgAdmin** | http://localhost/vendor/phppgadmin-7.13.0/ |

### Database Credentials

```
Host: 127.0.0.1
Username: root
Password: root
```

**Note**: Use `127.0.0.1` not `localhost` (port forwarding)

---

## Common Commands

### Container Management

```bash
# Start Devilbox
docker compose up -d

# Stop Devilbox
docker compose stop

# Restart services
docker compose restart

# View running containers
docker compose ps

# View logs
docker compose logs -f php
docker compose logs -f httpd
docker compose logs -f mysql
```

### PHP Commands

If you added `bin/` to PATH during setup:

```bash
# Run directly on host
composer install
npm install
artisan migrate
wp plugin list
```

Without PATH setup:

```bash
# Run via docker compose exec
docker compose exec php composer install
docker compose exec php npm install
docker compose exec php php artisan migrate
docker compose exec php wp plugin list
```

### Database Commands

```bash
# Connect to MySQL
docker compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl

# Create database
docker compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl \
    -e "CREATE DATABASE my_database;"

# Import SQL file
docker compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl \
    my_database < backup.sql

# Export database
docker compose exec php mysqldump -h 127.0.0.1 -u root -proot --skip-ssl \
    my_database > backup.sql
```

### Cache Commands

```bash
# Redis CLI
docker compose exec php redis-cli -h 127.0.0.1 ping

# Memcached stats
docker compose exec php telnet 127.0.0.1 11211
# Then type: stats

# Flush Redis
docker compose exec php redis-cli -h 127.0.0.1 FLUSHALL

# Flush Memcached
docker compose exec php telnet 127.0.0.1 11211
# Then type: flush_all
```

---

## Switching PHP Versions

Edit `.env`:

```bash
# Change this line
PHP_SERVER=8.4  # or 8.3
```

Restart:

```bash
docker compose restart
```

Check version:

```bash
docker compose exec php php -v
```

---

## Development Workflow

### 1. Create New Laravel Project

```bash
docker compose exec php laravel new my-app
```

### 2. Wait for Vhost Detection (30 seconds)

The vhost auto-detection service runs every 30 seconds and creates:
- `.devilbox/apache24.yml`
- `.devilbox/nginx.yml`

### 3. Configure Database

```bash
cd data/www/my-app

# Update .env
nano .env
```

Set:
```env
DB_HOST=127.0.0.1
DB_DATABASE=my_app
DB_USERNAME=root
DB_PASSWORD=root
```

### 4. Create Database

```bash
docker compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl \
    -e "CREATE DATABASE my_app;"
```

### 5. Run Migrations

```bash
docker compose exec php bash -c "cd my-app && php artisan migrate"
```

### 6. Visit Project

```
http://my-app.local
```

---

## Troubleshooting

### Docker Not Running

```bash
# Check Docker
docker info

# If error, start Docker Desktop
```

### Port 80 Already in Use

```bash
# Find process
sudo lsof -i :80

# Change port in .env
HOST_PORT_HTTPD=8000

# Restart
docker compose restart

# Access via
http://localhost:8000
```

### Project Shows 404

**Wait 30 seconds** for vhost auto-detection, then:

```bash
# Check vhost config exists
ls -la data/www/my-project/.devilbox/

# Force regeneration
docker compose restart httpd

# Check logs
docker compose logs httpd
```

### Database Connection Failed

**Use `127.0.0.1` not `localhost`**:

```php
// ❌ Wrong
DB_HOST=localhost

// ✅ Correct
DB_HOST=127.0.0.1
```

### .local Domains Don't Work

Add to `/etc/hosts`:

```bash
sudo nano /etc/hosts
```

Add line:
```
127.0.0.1 my-project.local
```

Or use Devilbox DNS (port 1053).

---

## Next Steps

### Documentation

- **Setup Wizard**: [SETUP-WIZARD.md](SETUP-WIZARD.md)
- **Development Guidelines**: [.claude/README.md](.claude/README.md)
- **Service Guides**: [.claude/skills/](.claude/skills/)
- **Roadmap**: [ROADMAP-MODERNIZATION.md](ROADMAP-MODERNIZATION.md)

### Skills & Guides

Located in `.claude/skills/`:

- `build-php-image.md` - Building custom PHP images
- `create-new-project.md` - Project creation guide
- `debug-container-issues.md` - Troubleshooting
- `test-mcp-server.md` - MCP testing
- `using-redis.md` - Redis configuration and usage
- `using-memcached.md` - Memcached guide
- `using-mysql.md` - MySQL/MariaDB guide
- `using-postgresql.md` - PostgreSQL guide
- `using-mongodb.md` - MongoDB guide

### Advanced Features

- **MCP Server**: AI-powered Devilbox control with Claude Code
- **Vhost Auto-Detection**: Automatic Apache/Nginx configuration
- **Port Forwarding**: Direct `127.0.0.1` database connections
- **Modern Tools**: Bun, Vite, Pest, React, Vue, Angular
- **Multiple PHP Versions**: Switch between 8.3 and 8.4

---

## Support

- **Issues**: Check container logs first
  ```bash
  docker compose logs <service>
  ```

- **Health Check**: Use dashboard at http://localhost

- **Reset**: Complete rebuild
  ```bash
  docker compose down
  docker compose up -d --build
  ```

---

## What's Included

### Modern Tools

- **PHP 8.3 & 8.4** with latest features
- **Composer 2.9.5** - Dependency management
- **Laravel Installer 5.24.5** - Quick project creation
- **WP-CLI 2.12.0** - WordPress management
- **Pest 4.3.2** - Modern testing framework
- **Node.js 24.13.1** - JavaScript runtime
- **Bun 1.3.9** - Fast all-in-one toolkit
- **Vite 7.3.1** - Lightning-fast build tool
- **Vue CLI** - Vue.js framework
- **React CLI** - React framework
- **Angular CLI** - Angular framework

### Database Services

- **MySQL 8.0** / MariaDB
- **PostgreSQL 16**
- **MongoDB 7**
- **Redis 7**
- **Memcached**

### Admin Tools

- **phpMyAdmin 5.2.3** - MySQL administration
- **Adminer 5.4.2** - Lightweight database manager
- **phpPgAdmin 7.13.0** - PostgreSQL admin
- **phpCacheAdmin 2.4.1** - Redis/Memcached manager

---

Happy coding! 🚀
