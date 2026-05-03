# Changelog

All notable changes to Devilbox Boost will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned
- Additional framework support (Symfony, CakePHP improvements)
- Performance optimizations
- Video tutorials  
- Docker Hub images (optional)

---

## [1.0.5] - 2026-05-03

### Added
- **Auto-docroot detection** — projects dropped in `data/www/` auto-detect their document root (`public/` for Laravel/Symfony, `web/` for Drupal, `pub/` for Magento, root for WordPress/generic). No manual `htdocs` symlinks or configuration needed
- **MCP server for Claude Code CLI** — `.mcp.json` project-level config, installer now configures both Claude Code CLI and Desktop
- **Auto-DNS documentation** — one-time resolver setup for macOS, Linux, and Windows

### Changed
- **Default TLD changed from `local` to `loc`** — `.local` is reserved by macOS for mDNS/Bonjour and does not resolve in browsers. All documentation updated to use `.loc`
- **Simplified vhost auto-configure** — removed broken per-project nginx/apache config generation that conflicted with stock httpd watcherd. Now only handles htdocs symlink creation
- **Custom PHP entrypoint** — now executes scripts from `/startup.2.d/` (autostart directory) before starting PHP-FPM
- **bin/ wrappers fixed** — all 15 host-side command wrappers (`composer`, `artisan`, `php`, `npm`, etc.) now use `docker compose exec` with dynamic path detection instead of hardcoded container names
- MCP server uses `docker compose exec -T php` instead of hardcoded `devilbox-php-1` container name
- MCP server installer updated for both Claude Code CLI (`.mcp.json`) and Desktop (`claude_desktop_config.json`)

### Fixed
- `bin/` wrappers failing with "No such container: devilbox-php-1" when project directory is not named "devilbox"
- `bin/mysql` wrapper had hardcoded path to wrong `.env` location
- MCP server `docker-compose` (hyphenated) replaced with `docker compose` (space-separated)
- Projects showing Devilbox dashboard instead of actual site content due to broken vhost config generation

---

## [1.0.4] - 2026-05-01

### Added
- **PHP Error Log Viewer** — live-tailing log page at Info > PHP Error Log with color-coded output, pause/resume, adjustable refresh, and clear functionality
- **PHPStan, Rector, Deno** — pre-installed in all custom PHP images and detected on dashboard
- **VS Code Dev Container** — `.devcontainer/devcontainer.json` for remote development

### Improved
- Complete documentation audit: fixed outdated PHP versions, service versions, broken links, old `docker-compose` syntax, and incorrect paths across 15+ files
- Credits page updated for Devilbox Boost with correct attributions and library versions
- Footer GitHub link now points to Devilbox Boost repository
- Modern services environment variables passed to PHP container (eliminates 60 debug errors)
- Default TLD changed from `dvl.to` to `local` in `env-example`
- Replaced deprecated SwiftMailer code example with Symfony Mailer in docs
- Removed hardcoded personal paths from all documentation
- Replaced outdated `docs/CLAUDE.md` with redirect to root `CLAUDE.md`
- Roadmap updated to reflect v1.0.3 completed items and new plans

### Fixed
- Rector version detection failing when run as non-root user (cache permission issue, falls back to composer package version)
- `docker compose restart` in docs replaced with full recreate command where `.env` changes require it

---

## [1.0.3] - 2026-05-01

### Added
- `install.sh` — one-line installer for new and existing Devilbox setups
- PHP 8.5 support with custom Docker image (`docker-images/php-8.5-work/`)
- PHP 8.5 configuration directories (`cfg/php-ini-8.5/`, `cfg/php-fpm-8.5/`, `cfg/php-startup-8.5/`)
- PHP 8.5 option in `env-example`

---

## [1.0.2] - 2026-05-01

### Added
- Quick Start connection snippets for MariaDB, PostgreSQL, Redis, Memcached, and MongoDB (Laravel, WordPress, PHP examples)
- `docker-compose.lean.yml` for minimal 4-container mode (PHP, Nginx, MariaDB, Redis)
- PHP-FPM pool tuning (`cfg/php-fpm-8.4/devilbox-fpm.conf`)
- Modern services baked into main `docker-compose.yml` (Meilisearch, Mailpit, RabbitMQ, MinIO)

