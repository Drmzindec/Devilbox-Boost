# Vhost Auto-Detection Feature

## Overview
Devilbox now includes **automatic vhost configuration** that detects your framework type and configures the correct document root automatically. No more manual vhost setup!

## How It Works

A background service runs inside the PHP container that:
1. Scans `/shared/httpd/` every 30 seconds for new projects
2. Detects the framework based on marker files
3. Automatically creates `.devilbox/apache24.yml` and `.devilbox/nginx.yml` configs
4. Sets the correct DocumentRoot based on framework type

## Supported Frameworks

| Framework | Detection Method | DocumentRoot |
|-----------|-----------------|--------------|
| **Laravel** | `artisan` file exists | `/public` |
| **Symfony** | `symfony.lock` or `bin/console` exists | `/public` |
| **CakePHP** | `bin/cake` exists | `/public` |
| **Yii** | `yii` file exists | `/public` |
| **WordPress** | `wp-config.php` exists | `/` (project root) |
| **CodeIgniter** | `system/core/CodeIgniter.php` exists | `/` (project root) |
| **Generic PHP** | No framework detected | `/` (project root) |

## What Gets Detected

From the current setup, the service successfully detected and configured:

### Laravel Projects (4)
- companychat → `/public`
- fansframe → `/public`
- moducraft-v2 → `/public`
- tapthetable → `/public`

### WordPress Projects (3)
- eatoutthebox → project root
- moducraft → project root
- testbench → project root

### Generic PHP Projects (3)
- avatar → project root
- company-chat-app → project root
- note-app → project root

## Framework-Specific Features

### Laravel/Symfony
- Nginx config includes `try_files $uri $uri/ /index.php?$query_string;` for routing
- Proper handling of front-controller pattern

### WordPress
- Nginx config includes `try_files $uri $uri/ /index.php?$args;` for permalink support
- Optimized for WordPress-style URLs

### Generic PHP
- Basic PHP-FPM configuration
- Standard index file handling

## Logs

View the auto-detection service logs:
```bash
docker logs devilbox-php-1 2>&1 | grep "vhost-auto-config"
```

Example output:
```
[vhost-auto-config] Starting vhost auto-configuration service...
[vhost-auto-config] Checking every 30 seconds for new projects
[vhost-auto-config] ✅ Configured companychat as laravel project
[vhost-auto-config] ✅ Configured eatoutthbox as wordpress project
```

## Manual Override

If you want to customize a vhost config:
1. The auto-detection script **only creates configs if they don't exist**
2. Once a `.devilbox/apache24.yml` or `.devilbox/nginx.yml` exists, it won't be overwritten
3. You can manually create/edit these files and the service will respect your changes

## How to Apply to New Projects

1. Create a new project in `/shared/httpd/` (or `data/www/` on host)
2. Wait up to 30 seconds (or restart the PHP container)
3. The service will automatically detect it and create vhost configs
4. Restart httpd to load the new vhost: `docker-compose restart httpd`

Example:
```bash
# Create new Laravel project
cd data/www
composer create-project laravel/laravel mynewapp

# Wait 30 seconds, or restart PHP container to trigger immediate scan
docker-compose restart php

# Restart httpd to load new vhost
docker-compose restart httpd

# Access at: http://mynewapp.local
```

## Technical Details

**Service Script**: `/usr/local/bin/vhost-auto-configure.sh`
**Started By**: `/usr/local/bin/docker-entrypoint.sh` (runs in background)
**Check Interval**: 30 seconds
**Projects Directory**: `/shared/httpd/` inside container

## Benefits

✅ **Laravel works out-of-the-box** - No more manual DocumentRoot configuration
✅ **WordPress works out-of-the-box** - Proper permalink support
✅ **Symfony, CakePHP, Yii supported** - Modern PHP frameworks just work
✅ **Zero maintenance** - Set it and forget it
✅ **Safe** - Never overwrites existing custom configs
✅ **Fast** - Checks every 30 seconds for new projects

## Troubleshooting

### Vhost not auto-created?

Check if the service is running:
```bash
docker exec devilbox-php-1 ps aux | grep vhost-auto-configure
```

### Need immediate detection?

Restart the PHP container:
```bash
docker-compose restart php
```

### Want to see what framework was detected?

Check the logs:
```bash
docker logs devilbox-php-1 2>&1 | grep "Configured"
```

## Comparison: Before vs After

### Before (Manual Setup)
1. Create Laravel project
2. Manually create `.devilbox/apache24.yml`
3. Manually set `DocumentRoot` to `/public`
4. Manually create `.devilbox/nginx.yml`
5. Configure routing rules
6. Restart httpd
7. Hope you got it right

### After (Auto-Detection)
1. Create Laravel project
2. **That's it!** Everything else is automatic

---

**This feature solves the critical issue:** "laravel doesnt properly work out the box since you cant easily change the .conf files"

Now Laravel, WordPress, Symfony, and other frameworks **just work** without any manual vhost configuration!
