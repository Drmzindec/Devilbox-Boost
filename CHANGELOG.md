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
- PHP 8.3/8.4 support
- Modern development tools
- Quality-of-life improvements
- AI integration capabilities
- Comprehensive documentation

Based on Devilbox by cytopia, enhanced for modern PHP development.

---

## Contributors

- Johan Pretorius - Initial modernization and development
- Claude Code - AI-assisted development and documentation
- Devilbox community - Original project and inspiration

---

[Unreleased]: https://github.com/OWNER/devilbox-boost/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/OWNER/devilbox-boost/releases/tag/v1.0.0