### Improved
- PHP 8.4 OPcache JIT configuration for development performance
- Dashboard layout: CLI Tools and PHP Container Status panels now fill full width
- Webpack version detection updated for webpack-cli 7.x compatibility

### Fixed
- Modern Services dashboard: MinIO card incorrectly displayed as RabbitMQ (PHP foreach-by-reference bug)
- Modern Services health checks: services now detected via Docker network hostnames instead of localhost

---

## [1.0.0] - 2026-02-14

### Added
- **PHP 8.3 & 8.4 Support** with custom Docker images
- **Modern Development Tools**:
  - Composer 2.9.5
  - Laravel Installer 5.24.5
  - WP-CLI 2.12.0
  - Pest 4.3.2 (Modern testing framework)
  - Node.js 24.13.1
  - Bun 1.3.9 (Fast JavaScript toolkit)
  - Vite 7.3.1 (Lightning-fast build tool)
  - Vue CLI 5.0.9
  - React CLI 5.0.1 (create-react-app)
  - Angular CLI 21.1.4
  - Prettier 3.8.1 & ESLint 10.0.0
  
- **Updated Admin Tools** (PHP 8.4 Compatible):
  - phpMyAdmin 5.2.3
  - Adminer 5.4.2 with prefilled login (127.0.0.1, root:root)
  - phpPgAdmin 7.13.0
  - phpCacheAdmin 2.4.1 (unified Redis/Memcached manager)
  - OpCache GUI 3.6.0

- **Smart Features**:
  - Vhost auto-detection for Laravel, WordPress, Symfony, CakePHP, Yii, CodeIgniter
  - Port forwarding for direct `127.0.0.1` database connections from PHP container
  - Interactive setup wizard (`setup-devilbox.sh`)
  - Command wrappers for running tools directly on host
  - Background service for automatic vhost configuration

- **MCP Server** (Optional):
  - Claude Code integration for AI-assisted development
  - 10 tools for service management, database operations, health checks
  - Automated installer

- **Comprehensive Documentation**:
  - Quick Start Guide (QUICKSTART.md)
  - Setup Wizard Guide (SETUP-WIZARD.md)
  - Migration Guide (MIGRATION.md)
  - Service-specific guides (Redis, Memcached, MySQL, PostgreSQL, MongoDB)
  - Claude Code development guidelines
  - Contributing guidelines

### Fixed
- Dashboard tool detection for WP-CLI, Pest, and other tools
- File permission issues for PHP-FPM accessing root-installed tools
- MySQL SSL connection errors
- Adminer session management issues
- Docker Compose version attribute warning
- Table layout issues in db_redis.php and info_mongo.php

### Changed
- Removed obsolete `version: '2.3'` from docker-compose.yml
- Improved dashboard UX with categorized tools table
- Enhanced OpCache GUI to open in new tab
- Updated phpCacheAdmin configuration with writable cache directories
- Improved .gitignore to exclude vendor archives

### Security
- Updated all admin tools to PHP 8.4 compatible versions
- No more deprecation warnings or security vulnerabilities

---

## Background

Devilbox Boost modernizes the official [Devilbox](https://github.com/cytopia/devilbox) with:
- PHP 8.3/8.4/8.5 support
- Modern development tools
- Quality-of-life improvements
- AI integration capabilities
- Comprehensive documentation

Based on Devilbox by cytopia, enhanced for modern PHP development.

---

## Contributors

- Devilbox Boost Contributors - Initial modernization and development
- Claude Code - AI-assisted development and documentation
- Devilbox community - Original project and inspiration

---

[Unreleased]: https://github.com/Drmzindec/Devilbox-Boost/compare/v1.0.4...HEAD
[1.0.4]: https://github.com/Drmzindec/Devilbox-Boost/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/Drmzindec/Devilbox-Boost/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/Drmzindec/Devilbox-Boost/compare/v1.0.0...v1.0.2
[1.0.0]: https://github.com/Drmzindec/Devilbox-Boost/releases/tag/v1.0.0
