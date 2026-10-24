# Changelog

All notable changes to Devilbox Boost will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned
- Additional framework support (Symfony, CakePHP improvements)
- Docker Hub images (optional)

---

## [1.0.8] - 2026-10-24

### Added
- **Companion: Doctor fix buttons** — findings the app can repair carry a one-click Fix (composer install, npm install + build, migrations, create a missing database, deactivate plugins missing from the repo, trust the CA, write the `/etc/resolver` file via an admin prompt, prune images); the check re-runs after each fix
- **Companion: Git panel** per project — changed files with per-file diffs, commit-all with a message, pull, push (sets the upstream on first push), branch switch/create, merge from another branch and a history card (last 30 commits, click for the patch). A conflicted merge stops cleanly with git's own words and an Abort button; there is deliberately no force-push. **Clone from git** on the Projects page clones straight into `data/www` (SSH URLs use the host's keys/agent)
- **Companion: repository browser** in the clone panel — GitLab (glab, per configured host), GitHub (gh) and Gitea (tea, per login) through their official CLIs so the app never touches a token; Bitbucket workspaces with an app password kept in the macOS Keychain. One-click install of the official CLIs and a login button that opens the interactive auth in your terminal
- **Companion: SSH view** — saved connections (`ssh.json`) plus the Host blocks from `~/.ssh/config`, list or cards, opened one click in an embedded terminal or handed to your terminal app. Optional per-connection **password in the macOS Keychain** (delivered via SSH_ASKPASS, never a file or argv) and **Install key** (one-click ssh-copy-id so the server moves to key auth). Export/Import of the connection list (passwords never leave the Keychain); native picker for the identity file
- **Companion: Xdebug toggle** on Overview — writes/removes the same `cfg/php-ini-<v>/zzz-mcp-xdebug.ini` drop-in the MCP server manages and reloads PHP-FPM
- **Companion: Scripts tab** — every composer.json and package.json script as a button, run inside the PHP container with streamed output
- **Companion: live logs** — the project Logs tab streams new lines (tail -F on app logs, `docker compose logs --follow` filtered per project) instead of polling
- **Companion: Databases create/drop** — create a utf8mb4 database, drop one with a type-the-name confirmation
- **Companion: web server switcher** on Overview — flip `HTTPD_SERVER` between the available `cfg/` servers (apache-2.4, nginx-stable, nginx-mainline) and recreate httpd in one click
- **Companion: launch at login** (tauri-plugin-autostart) and an **update check** comparing the newest GitHub release with the local checkout
- **MCP server v2.2: `devilbox_ssh_list` and `devilbox_ssh_exec`** — Claude can list the SSH connections the companion app knows (saved + `~/.ssh/config`, no credentials returned) and run non-interactive commands over them (key/agent auth, or the Keychain password via SSH_ASKPASS). Keep `devilbox_ssh_exec` permission-prompted rather than allowlisted: every remote command should be approved

### Changed
- **`tools/doctor.sh` checks the sites in parallel** (six at a time, results in the original order) — a full `dbox doctor` over a dozen sites drops from minutes to about half a minute
- Companion admin-tool links (Adminer, phpMyAdmin, phpCacheAdmin, OPcache GUI, phpPgAdmin) are discovered from the vendor folder instead of hardcoded versions
- Companion tray menu only rebuilds when its contents change, and the PHP crash-log scan runs every other poll
- Companion version now tracks the stack releases (1.0.8)

### Fixed
- **PHP work images never loaded the custom PHP-FPM pool config** — the stock `php-fpm.conf` only includes `/usr/local/etc/php-fpm.d/*.conf`, so `cfg/php-fpm-X.Y/devilbox-fpm.conf` (mounted at `/etc/php-fpm-custom.d`) was ignored and every install ran the default pool (`pm.max_children = 5`, no `request_terminate_timeout`). A handful of slow requests could pin all five workers and take every vhost down. The Dockerfiles now append the missing include; **rebuild your image** (`./docker-images/build-php.sh <version>` then `docker compose up -d --force-recreate php`). Workaround without a rebuild: `docker compose exec php sh -c 'printf "\ninclude=/etc/php-fpm-custom.d/*.conf\n" >> /usr/local/etc/php-fpm.conf; kill -USR2 1'`
- `cfg/php-fpm-8.4` / `8.5` pool configs gain `request_slowlog_timeout = 5s` with the slowlog on stderr, so stalled requests leave a PHP stack trace in `docker compose logs php`
- doctor.sh: a failed probe printed "000000" (curl already prints 000 on failure, the `|| echo 000` doubled it), so unreachable sites were reported as a WARN instead of a FAIL
- doctor.sh: `grep -c … || echo 0` produced "0\n0" when a log had no matches, breaking the count comparison; wp-cli's JSON `\/` escaping made every plugin in a folder look missing from disk
- Companion: reused tool windows (Adminer/phpMyAdmin) now navigate to the newly requested database instead of focusing the stale window; nvm Node versions sort numerically (v9 no longer beats v22); closed embedded terminals no longer leak `bash -l` processes in the PHP container; the MySQL root password is shell-quoted in import/list commands and percent-encoded in `mysql://` URLs; `.env` inline comments no longer leak into database names; a deliberate stack stop no longer races the poller into a false "container stopped" alert; the Files project selector matches exact paths

