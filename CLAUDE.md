# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Devilbox?

Devilbox is a zero-configuration Docker-based PHP development stack supporting LEMP and MEAN architectures. It provides a reproducible development environment with automatic virtual host generation, SSL certificates, DNS resolution, and unlimited project support. The stack runs on all major platforms (Linux, macOS, Windows) with support for multiple versions of PHP (5.2-8.2), MySQL/MariaDB/Percona, PostgreSQL, MongoDB, Redis, and Memcached.

## Core Architecture

### Service Stack (docker-compose.yml)
- **bind** (172.16.238.100) - DNS server for wildcard domain resolution (*.dvl.to, *.loc)
- **php** (172.16.238.10) - PHP-FPM container (devilbox/php-fpm work image)
- **httpd** (172.16.238.11) - Web server (Apache 2.2/2.4 or Nginx)
- **mysql** (172.16.238.12) - MySQL/MariaDB/Percona database
- **pgsql** (172.16.238.13) - PostgreSQL database
- **redis** (172.16.238.14) - Redis cache
- **memcd** (172.16.238.15) - Memcached
- **mongo** (172.16.238.16) - MongoDB

All services communicate over a custom bridge network (172.16.238.0/24). The PHP container forwards internal service ports to localhost via the FORWARD_PORTS_TO_LOCALHOST mechanism.

### Directory Structure
- `.devilbox/www/` - Intranet dashboard (PHP web application)
  - `htdocs/` - Dashboard pages (vhosts.php, info_php.php, db_mysql.php, etc.)
  - `include/lib/` - Core PHP classes (Logger, Html, Helper)
  - `include/lib/container/` - Docker container interaction classes
  - `config.php` - Central configuration and class autoloader
- `cfg/` - Service configuration files organized by service and version
  - `php-ini-{VERSION}/` - PHP configuration overrides
  - `php-fpm-{VERSION}/` - PHP-FPM configuration overrides
  - `php-startup-{VERSION}/` - PHP startup scripts
  - `apache-*/`, `nginx-*/` - Web server configurations
  - `vhost-gen/` - Virtual host generation templates
- `data/www/` - Your web projects mount here (mass virtual hosting)
- `log/` - All service logs
- `ca/` - Certificate Authority and SSL certificates
- `backups/` - Database backup storage
- `autostart/` - Scripts executed on PHP container startup
- `bash/` - Custom bash configuration for containers
- `supervisor/` - Supervisord process management configs
- `compose/` - Docker Compose override examples for optional services (ELK, Mailhog, RabbitMQ, Solr, Varnish, etc.)
- `docs/` - Sphinx documentation
- `.tests/` - Test suite and CI/CD test cases

### Configuration System
The entire stack is configured via the `.env` file (created by copying `env-example`). Key variables:
- `PHP_SERVER` - PHP version (5.2, 5.3, 5.4, 5.5, 5.6, 7.0-7.4, 8.0-8.2)
- `HTTPD_SERVER` - Web server (apache-2.2, apache-2.4, nginx-stable, nginx-mainline)
- `MYSQL_SERVER` - Database (mysql-5.5 through mysql-8.0, mariadb-*, percona-*)
- `PGSQL_SERVER`, `REDIS_SERVER`, `MEMCD_SERVER`, `MONGO_SERVER`
- `TLD_SUFFIX` - Top-level domain for projects (default: dvl.to)
- `NEW_UID`, `NEW_GID` - Match container user to host user
- `TIMEZONE` - Container timezone
- `DEBUG_ENTRYPOINT` - Logging verbosity (0-4)

**IMPORTANT**: Do NOT edit `docker-compose.yml` directly. Use `docker-compose.override.yml` for customizations.

## Common Commands

### Starting/Stopping Services
```bash
# Start basic stack (httpd, php, mysql)
docker-compose up httpd php mysql

# Start with additional services
docker-compose up httpd php mysql pgsql redis

# Start all services in background
docker-compose up -d

# Stop all services
docker-compose stop

# Remove containers (required after .env changes)
docker-compose rm -f

# Pull latest Docker images
./update-docker.sh
```

### Entering the PHP Work Container
```bash
# Linux/macOS/WSL2
./shell.sh

# Windows (without WSL2)
./shell.bat

# Equivalent manual command
docker-compose exec --user devilbox php bash -l
```

Inside the container, you have access to: composer, node, npm, yarn, git, phpcs, phpunit, webpack, and many other development tools.

### Configuration Validation
```bash
# Check prerequisites and validate configuration
./check-config.sh
```

### Managing Projects
Projects are placed in `data/www/`. Each project directory becomes accessible as:
- `http://<project-name>.dvl.to` (if using dvl.to TLD)
- `http://<project-name>.loc` (if using loc TLD)

The document root within each project is determined by `HTTPD_DOCROOT_DIR` in `.env` (default: `htdocs/`).

## Development Workflow

### Making Configuration Changes
1. Edit values in `.env` file
2. Stop services: `docker-compose stop`
3. Remove containers: `docker-compose rm -f`
4. Restart: `docker-compose up httpd php mysql`

