# Devilbox - AI Coding Assistant Guidelines

This directory contains guidelines and skills for AI coding assistants working with Devilbox.

## Project Overview

**Devilbox** is a modern PHP development stack based on Docker. This repository contains an enhanced version with:

- **Smart Vhost Auto-Detection** - Automatically detects Laravel, WordPress, etc.
- **Modern Admin Tools** - phpMyAdmin 5.2.3, Adminer 5.4.2, phpCacheAdmin 2.4.1
- **MCP Server** - AI-powered management via Claude Code
- **Custom PHP Images** - PHP 8.3 & 8.4 with modern tools (Bun, Vite, Pest, etc.)
- **Port Forwarding** - Standard Laravel `.env` works out of the box

## Directory Structure

```
devilbox/
├── .claude/                    # AI assistant configuration
│   ├── skills/                 # Reusable commands/tasks
│   └── README.md              # This file
├── .devilbox/                 # Dashboard files
│   └── www/htdocs/vendor/     # Admin tools (71MB)
├── bin/                       # Command wrappers (artisan, composer, etc.)
├── docker-images/             # Custom PHP image definitions
│   ├── php-8.3-work/         # PHP 8.3 custom image
│   └── php-8.4-work/         # PHP 8.4 custom image
├── mcp-server/                # MCP server for AI integration
└── data/www/                  # Project directories
```

## Coding Guidelines

### General Principles

1. **Backwards Compatible** - All changes must work with stock Devilbox
2. **Opt-in Features** - Custom images and features are optional
3. **No Breaking Changes** - Existing setups continue to work
4. **Documentation First** - Document before implementing
5. **Test Everything** - Verify on macOS, Linux (where possible)

### File Editing Rules

1. **Never edit stock Devilbox files** unless absolutely necessary
2. **Prefer docker-compose.override.yml** over editing docker-compose.yml
3. **Use .env for configuration** - Never hardcode values
4. **Always backup before destructive changes**

### Commit Message Format

```
<type>: <subject>

<body>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Types**: feat, fix, docs, refactor, test, chore

### Code Style

**Bash Scripts:**
- Use `#!/bin/bash` shebang
- Include description header
- Color output for better UX
- Validate inputs
- Provide helpful error messages

**PHP:**
- Follow Devilbox conventions
- Use try-catch for PHP 8.4 exceptions
- Never use deprecated functions
- Comment complex logic

**Docker:**
- Multi-stage builds where appropriate
- Minimize layers
- Clean up apt cache
- Use BuildKit for better caching

## Project Context

### Modernization Phases

**Completed:**
- ✅ Phase 1: Smart Vhost Auto-Detection
- ✅ Phase 2: Admin Tools Modernization
- ✅ Phase 4: MCP Server

**In Progress:**
- ⏳ Phase 3: Setup Wizard (TUI)
- ⏳ Phase 5: Devilbox Boost Package

### Key Technologies

- **Docker** - Containerization
- **Docker Compose** - Service orchestration
- **PHP 8.3/8.4** - Primary language
- **Node.js LTS** - Frontend tooling
- **Bun** - Fast npm alternative
- **MCP** - AI integration protocol

### Important Files

| File | Purpose | Edit? |
|------|---------|-------|
| `.env` | Environment configuration | ❌ Ignored |
| `docker-compose.yml` | Service definitions | ❌ Don't edit |
| `docker-compose.override.yml` | Custom overrides | ✅ User-specific |
| `.devilbox/www/include/lib/Html.php` | Dashboard menu | ✅ Carefully |
| `mcp-server/index.js` | MCP server implementation | ✅ Yes |
| `docker-images/php-*/Dockerfile` | Custom images | ✅ Yes |

## Common Tasks

### Building Custom Images

```bash
# Build PHP 8.4 work image
./docker-images/build-php.sh 8.4

# Build PHP 8.3 work image
./docker-images/build-php.sh 8.3
```

### Testing Admin Tools

```bash
# Visit dashboard
open http://localhost

# Test specific tools
open http://localhost/vendor/adminer-5.4.2-devilbox.php
open http://localhost/vendor/phpcacheadmin-2.4.1/
open http://localhost/vendor/phpmyadmin-5.2.3/
```

### MCP Server

```bash
# Install MCP server
cd mcp-server && ./install.sh

# Test MCP server
node test-mcp.js

# View logs
docker-compose logs -f php
```

### Debugging

```bash
# Check container status
docker-compose ps

# View logs
docker-compose logs php
docker-compose logs httpd
docker-compose logs mysql

# Enter container
docker-compose exec php bash

# Check PHP version
docker-compose exec php php -v

# Check installed extensions
docker-compose exec php php -m
```

## Skills Available

See `.claude/skills/` directory for reusable tasks:

- `build-php-image` - Build custom PHP images
- `test-mcp-server` - Test MCP server functionality
- `update-admin-tools` - Update web admin tools
- `create-vhost` - Create new virtual host
- `backup-devilbox` - Backup entire Devilbox setup

## Dependencies

### Required

- Docker Desktop (latest)
- Node.js LTS (for MCP server)
- Bash 4+ (for scripts)

### Optional

- jq (JSON parsing)
- whiptail/dialog (TUI)
- Claude Code (AI integration)

## Testing Guidelines

### Before Committing

1. ✅ Build custom images successfully
2. ✅ Test with stock Devilbox
3. ✅ Test with custom images
4. ✅ Verify all admin tools load
5. ✅ Check MCP server if modified
6. ✅ Run any test scripts
7. ✅ Update documentation

### Test Checklist

```bash
# Build both PHP versions
./docker-images/build-php.sh 8.3
./docker-images/build-php.sh 8.4

# Test MCP server
cd mcp-server && node test-mcp.js

# Verify admin tools
open http://localhost/vendor/adminer-5.4.2-devilbox.php

# Check dashboard health
open http://localhost

# Test vhost auto-detection
ls -la data/www/*/. devilbox/

# Verify port forwarding
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl -e "SHOW DATABASES;"
```

## Troubleshooting

### Common Issues

**Docker build fails:**
- Check Docker Desktop is running
- Clear build cache: `docker builder prune`
- Check Dockerfile syntax

**Port conflicts:**
- Check ports 80, 443, 3306, 5432, 6379, 11211, 27017
- Modify HTTPD_DOCROOT_DIR in .env if needed

**Permission denied:**
- Check UID/GID in docker-compose.override.yml
- Run `chmod +x` on scripts

**MCP server not detected:**
- Restart Claude Code completely
- Check config: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Verify installation: `cd mcp-server && ./install.sh`

## Resources

- **Main Documentation**: [Devilbox Docs](https://devilbox.readthedocs.io/)
- **Modernization Roadmap**: `ROADMAP-MODERNIZATION.md`
- **Changelog**: `CHANGELOG-MODERN.md`
- **MCP Server**: `mcp-server/README.md`
- **Vhost Auto-Detection**: `VHOST-AUTO-DETECT.md`

## Contributing

When working on Devilbox:

1. **Understand the context** - Read CHANGELOG-MODERN.md and ROADMAP-MODERNIZATION.md
2. **Use skills** - Check `.claude/skills/` for existing tasks
3. **Follow guidelines** - This document
4. **Test thoroughly** - Multiple PHP versions, services
5. **Document changes** - Update relevant docs
6. **Commit properly** - Include co-authorship

## Questions?

- Check existing documentation first
- Review similar implementations
- Ask for clarification before major changes
- Consider backwards compatibility

---

**Last Updated**: February 14, 2026
**Maintained By**: Johan Pretorius with Claude Code assistance
