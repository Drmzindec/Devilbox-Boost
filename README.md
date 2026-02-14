<div align="center">

# 🚀 Devilbox Boost

**Modern PHP 8.3/8.4 development stack powered by Docker**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![PHP](https://img.shields.io/badge/PHP-8.3%20%7C%208.4-777BB4?logo=php&logoColor=white)](https://www.php.net/)
[![Docker](https://img.shields.io/badge/Docker-required-2496ED?logo=docker&logoColor=white)](https://www.docker.com/)
[![Laravel](https://img.shields.io/badge/Laravel-ready-FF2D20?logo=laravel&logoColor=white)](https://laravel.com/)
[![WordPress](https://img.shields.io/badge/WordPress-ready-21759B?logo=wordpress&logoColor=white)](https://wordpress.org/)

[Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Contributing](#-contributing)

---

**Devilbox Boost** modernizes the official [Devilbox](https://github.com/cytopia/devilbox) with PHP 8.3/8.4 support, modern development tools (Bun, Vite, Pest), and quality-of-life improvements.

</div>

---

## ⚡ Quick Start

**New installation:**
```bash
git clone https://github.com/cytopia/devilbox.git && cd devilbox
curl -sSL https://raw.githubusercontent.com/Drmzindec/Devilbox-Boost/main/install.sh | bash
./setup-devilbox.sh  # Interactive wizard
```

**Upgrade existing Devilbox:**
```bash
cd /path/to/devilbox
curl -sSL https://raw.githubusercontent.com/Drmzindec/Devilbox-Boost/main/install.sh | bash
```

**First project:**
```bash
docker compose exec php laravel new my-blog
# Visit http://my-blog.local (auto-configured!)
```

📖 **Detailed guide:** [QUICKSTART.md](QUICKSTART.md)

---

## ✨ Features

### 🐘 Modern PHP Stack
- **PHP 8.3 & 8.4** - Zero deprecation warnings
- **Laravel Installer** - Instant Laravel projects
- **WP-CLI** - WordPress management
- **Pest** - Modern testing framework
- **Composer 2.9.5** - Latest dependency manager

### 🛠️ Modern JavaScript Tools
- **Bun 1.3.9** - Fast all-in-one toolkit
- **Vite 7.3.1** - Lightning-fast builds
- **Node.js 24** - Latest LTS
- **Vue/React/Angular CLIs** - Framework scaffolding
- **Prettier & ESLint** - Code quality

### 🗄️ Database & Caching
- **MySQL 8.0** / MariaDB
- **PostgreSQL 16**
- **MongoDB 7**
- **Redis 7**
- **Memcached**

### 🎯 Smart Automation
- **Auto Vhost Detection** - Laravel, WordPress, Symfony auto-configured
- **Port Forwarding** - Use `127.0.0.1` for database connections
- **Setup Wizard** - 10-minute interactive configuration
- **Command Wrappers** - Run `composer`, `artisan`, `npm` directly on host

### 🎨 Updated Admin Tools
- **phpMyAdmin 5.2.3** - Pre-filled login (127.0.0.1, root:root)
- **Adminer 5.4.2** - Lightweight database manager
- **phpCacheAdmin 2.4.1** - Redis/Memcached unified UI
- **OpCache GUI 3.6.0** - Performance monitoring
- **phpPgAdmin 7.13.0** - PostgreSQL admin

### 🤖 AI Integration (Optional)
- **MCP Server** - Claude Code integration
- **10+ tools** - Service management, database ops, health checks
- **Automated workflows** - AI-assisted development

### ⚡ Modern Services (Optional)
- **Meilisearch** - Lightning-fast search engine
- **Mailpit** - Modern email testing (replaces Mailhog)
- **RabbitMQ** - Message queue for async tasks
- **MinIO** - S3-compatible object storage

---

## 📊 Devilbox vs Devilbox Boost

| Feature | Devilbox | Devilbox Boost |
|---------|----------|----------------|
| **PHP 8.4 Support** | ❌ | ✅ |
| **Modern Tools** | ⚠️ Outdated (2023) | ✅ Bun, Vite, Pest |
| **Setup Time** | ⏱️ 1-2 hours manual | ⏱️ 10 minutes wizard |
| **Laravel Auto-Config** | ❌ Manual vhost | ✅ Auto-detected |
| **WordPress Support** | ⚠️ Manual setup | ✅ WP-CLI + auto-config |
| **Admin Tools** | ⚠️ PHP warnings | ✅ PHP 8.4 compatible |
| **AI Integration** | ❌ | ✅ Claude Code MCP |
| **Documentation** | ⚠️ Outdated | ✅ Comprehensive guides |

---

## 💻 Usage Examples

### Create Laravel Project
```bash
docker compose exec php laravel new my-app
# Auto-detected, configured, ready at http://my-app.local
```

### Create WordPress Site
```bash
docker compose exec php wp core download --path=my-site
# Visit http://my-site.local to complete setup
```

### Use Modern Tools
```bash
docker compose exec php bun install    # Fast package install
docker compose exec php vite build     # Lightning builds
docker compose exec php pest           # Modern testing
```

### Database Operations
```bash
# MySQL
docker compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl

# Redis
docker compose exec php redis-cli -h 127.0.0.1

# PostgreSQL
docker compose exec php psql -h 127.0.0.1 -U postgres
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [QUICKSTART.md](QUICKSTART.md) | 10-minute setup guide |
| [SETUP-WIZARD.md](SETUP-WIZARD.md) | Interactive wizard walkthrough |
| [MIGRATION.md](MIGRATION.md) | Upgrade from vanilla Devilbox |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guidelines |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [Roadmap](docs/ROADMAP.md) | Future features and milestones |

### Service Guides
- [Modern Services](docs/MODERN-SERVICES.md) - Meilisearch, Mailpit, RabbitMQ, MinIO
- [Using Redis](.claude/skills/using-redis.md) - Cache, sessions, queues
- [Using Memcached](.claude/skills/using-memcached.md) - High-performance caching
- [Using MySQL](.claude/skills/using-mysql.md) - Advanced SQL features
- [Using PostgreSQL](.claude/skills/using-postgresql.md) - JSONB, full-text search
- [Using MongoDB](.claude/skills/using-mongodb.md) - Document store operations

---

## 🔧 Requirements

- **OS:** macOS, Linux, or Windows (WSL2)
- **Docker:** Desktop 20.10+
- **RAM:** 8GB minimum (16GB recommended)
- **Disk:** 20GB free space

---

## 📥 Installation Options

### Option 1: New Installation (Recommended)

```bash
# 1. Clone Devilbox
git clone https://github.com/cytopia/devilbox.git
cd devilbox

# 2. Install Boost
curl -sSL https://raw.githubusercontent.com/Drmzindec/Devilbox-Boost/main/install.sh | bash

# 3. Run wizard
./setup-devilbox.sh
```

### Option 2: Upgrade Existing Devilbox

```bash
cd /path/to/devilbox
curl -sSL https://raw.githubusercontent.com/Drmzindec/Devilbox-Boost/main/install.sh | bash
./docker-images/build-php.sh 8.4
docker compose up -d
```

### Option 3: Manual Installation

See [MIGRATION.md](MIGRATION.md) for detailed manual setup.

---

## 🎮 Access Points

| Service | URL |
|---------|-----|
| **Dashboard** | http://localhost |
| **phpMyAdmin** | http://localhost/vendor/phpmyadmin-5.2.3/ |
| **Adminer** | http://localhost/vendor/adminer-5.4.2-devilbox.php |
| **phpCacheAdmin** | http://localhost/vendor/phpcacheadmin-2.4.1/ |
| **OpCache GUI** | http://localhost/vendor/opcache-gui-3.6.0.php |

**Database Credentials:**
```
Host: 127.0.0.1
Username: root
Password: root
```

---

## 🚀 What's New in v1.0.0

- ✅ **PHP 8.3 & 8.4** with custom Docker images
- ✅ **Modern tools** - Bun, Vite, Pest, latest Node.js
- ✅ **Auto vhost detection** - Laravel, WordPress, Symfony
- ✅ **Updated admin tools** - All PHP 8.4 compatible
- ✅ **Interactive wizard** - 10-minute setup
- ✅ **MCP server** - Claude Code AI integration
- ✅ **Comprehensive docs** - Guides for all services
- ✅ **Port forwarding** - Direct 127.0.0.1 connections
- ✅ **Command wrappers** - Run tools from host

See [CHANGELOG.md](CHANGELOG.md) for complete details.

---

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Bug reports
- Feature requests
- Pull request process
- Development setup

---

## 🐛 Troubleshooting

### Port 80 in use?
```bash
# Use different port
echo "HOST_PORT_HTTPD=8000" >> .env
docker compose restart
# Access via http://localhost:8000
```

### Database connection failed?
Always use `127.0.0.1` not `localhost`:
```env
DB_HOST=127.0.0.1  # ✅ Correct
DB_HOST=localhost  # ❌ Won't work
```

### Project shows 404?
Wait 30 seconds for vhost auto-detection:
```bash
sleep 30
docker compose restart httpd
```

More help: [QUICKSTART.md#troubleshooting](QUICKSTART.md#troubleshooting)

---

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.

Based on [Devilbox](https://github.com/cytopia/devilbox) by [cytopia](https://github.com/cytopia).

---

## 🌟 Show Your Support

If Devilbox Boost helps your development workflow:
- ⭐ Star this repository
- 🐛 Report bugs and suggest features
- 🤝 Contribute improvements
- 📣 Share with other developers

---

## 🔗 Links

- **Issues:** [Report bugs](https://github.com/Drmzindec/Devilbox-Boost/issues)
- **Discussions:** [Q&A and ideas](https://github.com/Drmzindec/Devilbox-Boost/discussions)
- **Original Devilbox:** [cytopia/devilbox](https://github.com/cytopia/devilbox)

---

<div align="center">

**Built with ❤️ for the PHP community**

[Get Started](#-quick-start) • [View Docs](#-documentation) • [Contribute](#-contributing)

</div>