### Adding Custom PHP Configuration
Place `.ini` files in `cfg/php-ini-{VERSION}/` to override PHP settings. These are mounted as `/etc/php-custom.d/` in the container.

### Adding Custom Web Server Configuration
- Apache: Place `.conf` files in `cfg/apache-{VERSION}/`
- Nginx: Place `.conf` files in `cfg/nginx-{VERSION}/`

### Custom Virtual Host Templates
Edit templates in `cfg/vhost-gen/` to customize how virtual hosts are generated for your projects.

### Auto-Startup Scripts
Place executable `.sh` scripts in `autostart/` to run commands when the PHP container starts (e.g., starting Node.js applications, running background processes).

### Adding Optional Services
Copy a template from `compose/docker-compose.override.yml-*` to `docker-compose.override.yml`:
```bash
# Example: Add Mailhog email interceptor
cp compose/docker-compose.override.yml-mailhog docker-compose.override.yml
docker-compose up httpd php mysql mailhog
```

## Intranet Dashboard

Access at `http://localhost` after starting services. The dashboard provides:
- Virtual host listing and management
- PHP configuration viewer (phpinfo)
- Database management (Adminer, phpMyAdmin, pgAdmin)
- Redis/Memcached viewers
- Email interception viewer
- XDebug configuration
- OPcache status
- Service configuration viewers

### Intranet Code Structure
The intranet is a custom PHP application located in `.devilbox/www/`:
- Entry point: `config.php` (autoloads all classes)
- Core libraries: `include/lib/` (Logger, Html, Helper singletons)
- Container abstraction: `include/lib/container/` (BaseClass extended by service-specific classes)
- Pages in `htdocs/` use lazy-loaded classes via `config.php`

When modifying intranet code:
- Maintain singleton pattern for Logger, Html, Helper classes
- Container classes extend BaseClass and provide service-specific methods
- Follow existing logging patterns using Logger::getInstance()
- Configuration is version-pinned (currently v3.0.0-beta-0.4)

## Testing

### Running Tests
The test suite is in `.tests/`:
```bash
cd .tests
# Run specific test scripts
./scripts/test-<service>.sh
```

CI/CD workflows (`.github/workflows/`) test:
- PHP versions (test-php.yml)
- Web servers (test-httpd.yml)
- Databases (test-mysql.yml, test-pgsql.yml)
- Cache systems (test-redis.yml, test-memcd.yml, test-mongo.yml)
- Code linting (lint.yml)
- Documentation builds (documentation.yml)

## Important Notes

### Port Conflicts
Ensure ports 80 and 443 are not in use on the host before starting Devilbox. These are required for the web server.

### File Permissions
The `NEW_UID` and `NEW_GID` variables ensure the container user matches your host user, preventing permission issues with files created inside containers.

### DNS Resolution
- The built-in DNS server (bind) enables wildcard DNS (*.dvl.to, *.loc)
- Configure host to use 127.0.0.1:1053 for DNS, or use dvl.to which always resolves to 127.0.0.1
- See documentation for OS-specific DNS setup

### Database Persistence
Database data is stored in Docker named volumes (devilbox-mysql-*, devilbox-pgsql-*, etc.). To reset a database, remove its volume:
```bash
docker-compose down
docker volume rm devilbox-mysql-8.0  # or specific version
```

### SSL Certificates
Self-signed SSL certificates are auto-generated in `ca/`. Projects are accessible via both HTTP and HTTPS by default.

### XDebug Configuration
XDebug is pre-installed in PHP images. Configure via `cfg/php-ini-{VERSION}/xdebug.ini`. Default configuration is at `cfg/php-ini-{VERSION}/devilbox-default-xdebug.ini`.

## Version Information

Current release branch: `release/v3.0.0-beta-0.4`
Main branch: `master` (use for pull requests)
Local development branch: `local-dev-2026` (not pushed to remote)

Architecture support: amd64, arm64

## Local Development Updates (2026)

This installation has been updated with the latest service versions available as of February 2026:

### Service Versions
- **PHP**: 8.2 (configured for 8.3 and 8.4 once Docker images become available)
- **MariaDB**: 11.8 LTS (updated from 10.6)
- **PostgreSQL**: 18 (updated from 14)
- **Redis**: 8.6 (updated from 6.2)
- **MongoDB**: 8.0 (updated from 5.0)
- **Memcached**: 1.6 (no major changes needed)

### PHP 8.3 and 8.4 Support
Configuration directories have been prepared for PHP 8.3 and 8.4:
- `cfg/php-ini-8.3/` and `cfg/php-ini-8.4/`
- `cfg/php-fpm-8.3/` and `cfg/php-fpm-8.4/`
- `cfg/php-startup-8.3/` and `cfg/php-startup-8.4/`

These can be used once Devilbox releases official Docker images for these PHP versions.

### Database Volumes
Docker Compose volumes have been added for:
- MariaDB 10.11 through 11.8
- PostgreSQL 16, 17, and 18
- MongoDB 6.0, 7.0, and 8.0

### Important Notes
- **Do not push this branch** - it contains local customizations beyond official Devilbox releases
- Official Devilbox currently supports PHP up to 8.2; 8.3+ requires community or custom images
- All database engines support the latest versions via official Docker Hub images
