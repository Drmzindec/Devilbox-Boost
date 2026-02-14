# Modern Admin Tools - Devilbox

## Overview
All Devilbox web-based administration tools have been updated to their latest versions with full PHP 8.4 compatibility.

## Updated Tools

### Database Management

#### phpMyAdmin 5.2.3
- **Purpose**: MySQL/MariaDB administration
- **Location**: `/vendor/phpmyadmin-5.2.3/`
- **URL**: http://localhost/vendor/phpmyadmin-5.2.3/
- **Previous Version**: 5.1.3
- **Released**: October 2025
- **Features**:
  - Full PHP 8.4 support
  - Modern UI with dark mode
  - Import/export improvements
  - Enhanced security features

#### phpPgAdmin 7.13.0
- **Purpose**: PostgreSQL administration
- **Location**: `/vendor/phppgadmin-7.13.0/`
- **URL**: http://localhost/vendor/phppgadmin-7.13.0/
- **Status**: Already current (no update needed)
- **Features**:
  - PHP 8.2+ compatibility
  - PostgreSQL 15+ support

#### Adminer 5.4.2
- **Purpose**: Multi-database management (MySQL, PostgreSQL, SQLite, MongoDB, etc.)
- **Location**: `/vendor/adminer-5.4.2-devilbox.php`
- **URL**: http://localhost/vendor/adminer-5.4.2-devilbox.php
- **Previous Version**: 4.8.1
- **Released**: December 2025
- **Features**:
  - Single-file database management
  - Support for MySQL, PostgreSQL, SQLite, MongoDB, and more
  - Lightweight and fast
  - Full PHP 8.4 compatibility
  - **Devilbox Enhancement**: Server field prefilled with `127.0.0.1` (uses port forwarding)

---

### Cache Management

#### phpCacheAdmin 2.4.1 (NEW!)
- **Purpose**: Unified Redis, Memcached, APCu, OPcache, and Realpath cache management
- **Location**: `/vendor/phpcacheadmin-2.4.1/`
- **URL**: http://localhost/vendor/phpcacheadmin-2.4.1/
- **Released**: February 2024
- **Replaces**: phpMemcachedAdmin (2014) + phpRedMin (2017)

**Key Features:**
- ✅ **Unified Interface**: One tool for all cache systems
- ✅ **Dark Mode**: Modern, eye-friendly interface
- ✅ **No Extensions Required**: Works without memcached/redis PHP extensions
- ✅ **Responsive Design**: Mobile-friendly admin panel
- ✅ **Real-time Metrics**: Live cache statistics and monitoring

**Supported Cache Systems:**
- **Redis**: Full CRUD operations, cluster support, slowlog analysis
- **Memcached**: Key management, stats, slabs info
- **OPcache**: Script invalidation, memory usage, hit rates
- **APCu**: User cache management, statistics
- **Realpath Cache**: View and clear PHP's realpath entries

**Configuration:**
```php
// /vendor/phpcacheadmin-2.4.1/config.php
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

---

### OpCache Management

#### OpCache GUI 3.6.0
- **Purpose**: OPcache monitoring and management
- **Location**: `/vendor/opcache-gui-3.6.0.php`
- **URL**: http://localhost/vendor/opcache-gui-3.6.0.php
- **Previous**: ocp.php (old viewer)
- **Released**: July 2025
- **Features**:
  - PHP 8.4 + JIT support
  - Real-time opcache statistics
  - Memory usage visualization
  - Script invalidation
  - Single-file deployment

---

## Devilbox Menu Integration

All tools are accessible from the Devilbox dashboard under **Tools** menu:

```
Tools
├── Adminer 5.4.2
├── phpMyAdmin 5.2.3
├── phpPgAdmin 7.13.0
├── phpCacheAdmin 2.4.1
└── OpCache GUI 3.6.0
```

The menu automatically selects the correct version based on your PHP version.

---

## Version Selection Logic

Defined in `.devilbox/www/include/lib/Html.php`:

### phpMyAdmin
- PHP < 5.5 → phpMyAdmin 4.0
- PHP < 7.1 → phpMyAdmin 4.9.7
- PHP >= 7.1 → **phpMyAdmin 5.2.3**

### Adminer
- PHP < 5.4 → Adminer 4.6.3
- PHP < 8.0 → Adminer 4.8.1
- PHP >= 8.0 → **Adminer 5.4.2**

### phpPgAdmin
- PHP < 7.1 → phpPgAdmin 5.6.0
- PHP < 7.2 → phpPgAdmin 7.12.1
- PHP >= 7.2 → **phpPgAdmin 7.13.0**

---

## Installation & Configuration

### phpMyAdmin
Config copied from previous version:
```bash
cp /vendor/phpmyadmin-5.1.3/config.inc.php /vendor/phpmyadmin-5.2.3/config.inc.php
```

### phpCacheAdmin
Custom configuration created at `/vendor/phpcacheadmin-2.4.1/config.php` with Devilbox service connections.

### OpCache GUI
No configuration needed - single-file deployment.

### Adminer
Custom wrapper created at `/vendor/adminer-5.4.2-devilbox.php` that:
- Prefills the server field with `127.0.0.1` (port forwarding)
- Uses JavaScript to set the value on page load
- Ensures users connect via localhost proxy instead of container IPs

**Credentials** (pre-filled server field):
- **MySQL**: Server=`127.0.0.1`, User=`root`, Pass=`root`
- **PostgreSQL**: Server=`127.0.0.1`, User=`postgres`, Pass=`postgres`

---

## Removed Tools

The following outdated tools have been replaced:

### phpMemcachedAdmin 1.3.0
- **Last Updated**: 2014 (12 years old)
- **Status**: Abandoned
- **Replaced By**: phpCacheAdmin 2.4.1
- **Reason**: Severely outdated, no PHP 8+ support

### phpRedMin
- **Last Commit**: June 27, 2017 (9 years old)
- **Status**: Abandoned
- **Replaced By**: phpCacheAdmin 2.4.1
- **Reason**: No longer maintained, PHP 8 incompatible

### ocp.php (Old OpCache Panel)
- **Status**: Outdated
- **Replaced By**: OpCache GUI 3.6.0
- **Reason**: No JIT support, outdated UI

---

## Testing Results

All tools tested with **PHP 8.4.18** on **macOS** with **Docker Desktop**:

| Tool | Status | Notes |
|------|--------|-------|
| phpMyAdmin 5.2.3 | ✅ Working | No warnings, full functionality |
| Adminer 5.4.2 | ✅ Working | Multi-DB support confirmed |
| phpPgAdmin 7.13.0 | ✅ Working | PostgreSQL 15 compatible |
| phpCacheAdmin 2.4.1 | ✅ Working | All dashboards functional |
| OpCache GUI 3.6.0 | ✅ Working | JIT stats displayed correctly |

---

## Troubleshooting

### phpCacheAdmin Shows Blank Page

**Issue**: Permission errors when creating cache directories

**Solution**: Config uses `/tmp` directory instead:
```php
'metricsdir' => '/tmp/phpcacheadmin/metrics',
'twigcache' => '/tmp/phpcacheadmin/twig',
```

This avoids Docker volume permission issues.

### phpMyAdmin Config Missing

**Issue**: phpMyAdmin shows setup wizard

**Solution**: Copy config from previous version:
```bash
docker exec devilbox-php-1 cp /var/www/default/htdocs/vendor/phpmyadmin-5.1.3/config.inc.php \
                                /var/www/default/htdocs/vendor/phpmyadmin-5.2.3/config.inc.php