---

## [1.0.7] - 2026-10-23

### Security
- **PHP work images rebuilt on the current `php:8.3-fpm`, `php:8.4-fpm` and `php:8.5-fpm` bases** (PHP 8.4.27 security release of October 2026 plus refreshed Debian layers). `./docker-images/build-php.sh` now passes `--pull` so every rebuild starts from the newest upstream image. Run `./docker-images/build-php.sh 8.4` (and 8.3 / 8.5 if you use them) then `docker compose up -d --force-recreate php`

### Added
- **`dbox` command line** (`bin/dbox`) and the `tools/` engine it shares with the companion app: `dbox new <type> <name> [--preset …]` (Laravel incl. Livewire/React/Vue starter kits, Statamic, Symfony, WordPress, Bedrock, Drupal 11, Craft 5, CakePHP 5, CodeIgniter 4, TYPO3, Yii 2, Nette, Slim, Laminas, static), `dbox update [--check] [--scope wordpress|php|all]`, `dbox doctor`, `dbox snapshot <project> create|list|restore|prune`, `dbox workers <project> start|stop queue|schedule|horizon|reverb`, `dbox media <project> <prod-url>`, `dbox logs`, `dbox import`, `dbox shell`. Presets live in `tools/presets/*.sh` (Filament, Pest/Pint/Larastan/Boost, WordPress dev tools, WooCommerce) and `~/.config/devilbox-boost/presets/`
- **Companion project detail panel** — per project: Database (open in a detected SQL client, Adminer, phpMyAdmin, import a dump, take/restore/prune snapshots), Workers & dev server (queue/schedule/horizon/reverb inside the container, `npm run dev` on the host with live output), Media fallback (serve missing uploads from production, multisite wildcard host), Logs (web server and PHP errors for the host, application log, live), Terminal (a real bash inside the PHP container, in the project folder, via xterm.js)
- **Companion Doctor view** — stack checks (Docker, `.loc` resolver, CA trust, intranet, check-config, recent PHP-FPM segfaults, reclaimable image space) and per-site checks (reachability with the matching error line, missing or empty databases, active plugins missing or incomplete on disk, missing active theme, fatal errors in debug.log, the `email` gitignore trap, missing uploads, pending migrations)
- **Shared Composer and npm caches** — `cache/composer` and `cache/npm` are bind-mounted into the PHP container (`COMPOSER_CACHE_DIR`, `npm_config_cache`), so every project reuses downloads. `procps` added to the work images for process management
- **In-app help** — a `?` button next to every page title explains that page, and a Help page collects them together with the `dbox` commands and where files live
- **Detected editor, terminal and git apps** — Settings offers the editors (PhpStorm, WebStorm, VS Code, Cursor, Zed, Sublime Text, Nova, …), terminals (Terminal, iTerm, Warp, Ghostty, …) and git clients (Fork, Tower, GitHub Desktop, Sourcetree, …) that are actually installed, with a custom app name as fallback; the project overview shows a read-only git summary (branch, changed files, ahead/behind, last commit) with "Open in …" for the detected git client
- **Smarter file drops** — before copying, the Files view shows a plan: a folder dropped onto a same-named folder merges instead of nesting, an archive whose only top-level folder matches the destination is unpacked straight into it, replaced files are listed
- **Menu bar submenus** — Tools (every web tool, in its own window) and Projects (open a site) straight from the tray icon
- **Intranet navigation mirrors the app** — Overview, Projects, Databases (DB pages plus Adminer, phpMyAdmin, phpPgAdmin), Tools (Emails, Mailpit, phpCacheAdmin, OPcache GUI, Meilisearch, RabbitMQ, RustFS, Command & Control), Info, Logs, Config; icons and logo carry a version query so browsers pick up the new branding
- **SQL client detection** — the companion only offers Sequel Ace, TablePlus, DBeaver, Beekeeper Studio, DbGate or Querious when they are actually installed
- **Companion site operations** — an *Updates* view (check and update one site, "Update all WordPress", "Update all Laravel / Statamic", or everything, with a daily schedule that runs while the app is open and notifies on completion; WordPress runs plugin/theme/core updates plus the database upgrade through wp-cli, Composer projects run `composer update`, migrations and a frontend build), a *New project* view (Laravel, Statamic, Symfony, WordPress or static HTML created inside the PHP container with database, `.env`/`wp-config.php`, key, migrations, assets and `htdocs` symlink ready), and a *Files* view (browse project folders, drag and drop files or folders to copy them in, with optional extraction of `.zip`/`.tar.gz` archives, handy for `wp-content/uploads`)
- **Companion alerts, launcher and dump import** — desktop notifications when a container stops on its own or PHP-FPM workers crash (SIGSEGV lines in the php log), a global `Cmd+Shift+D` shortcut that opens the project launcher (type to filter, Enter opens the first match), and "Import DB…" on every project and on the Databases page: pick a `.sql` / `.sql.gz` / `.zip` dump and import it, with WordPress projects going through `tools/import-wp-dumps.sh`
- **`tools/import-wp-dumps.sh` and `tools/import-wp-multisite.sh`** — import production WordPress dumps into the stack: match dumps to projects (by name or a `map.txt`), detect table prefix and production siteurl, create or fix `wp-config.php`, pin `WP_HOME`/`WP_SITEURL` to `https://<project>.<tld>`, repair the `DEFAULT x AS` export quirk, and for multisite networks rewrite every blog domain to `<slug>.<project>.<tld>` with the right `MULTISITE` constants
- **Docker Hub workflow** (`.github/workflows/php-images.yml`) — builds the 8.3/8.4/8.5 work images on every `v*` tag and pushes multi-arch images as `<user>/devilbox-php:<version>` when `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN` secrets exist; builds only otherwise
- **Vhost auto-detection** now also maps `webroot/` (CakePHP), `www/` (Nette and friends) and a built `dist/index.html` (static SPAs) as the document root
- **New Devilbox Boost icon and banner** (`branding/`): a rounded "container with a bolt" mark on a violet-to-orange gradient replaces the original triangular Devilbox logo. Used by the companion app, the intranet favicon/navbar/login banner, the docs and the README. Source SVGs live in `branding/`; render with `resvg`
- **Companion app (`companion/`)** — a native Tauri 2 menu bar app for macOS: stack status with start/stop/restart, a project launcher for `data/www` (open site, editor, terminal, Finder), database list with one-click open in Sequel Ace / TablePlus (`mysql://` URL), Adminer or phpMyAdmin, and the bundled web tools (phpMyAdmin, Adminer, phpPgAdmin, phpCacheAdmin, OPcache GUI, Mailpit, RabbitMQ, Meilisearch, RustFS) each in their own window. The intranet pages are embedded in the main window. See `companion/README.md`
- **Dashboard vhost links follow your TLS setting** — `vhosts.php` now links projects as `https://<project>.<tld>` when `HTTPD_VHOST_SSL_TYPE` is `ssl`, `both` or `redir` (honouring `HOST_PORT_HTTPD_SSL`), and keeps `http://` for `plain`. New `Httpd::getVhostScheme()/getVhostPort()/getVhostUrl()` helpers
- `sockets` PHP extension in the 8.3 / 8.4 / 8.5 work images (needed by `pestphp/pest-plugin-browser` and other socket-based packages). Rebuild with `./docker-images/build-php.sh <version>`

