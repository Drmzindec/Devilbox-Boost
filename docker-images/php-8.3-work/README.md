# Custom PHP 8.3 Work Image for Devilbox

This is a custom PHP 8.3 FPM image with essential development tools, created because official Devilbox images for PHP 8.3 are not yet available.

## Included Tools

- **PHP 8.3** with common extensions (pdo, mysqli, mbstring, zip, gd, intl, opcache, soap, etc.)
- **XDebug** for debugging
- **Redis extension** for caching
- **Composer** (latest) for PHP dependency management
- **Node.js LTS** via NVM (includes npm)
- **Yarn** package manager
- **Git** for version control
- **Common utilities**: vim, nano, curl, wget, unzip, rsync
- **Database clients**: MySQL, PostgreSQL
- **Supervisor** for process management

## Building

```bash
cd /Users/johanpretorius/devilbox
docker build \
  --build-arg NEW_UID=$(id -u) \
  --build-arg NEW_GID=$(id -g) \
  -t devilbox-php-8.3:work \
  ./docker-images/php-8.3-work/
```

## Using with Devilbox

The `docker-compose.override.yml` file is configured to use this image when `PHP_SERVER=8.3` is set in `.env`.

## Adding More Tools

To add additional tools, edit the Dockerfile and rebuild:

```bash
# Edit Dockerfile
vim docker-images/php-8.3-work/Dockerfile

# Rebuild
docker build \
  --build-arg NEW_UID=$(id -u) \
  --build-arg NEW_GID=$(id -g) \
  -t devilbox-php-8.3:work \
  ./docker-images/php-8.3-work/
```
