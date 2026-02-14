# Devilbox Quality of Life Improvements

This guide covers the QoL improvements added to make Devilbox easier to use.

## 1. Auto-Configure Laravel Projects

Instead of manually configuring each Laravel project, use the setup script:

```bash
# Setup Laravel vhost for a project
./setup-laravel-vhost.sh companychat

# Restart httpd to apply
docker-compose restart httpd
```

This automatically creates `.devilbox/apache24.yml` and `.devilbox/nginx.yml` configs that:
- Set DocumentRoot to `public/`
- Configure proper rewrite rules for Laravel routing
- Enable PHP-FPM for `.php` files

**Manual alternative:**
You can also just set `HTTPD_DOCROOT_DIR=public` in your `.env` if ALL your projects are Laravel.

---

## 2. Run Commands Without Entering Container

Add Devilbox bin directory to your PATH:

```bash
# Add to your ~/.zshrc or ~/.bashrc
export PATH="/Users/johanpretorius/devilbox/bin:$PATH"

# Reload shell
source ~/.zshrc  # or source ~/.bashrc
```

Now you can run commands directly:

### PHP Commands
```bash
# From anywhere
php -v
php script.php

# From your Laravel project directory
cd ~/devilbox/data/www/companychat
php artisan migrate
php artisan make:model Post
```

### Composer
```bash
composer install
composer require laravel/sanctum
composer update
```

### Laravel Artisan (context-aware)
```bash
# Navigate to your Laravel project
cd ~/devilbox/data/www/companychat

# Run artisan commands
artisan migrate
artisan make:controller UserController
artisan queue:work
```

The `artisan` wrapper automatically detects your current project!

### Node/NPM/Yarn/Bun
```bash
# NPM
npm install
npm run dev
npm run build

# Yarn
yarn install
yarn dev

# Bun (faster alternative!)
bun install
bun run dev
bun run build
bunx create-next-app

# Node
node script.js
```

### MySQL Client
```bash
# Connects automatically with root credentials from .env
mysql

# Or specify database
mysql fansframe

# Run SQL directly
mysql -e "SHOW DATABASES;"
```

---

## 3. Adding More Tools to Custom PHP Image

Want to add more tools like the official Devilbox images? Edit your Dockerfile:

### Add Laravel Installer
```dockerfile
# After Composer installation
RUN composer global require laravel/installer
ENV PATH="${PATH}:/home/devilbox/.composer/vendor/bin"
```

### Add WP-CLI
```dockerfile
RUN curl -O https://raw.githubusercontent.com/wp-cli/builds/gh-pages/phar/wp-cli.phar \
    && chmod +x wp-cli.phar \
    && mv wp-cli.phar /usr/local/bin/wp
```

### Add Symfony CLI
```dockerfile
RUN curl -sS https://get.symfony.com/cli/installer | bash \
    && mv /root/.symfony*/bin/symfony /usr/local/bin/symfony
```

### Add Python & Pip
```dockerfile
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*
```

Then rebuild:
```bash
./docker-images/build-php.sh 8.4
docker-compose up -d --force-recreate php
```

---

## Quick Reference

| What you want to do | Command |
|---------------------|---------|
| Setup Laravel project | `./setup-laravel-vhost.sh <project>` |
| Run PHP | `php artisan migrate` |
| Install dependencies | `composer install` |
| Run Laravel command | `artisan make:model Post` |
| Build frontend (npm) | `npm run build` |
| Build frontend (bun) | `bun run build` |
| Access MySQL | `mysql` |
| Enter container | `./shell.sh` |

---

## Why Bun?

Bun is a modern JavaScript runtime that's **much faster** than Node.js:

- **Install packages 10-25x faster** than npm
- **Run scripts 2-3x faster** than Node
- **Built-in bundler** (no need for webpack/vite)
- **Drop-in replacement** for npm/yarn/node
- **Native TypeScript support**

### Comparison
```bash
# Old way (npm)
npm install           # 45 seconds
npm run build         # 12 seconds

# New way (Bun)
bun install           # 2 seconds ⚡
bun run build         # 4 seconds ⚡
```

You can use Bun alongside Node - they coexist peacefully!

---

## Tips

1. **The artisan wrapper is smart**: It uses your current directory name to find the project in the container

2. **All wrappers preserve arguments**: `php -v`, `composer require foo`, `npm install --save-dev` all work as expected

3. **Interactive commands work**: `mysql`, `php artisan tinker`, etc. work with proper TTY

4. **For complex workflows**, you might still want `./shell.sh` to stay in the container

5. **Try Bun for faster builds**: Replace `npm` with `bun` in your workflows:
   ```bash
   bun install          # instead of npm install
   bun run dev          # instead of npm run dev
   bunx vite build      # instead of npx vite build
   ```

6. **Create aliases**: Add to your shell config:
   ```bash
   alias art='artisan'
   alias pa='php artisan'
   alias nrd='npm run dev'
   alias brd='bun run dev'
   ```