### Changed
- **Opcache JIT is now off by default** in `cfg/php-ini-*/custom.ini`. With PHP 8.4.26 the tracing JIT (`1255`) segfaulted PHP-FPM workers on Laravel 13 apps and every request returned 502. Opcache itself stays enabled; re-enable JIT in `custom.ini` if you want to experiment

### Fixed
- MCP `devilbox_xdebug` / PHP-FPM reload used a `kill` binary that does not exist in the PHP image; it now runs `sh -c 'kill -USR2 1'`

---

## [1.0.6] - 2026-10-17

### Added
- **`check-config.sh` validates the `MYSQL_SERVER` / `MYSQL_IMAGE` pair** — the config dir, data volume and Docker image must now describe the same database release, so data can no longer silently live under a misleading volume name
- **bin/ wrappers defer to host binaries outside the Devilbox tree** — with `bin/` on your PATH, `node`, `npm`, `npx`, `php`, `composer` etc. now run the host's own binary when you are not inside the Devilbox directory (the container cannot see those paths anyway). Devilbox-only commands (`artisan`, `wp`, `dbox-shell`, ...) still run in the container
- **Full PHP 8.5 support** — PHP 8.5 is now stable and fully supported (`cfg/php-startup-8.5` added); wizard PHP menu offers 8.4 (recommended), 8.5 (newest stable), 8.3, or all of them
- **RustFS** — S3-compatible object storage replaces MinIO, whose community edition was archived upstream (April 2026). Same ports (9000 S3 API, 9001 console), new default credentials `rustfsadmin`/`rustfsadmin`. Existing MinIO bucket data does not migrate automatically; the old `devilbox-minio` volume is left intact
- **Configurable modern-service credentials** — `MEILI_MASTER_KEY`, `RABBIT_DEFAULT_USER`/`RABBIT_DEFAULT_PASS`, `RUSTFS_ACCESS_KEY`/`RUSTFS_SECRET_KEY` and friends can now be overridden in `.env` (commented defaults in `env-example`)
- New `bin/` commands: `tinker`, `pint`, `phpstan`, `phpunit`, `npx`, `dbox-shell`

