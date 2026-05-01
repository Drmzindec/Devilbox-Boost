# Custom PHP 8.5 Work Image for Devilbox

This is a custom PHP 8.5 FPM image with essential development tools, created because official Devilbox images for PHP 8.5 are not yet available.

## Included Tools

- **PHP 8.5** with common extensions (pdo, mysqli, mbstring, zip, gd, intl, opcache, soap, etc.)
- **XDebug** for debugging
- **Redis extension** for caching
- **Composer** (latest) for PHP dependency management
- **Node.js LTS** via NodeSource (includes npm)
- **Yarn** package manager
- **Bun** fast JavaScript toolkit
- **Vite** build tool
- **Git** for version control
- **Common utilities**: vim, nano, curl, wget, unzip, rsync
- **Database clients**: MySQL, PostgreSQL
- **Supervisor** for process management

## Building

```bash
./docker-images/build-php.sh 8.5
```
