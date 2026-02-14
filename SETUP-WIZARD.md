# Devilbox Setup Wizard

The interactive terminal wizard that makes Devilbox setup effortless.

## Quick Start

```bash
./setup-devilbox.sh
```

The wizard will guide you through complete Devilbox setup in 10-20 minutes.

---

## What It Does

The setup wizard is a comprehensive interactive TUI (Terminal User Interface) that:

✅ **Checks Prerequisites** - Verifies Docker and Docker Compose are installed and running
✅ **Configures Environment** - Sets up `.env` file with your preferences
✅ **Builds Custom Images** - Creates PHP 8.3/8.4 images with modern tools
✅ **Starts Services** - Launches all Devilbox containers
✅ **Creates First Project** - Optionally scaffolds Laravel, WordPress, or custom PHP
✅ **Installs Tools** - Optionally sets up MCP server for Claude Code integration

---

## Step-by-Step Guide

### 1. Pre-flight Checks

The wizard automatically verifies:

- ✅ Docker is installed
- ✅ Docker is running
- ✅ Docker Compose is available
- ✅ `.env` file exists (creates from `env-example` if missing)

**No action required** - the wizard handles everything.

### 2. PHP Version Selection

Choose which PHP version(s) to install:

```
Which PHP version(s) do you want to use?
  1) PHP 8.4 only (recommended for new projects)
  2) PHP 8.3 only
  3) Both 8.3 and 8.4 (can switch between them)
```

**Recommendation**: Choose option 3 to have both versions available.

**What happens**:
- Updates `PHP_SERVER` in `.env`
- Determines which images to build later

### 3. Basic Configuration

Configure core Devilbox settings:

#### MySQL Root Password
```
MySQL root password [root]:
```

**Default**: `root`
**What it does**: Sets `MYSQL_ROOT_PASSWORD` in `.env`

#### HTTP Port
```
HTTP port (80 for localhost, 8000 for localhost:8000) [80]:
```

**Default**: `80`
**What it does**: Sets `HOST_PORT_HTTPD` in `.env`
**Options**:
- `80` - Access via `http://localhost`
- `8000` - Access via `http://localhost:8000`
- Any other port

#### HTTPS Port
```
HTTPS port [443]:
```

**Default**: `443`
**What it does**: Sets `HOST_PORT_HTTPS` in `.env`

#### TLD Suffix
```
TLD suffix for projects (e.g., .local, .test, .dev) [local]:
```

**Default**: `local`
**What it does**: Sets `TLD_SUFFIX` in `.env`
**Result**: Your projects will be accessible at `http://project-name.local`

**Popular Options**:
- `.local` - macOS/Linux friendly
- `.test` - Recommended for modern development
- `.dev` - Requires HTTPS (Chrome enforces)

### 4. Development Tools Setup

Add command wrappers to your PATH:

```
Add Devilbox command wrappers to your PATH? [Y/n]:
```

**Recommended**: `Y` (yes)

**What it does**:
- Detects your shell (zsh, bash, etc.)
- Adds `export PATH="/path/to/devilbox/bin:$PATH"` to shell config
- Enables convenient commands like:
  - `composer install` - runs in PHP container
  - `npm install` - runs in PHP container
  - `mysql` - connects to MySQL
  - `artisan` - Laravel Artisan
  - `wp` - WordPress CLI

**After setup**: Run `source ~/.zshrc` (or `~/.bashrc`) to activate immediately.

### 5. Build Custom PHP Images

Build PHP images with modern tools:

```
Build custom PHP images with modern tools?
Includes: Laravel, WP-CLI, Bun, Vite, Pest, React, Vue, Angular

This will take 10-15 minutes per image

Build images now? [Y/n]:
```

**Recommended**: `Y` (yes) if you have time

**What it does**:
- Builds custom PHP 8.3 and/or 8.4 Docker images
- Installs development tools:
  - **PHP**: Laravel Installer, WP-CLI, Composer, Pest
  - **Node**: Node.js 24, npm, Yarn, Bun
  - **Build Tools**: Vite, Webpack, Gulp, Grunt
  - **Frameworks**: React CLI, Vue CLI, Angular CLI
  - **Code Quality**: Prettier, ESLint

