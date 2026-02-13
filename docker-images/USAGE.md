# Using Custom PHP Images with Devilbox

Since official Devilbox doesn't yet support PHP 8.3+, this directory contains custom Dockerfiles that extend the official PHP images with Devilbox-compatible tooling.

## Quick Start

### 1. Build the PHP 8.3 Image

```bash
cd /Users/johanpretorius/devilbox
./docker-images/build-php.sh 8.3
```

This will build a custom image tagged as `devilbox-php-8.3:work` with:
- PHP 8.3 FPM
- Common PHP extensions (XDebug, Redis, PDO, GD, etc.)
- Composer (latest)
- Node.js LTS via NVM
- Yarn, npm, webpack
- Git, vim, and other utilities
- User mapping matching your host UID/GID

### 2. Configure Devilbox to Use Custom Image

**Option A: Modify docker-compose.yml directly**

Edit `/Users/johanpretorius/devilbox/docker-compose.yml`:

```yaml
php:
  # Comment out the default image:
  # image: devilbox/php-fpm:${PHP_SERVER}-work-0.151

  # Add custom image:
  image: devilbox-php-${PHP_SERVER}:work
```

**Option B: Use docker-compose.override.yml**

Edit `/Users/johanpretorius/devilbox/docker-compose.override.yml`:

```yaml
version: '2.3'

services:
  php:
    image: devilbox-php-${PHP_SERVER}:work
```

### 3. Set PHP Version in .env

```bash
# Edit .env
PHP_SERVER=8.3
```

### 4. Start Devilbox

```bash
# Stop and remove existing containers
docker-compose stop
docker-compose rm -f

# Start with new PHP version
docker-compose up httpd php mysql
```

### 5. Verify

Access http://localhost in your browser to see the Devilbox dashboard.
Check the PHP version in the dashboard or run:

```bash
./shell.sh
php -v
```

## Building PHP 8.4

When you're ready to test PHP 8.4:

```bash
# Copy the 8.3 Dockerfile as a template
cp -r docker-images/php-8.3-work docker-images/php-8.4-work

# Edit the Dockerfile
sed -i '' 's/8.3/8.4/g' docker-images/php-8.4-work/Dockerfile

# Build
./docker-images/build-php.sh 8.4

# Update .env
PHP_SERVER=8.4

# Restart
docker-compose stop && docker-compose rm -f && docker-compose up httpd php mysql
```

## Customizing the Image

### Add More PHP Extensions

Edit `docker-images/php-8.3-work/Dockerfile` and add to the RUN docker-php-ext-install section:

```dockerfile
RUN docker-php-ext-install -j$(nproc) \
    pdo \
    pdo_mysql \
    # ... existing extensions ...
    imagick \
    sodium
```

Then rebuild: `./docker-images/build-php.sh 8.3`

### Add More Node Packages

```dockerfile
RUN npm install -g \
    yarn \
    # ... existing packages ...
    typescript \
    @vue/cli
```

### Add More System Tools

```dockerfile
RUN apt-get update && apt-get install -y \
    git \
    vim \
    # ... existing tools ...
    htop \
    tmux \
    && rm -rf /var/lib/apt/lists/*
```

## Troubleshooting

### Permission Issues

If you encounter permission issues, rebuild with your correct UID/GID:

```bash
docker build \
  --build-arg NEW_UID=$(id -u) \
  --build-arg NEW_GID=$(id -g) \
  -t devilbox-php-8.3:work \
  ./docker-images/php-8.3-work/
```

### Image Not Found

Make sure the image is built:

```bash
docker images | grep devilbox-php
```

Should show:
```
devilbox-php-8.3  work  <image-id>  <time>  <size>
```

### Container Won't Start

Check logs:

```bash
docker-compose logs php
```

### Need to Start Fresh

```bash
# Remove the image
docker rmi devilbox-php-8.3:work

# Rebuild
./docker-images/build-php.sh 8.3
```

## Differences from Official Devilbox Images

This custom image is **lighter** than the official Devilbox work images. It includes:

✅ Essential PHP extensions
✅ Composer & Node.js
✅ Basic development tools

❌ Does NOT include (by default):
- Full test framework suite (codeception, phpspec, etc.)
- All framework CLIs (laravel, symfony, wp-cli, etc.)
- Advanced tools (deployer, taskfile, etc.)
- Multiple Node versions

You can add any of these by customizing the Dockerfile as shown above.
