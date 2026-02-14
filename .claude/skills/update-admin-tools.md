# Update Admin Tools

Update Devilbox web-based admin tools to latest PHP 8.4 compatible versions.

## Current Versions

| Tool | Current | Location |
|------|---------|----------|
| phpMyAdmin | 5.2.3 | `/vendor/phpmyadmin-5.2.3/` |
| Adminer | 5.4.2 | `/vendor/adminer-5.4.2-en.php` |
| phpCacheAdmin | 2.4.1 | `/vendor/phpcacheadmin-2.4.1/` |
| OpCache GUI | 3.6.0 | `/vendor/opcache-gui-3.6.0.php` |
| phpPgAdmin | 7.13.0 | `/vendor/phppgadmin-7.13.0/` |

## Adding New Tool

### 1. Download

```bash
cd .devilbox/www/htdocs/vendor

# Example: Download new phpMyAdmin
wget https://files.phpmyadmin.net/phpMyAdmin/5.2.3/phpMyAdmin-5.2.3-all-languages.tar.gz

# Extract
tar -xzf phpMyAdmin-5.2.3-all-languages.tar.gz
mv phpMyAdmin-5.2.3-all-languages phpmyadmin-5.2.3
```

### 2. Configure

```bash
# Copy config from previous version
cp phpmyadmin-5.1.3/config.inc.php phpmyadmin-5.2.3/config.inc.php

# Or create new config
cp phpmyadmin-5.2.3/config.sample.inc.php phpmyadmin-5.2.3/config.inc.php
```

### 3. Update Menu

Edit `.devilbox/www/include/lib/Html.php`:

```php
if ($el['path'] == '__PHPMYADMIN__') {
    if (version_compare(loadClass('Php')->getVersion(), '5.5', '<')) {
        $el['path'] = '/vendor/phpmyadmin-4.0/index.php';
    } elseif (version_compare(loadClass('Php')->getVersion(), '7.1', '<')) {
        $el['path'] = '/vendor/phpmyadmin-4.9.7/index.php';
    } else {
        $el['path'] = '/vendor/phpmyadmin-5.2.3/index.php';  // ← Updated
    }
}
```

### 4. Test

```bash
# Restart services
docker-compose restart httpd php

# Open tool
open http://localhost/vendor/phpmyadmin-5.2.3/
```

### 5. Commit

```bash
# Add to git (but ignore archives)
git add .devilbox/www/htdocs/vendor/phpmyadmin-5.2.3/
git add .devilbox/www/include/lib/Html.php

# Update gitignore if needed
echo "/.devilbox/www/htdocs/vendor/*.tar.gz" >> .gitignore

# Commit
git commit -m "Update phpMyAdmin to 5.2.3

- Full PHP 8.4 compatibility
- Modern UI improvements
- Security updates

🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
```

## Special Cases

### Adminer Custom Wrapper

For auto-login functionality:

```php
<?php
// vendor/adminer-5.4.2-devilbox.php
if (!isset($_POST['auth'])) {
    $_GET['server'] = $_GET['server'] ?? '127.0.0.1';
    $_GET['username'] = $_GET['username'] ?? 'root';
}
include './adminer-5.4.2-en.php';
```

### phpCacheAdmin Config

```php
<?php
// vendor/phpcacheadmin-2.4.1/config.php
return [
    'redis' => [
        ['name' => 'Devilbox Redis', 'host' => '127.0.0.1', 'port' => 6379],
    ],
    'memcached' => [
        ['name' => 'Devilbox Memcached', 'host' => '127.0.0.1', 'port' => 11211],
    ],
    'metricsdir' => '/tmp/phpcacheadmin/metrics',
    'twigcache' => '/tmp/phpcacheadmin/twig',
];
```

## Verification Checklist

- [ ] Tool loads without errors
- [ ] Connects to services (MySQL, Redis, etc.)
- [ ] No PHP warnings/deprecations
- [ ] Works on PHP 8.3 and 8.4
- [ ] Menu link points to correct path
- [ ] Archives added to .gitignore
- [ ] Documentation updated
- [ ] Changes committed
