# Using Memcached in Devilbox

Memcached is a high-performance distributed memory caching system, perfect for simple key-value caching.

## Quick Start

### Connection Details

```bash
Host: 127.0.0.1 (via port forwarding)
Port: 11211
```

### Test Connection

```bash
# From PHP container
docker-compose exec php telnet 127.0.0.1 11211
# Type: stats
# Quit with: quit

# Or use PHP
docker-compose exec php php -r '$m = new Memcached(); $m->addServer("127.0.0.1", 11211); var_dump($m->getStats());'
```

## Laravel Configuration

**`.env`**:
```env
CACHE_DRIVER=memcached
MEMCACHED_HOST=127.0.0.1
MEMCACHED_PORT=11211
```

**`config/cache.php`**:
```php
'memcached' => [
    'driver' => 'memcached',
    'persistent_id' => env('MEMCACHED_PERSISTENT_ID'),
    'sasl' => [
        env('MEMCACHED_USERNAME'),
        env('MEMCACHED_PASSWORD'),
    ],
    'options' => [
        // Memcached::OPT_CONNECT_TIMEOUT => 2000,
    ],
    'servers' => [
        [
            'host' => env('MEMCACHED_HOST', '127.0.0.1'),
            'port' => env('MEMCACHED_PORT', 11211),
            'weight' => 100,
        ],
    ],
],
```

**Usage:**
```php
<?php
// Store
Cache::put('key', 'value', 600); // 10 minutes

// Retrieve
$value = Cache::get('key');

// Remember (get or store)
$users = Cache::remember('users.all', 3600, function () {
    return DB::table('users')->get();
});
```

## WordPress Configuration

### 1. Using W3 Total Cache Plugin

1. Install W3 Total Cache
2. Performance → General Settings
3. Page Cache → Enable: Memcached
4. Object Cache → Enable: Memcached
5. Database Cache → Enable: Memcached
6. Configure server: 127.0.0.1:11211

### 2. Using Memcached Object Cache Drop-in

**`wp-content/object-cache.php`**:
```php
<?php
global $memcached_servers;
$memcached_servers = [
    ['127.0.0.1', 11211],
];
```

## Direct PHP Usage

### 1. Using Memcached Extension

```php
<?php
// Create instance
$memcached = new Memcached();

// Add server
$memcached->addServer('127.0.0.1', 11211);

// Test connection
$stats = $memcached->getStats();
if (!empty($stats)) {
    echo "Connected to Memcached!\n";
}

// Store value (simple)
$memcached->set('key', 'value', 600); // 600 seconds = 10 minutes

// Store value (complex)
$userData = [
    'id' => 123,
    'name' => 'John Doe',
    'email' => 'john@example.com'
];
$memcached->set('user:123', $userData, 3600);

// Retrieve value
$value = $memcached->get('key');
$user = $memcached->get('user:123');

// Check if key exists
if ($memcached->get('key') !== false) {
    echo "Key exists\n";
}

// Delete key
$memcached->delete('key');

// Flush all keys
$memcached->flush();
```

### 2. Using Memcache Extension (Legacy)

```php
<?php
// Create instance
$memcache = new Memcache();

// Connect
$memcache->connect('127.0.0.1', 11211);

// Same operations
$memcache->set('key', 'value', 0, 600);
$value = $memcache->get('key');
$memcache->delete('key');
```

## Common Use Cases

### 1. Database Query Caching

```php
<?php
function getUsers($memcached) {
    $cacheKey = 'database:users:all';

    // Try cache first
    $users = $memcached->get($cacheKey);

    if ($users === false) {
        // Cache miss - query database
        $pdo = new PDO('mysql:host=127.0.0.1;dbname=mydb', 'root', 'root');
        $stmt = $pdo->query('SELECT * FROM users');
        $users = $stmt->fetchAll(PDO::FETCH_ASSOC);

        // Store in cache for 5 minutes
        $memcached->set($cacheKey, $users, 300);
    }

    return $users;
}
```

### 2. API Response Caching

```php
<?php
$memcached = new Memcached();
$memcached->addServer('127.0.0.1', 11211);

$cacheKey = 'api:weather:london';
$ttl = 1800; // 30 minutes

$weather = $memcached->get($cacheKey);

if ($weather === false) {
    // Fetch from external API
    $weather = file_get_contents('https://api.weather.com/london');
    $memcached->set($cacheKey, $weather, $ttl);
}

header('Content-Type: application/json');
echo $weather;
```

### 3. Session Storage

```php
<?php
// Configure PHP to use Memcached for sessions
ini_set('session.save_handler', 'memcached');
ini_set('session.save_path', '127.0.0.1:11211');

session_start();

$_SESSION['user_id'] = 123;
$_SESSION['username'] = 'johndoe';
```

### 4. Fragment Caching

```php
<?php
$memcached = new Memcached();
$memcached->addServer('127.0.0.1', 11211);

function renderExpensiveWidget($memcached, $widgetId) {
    $cacheKey = "widget:$widgetId";

    $html = $memcached->get($cacheKey);

    if ($html === false) {
        ob_start();
        // Expensive rendering logic
        include "widgets/$widgetId.php";
        $html = ob_get_clean();

        $memcached->set($cacheKey, $html, 600);
    }

    return $html;
}

echo renderExpensiveWidget($memcached, 'sidebar-stats');
```

### 5. Counter/Statistics

```php
<?php
$memcached = new Memcached();
$memcached->addServer('127.0.0.1', 11211);

// Increment page view counter
$memcached->increment('stats:page_views', 1);

// Increment with initial value
if ($memcached->get('stats:visitors') === false) {
    $memcached->set('stats:visitors', 0);
}
$memcached->increment('stats:visitors');

// Get current value
$pageViews = $memcached->get('stats:page_views');
echo "Page views: $pageViews\n";
```