```

### Tools Not Showing in Menu

**Issue**: Old tools still appear in Devilbox menu

**Solution**: Restart httpd container:
```bash
docker-compose restart httpd
```

---

## Benefits Over Old Tools

### phpCacheAdmin vs phpMemcachedAdmin + phpRedMin

| Feature | Old Tools | phpCacheAdmin |
|---------|-----------|---------------|
| **Last Updated** | 2014 / 2017 | 2024 (actively maintained) |
| **PHP 8.4 Support** | ❌ No | ✅ Yes |
| **Redis Support** | phpRedMin only | ✅ Full support + cluster |
| **Memcached Support** | phpMemcachedAdmin only | ✅ No extension required |
| **OPcache Support** | ❌ No | ✅ Yes |
| **APCu Support** | ❌ No | ✅ Yes |
| **Dark Mode** | ❌ No | ✅ Yes |
| **Responsive UI** | ❌ No | ✅ Yes |
| **Tools Required** | 2 separate tools | 1 unified tool |

### OpCache GUI 3.6.0 vs ocp.php

| Feature | ocp.php | OpCache GUI 3.6.0 |
|---------|---------|-------------------|
| **PHP 8.4 Support** | ⚠️ Limited | ✅ Full |
| **JIT Statistics** | ❌ No | ✅ Yes |
| **Modern UI** | ❌ No | ✅ Yes |
| **Active Maintenance** | ❌ No | ✅ Yes (2025) |

---

## Version History

| Tool | Old Version | New Version | Release Date | Improvement |
|------|-------------|-------------|--------------|-------------|
| phpMyAdmin | 5.1.3 | 5.2.3 | Oct 2025 | +2 minor versions |
| Adminer | 4.8.1 | 5.4.2 | Dec 2025 | +1 major + 4 minor |
| phpMemcachedAdmin | 1.3.0 (2014) | - | - | Replaced by phpCacheAdmin |
| phpRedMin | 1.0 (2017) | - | - | Replaced by phpCacheAdmin |
| phpCacheAdmin | - | 2.4.1 | Feb 2024 | New addition |
| OpCache Viewer | ocp.php | opcache-gui 3.6.0 | Jul 2025 | Complete replacement |
| phpPgAdmin | 7.13.0 | 7.13.0 | - | Already current |

---

## Future Updates

Admin tools should be checked for updates periodically:

1. **phpMyAdmin**: https://www.phpmyadmin.net/downloads/
2. **Adminer**: https://github.com/vrana/adminer/releases
3. **phpCacheAdmin**: https://github.com/RobiNN1/phpCacheAdmin/releases
4. **OpCache GUI**: https://github.com/amnuts/opcache-gui
5. **phpPgAdmin**: https://github.com/phppgadmin/phppgadmin/releases

---

## Credits

- **phpMyAdmin**: https://www.phpmyadmin.net/
- **Adminer**: https://www.adminer.org/
- **phpPgAdmin**: https://github.com/phppgadmin/phppgadmin
- **phpCacheAdmin**: https://github.com/RobiNN1/phpCacheAdmin
- **OpCache GUI**: https://github.com/amnuts/opcache-gui

---

**Last Updated**: February 14, 2026
**Devilbox Modern Version**: v3.0.0-beta-0.4+
**PHP Version Tested**: 8.4.18
