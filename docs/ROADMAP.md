# Devilbox Boost Roadmap

**Status:** v1.0.3 Released - May 2026

---

## 🎯 Vision

Create a modern, developer-friendly Docker-based PHP development stack that:
- Supports latest PHP versions (8.3, 8.4, 8.5)
- Includes contemporary development tools
- Provides excellent developer experience
- Maintains backwards compatibility with Devilbox

---

## ✅ Completed - v1.0.0 through v1.0.3

### Core Modernization
- [x] **PHP 8.3, 8.4 & 8.5 Support** - Custom Docker images with zero deprecation warnings
- [x] **Modern JavaScript Tools** - Bun, Vite, Deno, latest Node.js, Vue/React/Angular CLIs
- [x] **Updated Admin Tools** - All PHP 8.4 compatible (phpMyAdmin 5.2.3, Adminer 5.4.2, phpCacheAdmin 2.4.1)
- [x] **Port Forwarding** - Direct `127.0.0.1` database connections from PHP container
- [x] **PHPStan & Rector** - Static analysis and automated refactoring pre-installed
- [x] **Prettier & ESLint** - Code quality tools pre-installed

### Automation & UX
- [x] **Interactive Setup Wizard** - 10-minute guided installation
- [x] **Smart Vhost Auto-Detection** - Laravel, WordPress, Symfony, CakePHP, Yii, CodeIgniter
- [x] **Command Wrappers** - Run tools directly on host (`composer`, `artisan`, `npm`)
- [x] **Dashboard Improvements** - Better organization, category headers, improved layouts
- [x] **One-Line Installer** - `install.sh` for new and existing Devilbox setups
- [x] **VS Code Dev Container** - `.devcontainer/devcontainer.json` for remote development
- [x] **Health Checks** - All 12 services have Docker health checks
- [x] **Restart Policies** - All services auto-restart on failure

### Modern Services (Built-in)
- [x] **Meilisearch** - Fast search engine with typo-tolerance
- [x] **Mailpit** - Modern email testing (replaces Mailhog)
- [x] **RabbitMQ** - Message queue for async tasks
- [x] **MinIO** - S3-compatible object storage
- [x] **Lean Mode** - `docker-compose.lean.yml` for minimal 4-container setup

### AI Integration
- [x] **MCP Server** - Claude Code integration with 10+ tools
- [x] **Service Management** - AI-assisted container control
- [x] **Database Operations** - Create databases, list services via AI
- [x] **Health Monitoring** - Automated health checks and diagnostics

### Documentation
- [x] **Quick Start Guide** - 10-minute setup tutorial
- [x] **Setup Wizard Guide** - Complete walkthrough
- [x] **Migration Guide** - Upgrade from vanilla Devilbox
- [x] **Service Guides** - Redis, Memcached, MySQL, PostgreSQL, MongoDB best practices
- [x] **Contributing Guidelines** - Development setup and standards

---

## 🚀 Planned - v1.1.0 (Q3 2026)

### Enhanced Framework Support
- [ ] **Symfony Improvements** - Better auto-detection and configuration
- [ ] **CakePHP Templates** - Optimized vhost templates
- [ ] **Yii Framework** - Dedicated documentation and examples
- [ ] **CodeIgniter** - Enhanced support

### Performance Optimizations
- [ ] **Image Size Reduction** - Optimize Docker images (current: ~1.5GB)
- [ ] **Build Time Improvements** - Reduce image build time from 10-15 min
- [ ] **Caching Strategies** - Docker layer caching for faster rebuilds
- [ ] **Resource Optimization** - Lower memory footprint

### Additional Tools
- [ ] **Turbo** - Incremental bundler for JavaScript/TypeScript

### Admin Tools
- [ ] **MongoDB Admin UI** - Modern MongoDB management interface
- [ ] **RabbitMQ Management** - Queue management interface (already included via management tag)

### Developer Experience
- [ ] **PhpStorm Settings** - Pre-configured project settings
- [ ] **Git Hooks** - Pre-commit hooks for code quality
- [ ] **Health Check Dashboard** - Visual service health monitoring
- [ ] **Windows Setup Script** - setup-devilbox.bat for native Windows support

---

## 🔮 Future (v2.0.0+)

### Architecture Improvements
- [ ] **FrankenPHP** - Optional Caddy-based PHP server (replaces PHP-FPM + Nginx)
- [ ] **Multi-Platform Builds** - Native ARM64 support (Apple Silicon)
- [ ] **Podman Support** - Full Podman compatibility as Docker alternative

### New Services
- [ ] **Elasticsearch/OpenSearch** - Advanced search and analytics

### Enhanced AI Features
- [ ] **Natural Language Commands** - "Create a Laravel project with authentication"
- [ ] **Auto-Debugging** - AI-assisted error resolution

### Advanced Features
- [ ] **Multiple PHP Versions Simultaneously** - Run different PHP versions per project
- [ ] **Project Templates** - One-command project scaffolding
- [ ] **Backup & Restore** - Automated project and database backups

---

## 🤝 Community Requests

**Want something not on the roadmap?**

1. Check [existing issues](https://github.com/Drmzindec/Devilbox-Boost/issues)
2. Open a [feature request](https://github.com/Drmzindec/Devilbox-Boost/issues/new)
3. Join the [discussion](https://github.com/Drmzindec/Devilbox-Boost/discussions)

---

## 📝 Changelog

See [CHANGELOG.md](../CHANGELOG.md) for detailed version history.

---

## 💡 Contributing

Want to help shape the future of Devilbox Boost?

- **Code Contributions:** See [CONTRIBUTING.md](../CONTRIBUTING.md)
- **Bug Reports:** [Open an issue](https://github.com/Drmzindec/Devilbox-Boost/issues)
- **Feature Suggestions:** [Start a discussion](https://github.com/Drmzindec/Devilbox-Boost/discussions)
- **Documentation:** Help improve our guides

---

**Last Updated:** May 1, 2026
**Current Version:** v1.0.3