### Changed
- **MCP server migrated to the MCP TypeScript SDK v2** (`@modelcontextprotocol/server` 2.3.x, server v2.1.0). Tool definitions are unchanged; inputs are now schema-validated by the SDK before a handler runs. Requires **Node.js 20+** (`install.sh` checks this). Re-run `npm install` in `mcp-server/` after updating
- **New defaults: MongoDB 9.0 and Redis `8-alpine`** (currently 8.10). `MONGO_SERVER=8.2` / `8.0` / `7.0` remain selectable
- **Admin tools updated** — Adminer 6.1.2 (was 5.4.2), phpCacheAdmin 2.7.2 (was 2.4.1), OpCache GUI 3.6.1 (was 3.6.0). Old versions removed from `.devilbox/www/htdocs/vendor/`
- **Official MariaDB/MySQL images** — MySQL service now runs official images via the new `MYSQL_SERVER` (config dir + data volume name) / `MYSQL_IMAGE` (Docker image) pair. Default is MariaDB 11.8 LTS with `MARIADB_AUTO_UPGRADE=1`, so pointing a newer image at an old data volume upgrades it in place. MariaDB 10.6 (EOL July 2026) and the legacy `devilbox/mysql` image were removed, see MIGRATION.md
- **Default PHP version is now 8.4** (was 8.2)
- **`bin/` wrappers rewritten as a single dispatcher** — run as the devilbox user and honor `docker-compose.override.yml` files
- **MCP server v2.0 security rewrite** — 16 tools, no shell execution, service allowlist
- `compose/docker-compose.override.yml-minio` renamed to `compose/docker-compose.override.yml-rustfs`. **Existing `.env` files must update `FORWARD_PORTS_TO_LOCALHOST`** from `9100:minio:9000,9101:minio:9001` to `9100:rustfs:9000,9101:rustfs:9001`, otherwise the PHP container's port forwarding for object storage silently fails

### Removed (end-of-life)
Boost only ships PHP 8.3+ custom images, so everything that existed solely for upstream's legacy images is gone:
- `env-example` options, `cfg/` and `log/` directories and compose volume definitions for PHP ≤ 8.2, Apache 2.2, MariaDB ≤ 10.11 (incl. the old 10.6 default), MySQL 5.x / 8.0, Percona, PostgreSQL ≤ 14, Redis ≤ 7.2, Memcached 1.4/1.5 and MongoDB ≤ 6.0
- Compose templates `php-community`, `php-multi` (PHP 5.4–8.2 side-by-side) and `mailhog` (replaced by Mailpit); the `-all` template no longer overrides the PHP image
- Legacy dashboard tools only reachable from PHP < 8: phpMyAdmin 4.0 / 4.9.7 / 5.1.3, phpPgAdmin 5.6.0 / 7.12.1, Adminer 4.6.3 (≈ 90 MB less in the repo)
- Upstream CI matrices (`params-*.yml`, `test-*.yml`) that tested devilbox/php-fpm 5.3–8.2 images Boost never ran
- See MIGRATION.md if your `.env` still references a removed `MYSQL_SERVER`

### Fixed
- Dashboard version string was still reporting v1.0.5
- Setup wizard now writes the correct HTTPS port variable (`HOST_PORT_HTTPD_SSL`; the previously documented `HOST_PORT_HTTPS` never existed)
- Setup wizard zsh PATH detection
- Xdebug recipe files now use `client_port` 9003 (9000 collided with PHP-FPM)

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