## Advanced Features

### 1. Multi-Get (Batch Retrieval)

```php
<?php
$memcached = new Memcached();
$memcached->addServer('127.0.0.1', 11211);

// Get multiple keys at once
$keys = ['user:1', 'user:2', 'user:3'];
$users = $memcached->getMulti($keys);

foreach ($users as $key => $user) {
    echo "$key: " . print_r($user, true) . "\n";
}
```

### 2. CAS (Check And Set)

```php
<?php
$memcached = new Memcached();
$memcached->addServer('127.0.0.1', 11211);

// Get with CAS token
$cas = null;
$value = $memcached->get('counter', null, $cas);

if ($value !== false) {
    $newValue = $value + 1;

    // Only set if value hasn't changed
    if ($memcached->cas($cas, 'counter', $newValue)) {
        echo "Updated successfully\n";
    } else {
        echo "Value was modified by another process\n";
    }
}
```

### 3. Delayed Operations

```php
<?php
$memcached = new Memcached();
$memcached->addServer('127.0.0.1', 11211);

// Set with specific options
$memcached->setOption(Memcached::OPT_COMPRESSION, true);

// Add (only if key doesn't exist)
$memcached->add('key', 'value', 600);

// Replace (only if key exists)
$memcached->replace('key', 'new-value', 600);

// Append to existing value
$memcached->append('log', "\nNew log entry");

// Prepend to existing value
$memcached->prepend('header', "Important: ");
```

## Performance Best Practices

### 1. Connection Pooling

```php
<?php
// Use persistent connections
$memcached = new Memcached('pool_id');

if (count($memcached->getServerList()) === 0) {
    $memcached->addServer('127.0.0.1', 11211);
    $memcached->setOption(Memcached::OPT_COMPRESSION, true);
    $memcached->setOption(Memcached::OPT_BINARY_PROTOCOL, true);
}
```

### 2. Compression

```php
<?php
$memcached = new Memcached();
$memcached->addServer('127.0.0.1', 11211);

// Enable compression for large values
$memcached->setOption(Memcached::OPT_COMPRESSION, true);

// Store large data
$largeData = file_get_contents('large-file.json');
$memcached->set('large:data', $largeData, 3600);
```

### 3. Key Naming Convention

```php
<?php
// Good key naming
$memcached->set('user:123:profile', $userData, 3600);
$memcached->set('post:456:comments', $comments, 600);
$memcached->set('cache:query:users:active', $users, 300);

// Avoid
$memcached->set('userprofile123', $userData, 3600); // Not descriptive
$memcached->set('x', $value, 600); // Too short
```

## Redis vs Memcached - When to Use Which?

### Use Memcached When:

✅ You need simple key-value caching
✅ You want multi-threaded performance
✅ You need distributed caching across servers
✅ Memory management is important (LRU eviction)
✅ You want dead-simple setup

### Use Redis When:

✅ You need data structures (lists, sets, sorted sets, hashes)
✅ You need persistence
✅ You need pub/sub messaging
✅ You need atomic operations
✅ You need transactions
✅ You need Lua scripting

## Monitoring & Debugging

### Check Memcached Stats

```bash
# Connect via telnet
docker-compose exec php telnet 127.0.0.1 11211

# Common commands:
stats           # General statistics
stats items     # Item statistics
stats slabs     # Slab statistics
stats sizes     # Size statistics
flush_all       # Clear all data
quit            # Exit
```

### Using phpCacheAdmin

```bash
# Open in browser
open http://localhost/vendor/phpcacheadmin-2.4.1/

# Features:
# - View all keys
# - Server statistics
# - Memory usage
# - Hit/miss ratios
# - Flush cache
```

### PHP Script to Monitor

```php
<?php
$memcached = new Memcached();
$memcached->addServer('127.0.0.1', 11211);

$stats = $memcached->getStats();
$serverStats = $stats['127.0.0.1:11211'];

echo "Memcached Statistics:\n";
echo "Uptime: " . $serverStats['uptime'] . " seconds\n";
echo "Current items: " . $serverStats['curr_items'] . "\n";
echo "Total items: " . $serverStats['total_items'] . "\n";
echo "Bytes used: " . $serverStats['bytes'] . "\n";
echo "Get hits: " . $serverStats['get_hits'] . "\n";
echo "Get misses: " . $serverStats['get_misses'] . "\n";
echo "Hit rate: " . round($serverStats['get_hits'] / ($serverStats['get_hits'] + $serverStats['get_misses']) * 100, 2) . "%\n";
```

## Troubleshooting

### Connection Failed

```bash
# Check Memcached is running
docker-compose ps memcd

# Test from PHP container
docker-compose exec php telnet 127.0.0.1 11211

# Check port forwarding
docker-compose exec php netstat -tlnp | grep 11211

# Check logs
docker-compose logs memcd
```

### Keys Not Persisting

Memcached is **memory-only** - no persistence:
- Data is lost on restart
- Use Redis if you need persistence
- This is normal Memcached behavior

### Memory Full

```bash
# Check memory usage
telnet 127.0.0.1 11211
stats

# Increase memory in docker-compose.override.yml
services:
  memcd:
    command: ["-m", "256"] # 256MB
```

### Extension Not Found

```bash
# Check if Memcached extension is installed
docker-compose exec php php -m | grep memcached

# If missing, rebuild custom image
./docker-images/build-php.sh 8.4
```

## Resources

- **Memcached Wiki**: https://github.com/memcached/memcached/wiki
- **PHP Memcached**: https://www.php.net/manual/en/book.memcached.php
- **Laravel Cache**: https://laravel.com/docs/cache
- **Monitor in Devilbox**: http://localhost/vendor/phpcacheadmin-2.4.1/