**Time**: 10-15 minutes per image

**Skip option**: You can build later with:
```bash
./docker-images/build-php.sh 8.4
./docker-images/build-php.sh 8.3
```

### 6. Start Devilbox

Launch all containers:

```
Start Devilbox now? [Y/n]:
```

**Recommended**: `Y` (yes)

**What it does**:
- Runs `docker compose up -d`
- Starts all services:
  - PHP-FPM (8.3 or 8.4)
  - Apache/Nginx web server
  - MySQL/MariaDB
  - PostgreSQL
  - Redis
  - Memcached
  - MongoDB
  - Bind DNS server

**Access points displayed**:
- Dashboard: `http://localhost`
- phpMyAdmin: `http://localhost/vendor/phpmyadmin-5.2.3/`
- Adminer: `http://localhost/vendor/adminer-5.4.2-devilbox.php`

### 7. Create Your First Project

Optionally create a starter project:

```
Create a starter project? [Y/n]:

What type of project?
  1) Laravel (PHP framework)
  2) WordPress (CMS)
  3) Custom PHP
  4) Skip
```

#### Option 1: Laravel
```
Project name (e.g., my-blog):
```

**What it does**:
- Runs `docker compose exec php laravel new <project-name>`
- Creates Laravel project in `data/www/<project-name>/`
- Vhost auto-detection creates Apache/Nginx config (30 seconds)
- Accessible at: `http://<project-name>.local`

#### Option 2: WordPress
```
Project name (e.g., my-site):
```

**What it does**:
- Downloads WordPress to `data/www/<project-name>/`
- Accessible at: `http://<project-name>.local`
- Complete installation in browser

#### Option 3: Custom PHP
```
Project name:
```

**What it does**:
- Creates directory with `index.php` (phpinfo)
- Accessible at: `http://<project-name>.local`

### 8. Optional: Claude Code Integration

Install MCP server for AI-assisted development:

```
Install MCP server for Claude Code integration?
This enables AI-assisted development with Devilbox

Install MCP server? [y/N]:
```

**Default**: `N` (no)

**What it does**:
- Runs `mcp-server/install.sh`
- Configures Claude Code to control Devilbox
- Enables AI commands for:
  - Starting/stopping services
  - Creating projects
  - Managing databases
  - Viewing logs

**Requirements**: Claude Code installed

---

## Completion Summary

After setup, you'll see:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🚀 Devilbox is ready!

Quick Reference:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Access Points:
  Dashboard:    http://localhost
  phpMyAdmin:   http://localhost/vendor/phpmyadmin-5.2.3/
  Adminer:      http://localhost/vendor/adminer-5.4.2-devilbox.php

Common Commands:
  docker compose ps              - View running containers
  docker compose logs -f php     - View PHP logs
  docker compose restart         - Restart all containers
  docker compose stop            - Stop Devilbox

Create New Projects:
  docker compose exec php laravel new my-project
  docker compose exec php wp core download --path=my-wp

Documentation:
  .claude/README.md              - Development guidelines
  .claude/skills/                - How-to guides
  ROADMAP-MODERNIZATION.md       - Project roadmap

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## What Gets Modified

The wizard modifies these files:

### `.env`
```bash
PHP_SERVER=8.4                    # Your selected PHP version
MYSQL_ROOT_PASSWORD=root          # Your MySQL password
HOST_PORT_HTTPD=80               # Your HTTP port
HOST_PORT_HTTPS=443              # Your HTTPS port
TLD_SUFFIX=local                 # Your TLD suffix
```

### Shell Config (`~/.zshrc` or `~/.bashrc`)
```bash
# Devilbox command wrappers
export PATH="/path/to/devilbox/bin:$PATH"
```

### Docker Images Built
- `devilbox-php-8.4:work` (if PHP 8.4 selected)
- `devilbox-php-8.3:work` (if PHP 8.3 selected)

---

## Troubleshooting

### "Docker is not running"

**Solution**: Start Docker Desktop and try again.

### "Port 80 already in use"

**Solution**:
1. Re-run the wizard
2. Choose a different HTTP port (e.g., 8000)
3. Access via `http://localhost:8000`

