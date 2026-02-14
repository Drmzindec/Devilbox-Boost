# Devilbox Boost

**Modern tools and workflow improvements for Devilbox.**

Devilbox Boost is an enhancement layer that modernizes the official [Devilbox](https://github.com/cytopia/devilbox) with PHP 8.3/8.4 support, modern development tools, and quality-of-life improvements.

---

## What's Included

### Modern PHP Support
- PHP 8.3 & 8.4 with latest features
- No deprecation warnings
- Production-ready configurations

### Development Tools
- **Composer 2.9.5** - Dependency management
- **Laravel Installer 5.24.5** - Quick project creation
- **WP-CLI 2.12.0** - WordPress management
- **Pest 4.3.2** - Modern testing framework
- **Node.js 24.13.1** - JavaScript runtime
- **Bun 1.3.9** - Fast all-in-one toolkit
- **Vite 7.3.1** - Lightning-fast build tool
- **Vue CLI 5.0.9** - Vue.js framework
- **React CLI 5.0.1** - React framework (create-react-app)
- **Angular CLI 21.1.4** - Angular framework
- **Prettier 3.8.1** & **ESLint 10.0.0** - Code quality

### Database Services
- MySQL 8.0 / MariaDB
- PostgreSQL 16
- MongoDB 7
- Redis 7
- Memcached

### Updated Admin Tools (PHP 8.4 Compatible)
- **phpMyAdmin 5.2.3** - MySQL administration
- **Adminer 5.4.2** - Lightweight database manager (prefilled login)
- **phpPgAdmin 7.13.0** - PostgreSQL admin
- **phpCacheAdmin 2.4.1** - Redis/Memcached manager (unified)
- **OpCache GUI 3.6.0** - OpCache viewer

### Smart Features
- **Vhost Auto-Detection** - Automatically detects Laravel, WordPress, Symfony, CakePHP, Yii, CodeIgniter
- **Port Forwarding** - Direct `127.0.0.1` database connections from PHP
- **Interactive Setup Wizard** - 10-minute guided installation
- **Command Wrappers** - Run `composer`, `npm`, `artisan` directly on host
- **MCP Server** (Optional) - Claude Code integration for AI-assisted development

### Comprehensive Documentation
- Quick start guide
- Interactive setup wizard documentation
- Service-specific guides (Redis, Memcached, MySQL, PostgreSQL, MongoDB)
- Development guidelines for Claude Code
- Migration guide from vanilla Devilbox

---

## Quick Install

**For existing Devilbox installations:**

```bash
cd /path/to/devilbox
curl -sSL https://raw.githubusercontent.com/Drmzindec/Devilbox-Boost/main/install.sh | bash
```

**For new installations:**

See [Installation Guide](#installation) below.

---

## Installation

### Prerequisites

- macOS, Linux, or Windows (WSL2)
- Docker Desktop installed and running
- 8GB RAM minimum (16GB recommended)
- 20GB free disk space

### Option 1: New Devilbox Installation

```bash
# Clone official Devilbox
git clone https://github.com/cytopia/devilbox.git
cd devilbox

# Install Boost enhancements
curl -sSL https://raw.githubusercontent.com/Drmzindec/Devilbox-Boost/main/install.sh | bash

# Follow the interactive setup wizard
./setup-devilbox.sh
```

### Option 2: Upgrade Existing Devilbox

```bash
# Navigate to your Devilbox directory
cd /path/to/devilbox

# Install Boost (non-destructive)
curl -sSL https://raw.githubusercontent.com/Drmzindec/Devilbox-Boost/main/install.sh | bash

# Rebuild with modern tools
./docker-images/build-php.sh 8.4
docker compose up -d
```

See [MIGRATION.md](MIGRATION.md) for detailed upgrade instructions.

---

## Quick Start

After installation, create your first project:

### Laravel

```bash
docker compose exec php laravel new my-blog
# Wait 30 seconds for vhost auto-detection
# Visit: http://my-blog.local
```

### WordPress

```bash
docker compose exec php wp core download --path=my-site
# Visit: http://my-site.local
```

### Custom PHP

```bash
mkdir -p data/www/my-project
echo '<?php phpinfo();' > data/www/my-project/index.php
# Visit: http://my-project.local
```

See [QUICKSTART.md](QUICKSTART.md) for detailed tutorials.

---

## Access Points

| Service | URL |
|---------|-----|
| **Dashboard** | http://localhost |
| **phpMyAdmin** | http://localhost/vendor/phpmyadmin-5.2.3/ |
| **Adminer** | http://localhost/vendor/adminer-5.4.2-devilbox.php |
| **phpCacheAdmin** | http://localhost/vendor/phpcacheadmin-2.4.1/ |
| **OpCache GUI** | http://localhost/vendor/opcache-gui-3.6.0.php |

**Database Credentials:**
- Host: `127.0.0.1`
- Username: `root`
- Password: `root`

---

## Documentation

- [Quick Start Guide](QUICKSTART.md) - Get up and running in 10 minutes
- [Setup Wizard Guide](SETUP-WIZARD.md) - Complete walkthrough of interactive setup
- [Migration Guide](MIGRATION.md) - Upgrade from vanilla Devilbox
- [Phase 4 Plan](PHASE-4-PLAN.md) - Distribution strategy and roadmap
- [Modernization Roadmap](ROADMAP-MODERNIZATION.md) - Project history and features

### Development Guides

Located in `.claude/skills/`:
- [Using Redis](skills/using-redis.md) - Cache, sessions, queues
- [Using Memcached](skills/using-memcached.md) - High-performance caching
- [Using MySQL/MariaDB](skills/using-mysql.md) - Advanced SQL features
- [Using PostgreSQL](skills/using-postgresql.md) - JSONB, full-text search, CTEs
- [Using MongoDB](skills/using-mongodb.md) - Document store, aggregations

---

## Features Comparison

| Feature | Official Devilbox | Devilbox Boost |
|---------|-------------------|----------------|
| PHP 8.4 Support | ❌ (last update 2023) | ✅ Full support |
| Modern Tools | ❌ Outdated (Grunt/Gulp era) | ✅ Bun, Vite, Pest |
| AI Integration | ❌ None | ✅ MCP server for Claude Code |
| Easy Setup | ⚠️ Manual .env editing | ✅ Interactive wizard |
| Command Wrappers | ❌ Must use shell.sh | ✅ Direct host commands |
| Laravel Support | ⚠️ Manual vhost setup | ✅ Auto-detect & configure |
| Admin Tools | ⚠️ Showing PHP warnings | ✅ Latest PHP 8.4 compatible |
| Port Forwarding | ⚠️ Official images only | ✅ Built-in |
| Documentation | ⚠️ Outdated | ✅ Modern, comprehensive |
| Service Guides | ❌ None | ✅ Redis, Memcached, MySQL, PostgreSQL, MongoDB |

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
```

### Development

```bash
# Using command wrappers (if added to PATH)
composer install
npm install
artisan migrate
wp plugin list

# Or via docker compose exec
docker compose exec php composer install
docker compose exec php npm install
docker compose exec php php artisan migrate
docker compose exec php wp plugin list --allow-root
```

### Database

```bash
# Connect to MySQL
docker compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl

# Create database
docker compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl \
    -e "CREATE DATABASE my_database;"

# Redis CLI
docker compose exec php redis-cli -h 127.0.0.1
```

---

## Switching PHP Versions

Edit `.env`:

```bash
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

## Troubleshooting

### Docker Not Running

```bash
# Check Docker
docker info

# If error, start Docker Desktop
```

### Port 80 Already in Use

```bash
# Find process using port 80
sudo lsof -i :80

# Change port in .env
HOST_PORT_HTTPD=8000

# Restart
docker compose restart

# Access via http://localhost:8000
```

### .local Domains Don't Resolve

Add to `/etc/hosts`:

```bash
sudo nano /etc/hosts
```

Add:

```
127.0.0.1 my-project.local
```

Or use Devilbox DNS (port 1053).

### Database Connection Failed

Always use `127.0.0.1` not `localhost`:

```env
# ❌ Wrong
DB_HOST=localhost

# ✅ Correct
DB_HOST=127.0.0.1
```

---

## Optional: Claude Code Integration

Install the MCP server for AI-assisted Devilbox management:

```bash
cd mcp-server
./install.sh
```

This enables Claude Code to:
- Start/stop services
- List projects and databases
- Create databases
- View container health
- Manage vhosts

See [.claude/README.md](.claude/README.md) for details.

---

## What Gets Modified

Devilbox Boost is **non-destructive** and only adds/enhances:

### Added Files
- `docker-images/php-8.3-work/` - Custom PHP 8.3 image
- `docker-images/php-8.4-work/` - Custom PHP 8.4 image
- `docker-images/build-php.sh` - Image builder script
- `setup-devilbox.sh` - Interactive setup wizard
- `bin/` - Command wrappers (optional)
- `mcp-server/` - Claude Code integration (optional)
- `.claude/` - Development guidelines and skills

### Updated Files
- `.env` - Your configuration choices (via wizard)
- `.devilbox/www/` - Dashboard improvements and updated admin tools
- `docker-compose.override.yml` - Port forwarding configuration

### Never Modified
- Core Devilbox files remain untouched
- Your existing projects continue to work
- Can be uninstalled cleanly

---

## Uninstall

To remove Boost enhancements:

```bash
# Remove custom images
docker rmi devilbox-php-8.3:work
docker rmi devilbox-php-8.4:work

# Remove Boost files
rm -rf docker-images/php-8.3-work
rm -rf docker-images/php-8.4-work
rm -f docker-images/build-php.sh
rm -f setup-devilbox.sh
rm -rf bin
rm -rf mcp-server
rm -rf .claude

# Restore to official Devilbox
git checkout .env docker-compose.override.yml
docker compose up -d
```

Your projects and data remain untouched.

---

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## Support

- **Issues**: [GitHub Issues](https://github.com/Drmzindec/Devilbox-Boost/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Drmzindec/Devilbox-Boost/discussions)
- **Documentation**: Check docs in this repository
- **Original Devilbox**: [cytopia/devilbox](https://github.com/cytopia/devilbox)

---

## License

Devilbox Boost is released under the MIT License.

Based on [Devilbox](https://github.com/cytopia/devilbox) by cytopia.

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

---

## Acknowledgments

- **cytopia** and contributors for the original Devilbox
- The PHP community for modern tools
- Claude Code team for MCP protocol
- All beta testers and early adopters

---

## Roadmap

### Completed ✅
- PHP 8.3 & 8.4 support with modern tools
- Smart vhost auto-detection for multiple frameworks
- Updated admin tools (PHP 8.4 compatible)
- Interactive setup wizard
- MCP server for Claude Code
- Comprehensive documentation and guides

### Planned 🚀
- Additional framework support (Symfony, CakePHP improvements)
- Performance optimizations
- More admin tools
- Video tutorials
- Docker Hub images (optional)

---

**Devilbox Boost** - Making Devilbox modern, easy, and powerful.

Get started: `./setup-devilbox.sh`

---
