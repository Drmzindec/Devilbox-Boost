# Build Custom PHP Image

Build a custom PHP work image with modern development tools.

## Usage

```bash
./docker-images/build-php.sh <version>
```

**Versions**: `8.3` or `8.4`

## What It Does

1. Builds custom PHP image from `docker-images/php-<version>-work/Dockerfile`
2. Includes all development tools:
   - Composer, Laravel Installer, WP-CLI, Pest
   - Node.js LTS, npm, yarn, Bun
   - Modern build tools (Vite, Webpack, Prettier, ESLint)
   - All PHP extensions (pgsql, redis, memcached, mongodb, xdebug)
3. Configures vhost auto-detection service
4. Sets up port forwarding capability
5. Tags as `devilbox-php-<version>:work`

## Steps

```bash
# Check current directory
cd /path/to/devilbox

# Build PHP 8.4
./docker-images/build-php.sh 8.4

# Build PHP 8.3
./docker-images/build-php.sh 8.3

# Verify images
docker images | grep devilbox-php
```

## Expected Output

```
Successfully built devilbox-php-8.4:work
Image size: ~2.75GB
```

## Troubleshooting

**Build fails:**
- Ensure Docker Desktop is running
- Clear build cache: `docker builder prune`
- Check internet connection (downloads packages)

**Out of disk space:**
- Clean Docker: `docker system prune -a`
- Check available space: `df -h`

## After Building

Update `docker-compose.override.yml`:

```yaml
services:
  php:
    image: devilbox-php-8.4:work
    environment:
      - NEW_UID=501
      - NEW_GID=20
      - FORWARD_PORTS_TO_LOCALHOST=3306:mysql:3306,5432:pgsql:5432,6379:redis:6379
```

Then restart:

```bash
docker-compose up -d --force-recreate php
docker-compose restart httpd
```