Or stop the service using port 80:
```bash
# Find what's using port 80
sudo lsof -i :80

# Stop it (macOS/Linux)
sudo kill <PID>
```

### "Build failed"

**Solution**: Build manually:
```bash
./docker-images/build-php.sh 8.4
```

Check logs for specific error messages.

### ".local domains don't resolve"

**Solution**: Configure DNS or use `/etc/hosts`:

```bash
sudo nano /etc/hosts
```

Add:
```
127.0.0.1 my-project.local
127.0.0.1 my-wordpress.local
```

### "Permission denied on PATH setup"

**Solution**: Manually add to your shell config:

```bash
echo 'export PATH="/path/to/devilbox/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

---

## Advanced Options

### Running Without Interactive Prompts

While the wizard is designed to be interactive, you can pre-configure `.env`:

```bash
# Copy and edit
cp env-example .env
nano .env

# Build images
./docker-images/build-php.sh 8.4

# Start manually
docker compose up -d
```

### Changing PHP Version Later

Edit `.env`:
```bash
PHP_SERVER=8.3  # or 8.4
```

Restart containers:
```bash
docker compose restart
```

### Re-running the Wizard

Safe to run multiple times:
- Existing `.env` values are used as defaults
- Won't rebuild images unless you confirm
- Won't overwrite existing PATH entries

---

## Time Estimates

| Step | Time | Can Skip? |
|------|------|-----------|
| Pre-flight checks | 5 seconds | No |
| Configuration | 1-2 minutes | No |
| Image building (both) | 20-30 minutes | Yes |
| Image building (one) | 10-15 minutes | Yes |
| Starting services | 30-60 seconds | Yes |
| Creating project | 1-2 minutes | Yes |
| MCP installation | 30 seconds | Yes |

**Total with image builds**: 20-35 minutes
**Total skipping builds**: 5-10 minutes

---

## What's Included in Custom Images

### PHP Tools
- Composer 2.9.5
- Laravel Installer 5.24.5
- WP-CLI 2.12.0
- Pest 4.3.2
- PHPUnit

### Node.js Ecosystem
- Node.js 24.13.1
- npm 11.8.0
- Yarn 1.22.22
- Bun 1.3.9

### Build Tools
- Vite 7.3.1
- Webpack 6.0.1
- Gulp 3.1.0
- Grunt 1.5.0

### Framework CLIs
- Vue CLI 5.0.9
- React CLI (create-react-app) 5.0.1
- Angular CLI 21.1.4
- Next.js (create-next-app)

### Code Quality
- Prettier 3.8.1
- ESLint 10.0.0

---

## Next Steps After Setup

1. **Access the dashboard**: http://localhost
2. **Check all services are running**: `docker compose ps`
3. **Create your first project** (if you skipped it):
   ```bash
   docker compose exec php laravel new my-app
   ```
4. **Wait 30 seconds** for vhost auto-detection
5. **Visit your project**: http://my-app.local

---

## Getting Help

- **Documentation**: `.claude/README.md`
- **Skills/Guides**: `.claude/skills/`
- **Issues**: Check `docker compose logs <service>`
- **Roadmap**: `ROADMAP-MODERNIZATION.md`

---

## Comparison: Manual vs Wizard Setup

### Manual Setup (Old Way)
```bash
# 1. Copy env file
cp env-example .env

# 2. Edit env (manual)
nano .env

# 3. Build images
./docker-images/build-php.sh 8.4
./docker-images/build-php.sh 8.3

# 4. Start services
docker compose up -d

# 5. Create project
docker compose exec php laravel new my-app

# 6. Configure vhost (manual)
# ... complex manual steps ...

# 7. Configure DNS/hosts (manual)
sudo nano /etc/hosts

# Time: 45-60 minutes
# Errors: Common
```

### Wizard Setup (New Way)
```bash
./setup-devilbox.sh

# Answer prompts
# Time: 10-20 minutes
# Errors: Rare
```

**Wizard advantages**:
- ✅ Guided experience
- ✅ Smart defaults
- ✅ Error handling
- ✅ Progress feedback
- ✅ No manual configuration
- ✅ Validates choices
- ✅ Auto-configures everything

---

Happy coding! 🚀
