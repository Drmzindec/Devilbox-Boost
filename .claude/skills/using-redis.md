# Using Redis in Devilbox

Redis is a powerful in-memory data store used for caching, sessions, queues, and real-time applications.

## Quick Start

### Connection Details

```bash
Host: 127.0.0.1 (via port forwarding)
Port: 6379
Password: (none)
```

### Test Connection

```bash
# From PHP container
docker-compose exec php redis-cli -h 127.0.0.1 ping
# Response: PONG

# From host (if redis-cli installed)
redis-cli -h 127.0.0.1 -p 6379 ping
```

## Laravel Configuration

### 1. Cache Configuration

**`.env`**:
```env
CACHE_DRIVER=redis
REDIS_HOST=127.0.0.1
REDIS_PASSWORD=null
REDIS_PORT=6379
REDIS_CLIENT=phpredis
```

**`config/database.php`**:
```php
'redis' => [
    'client' => env('REDIS_CLIENT', 'phpredis'),

    'default' => [
        'host' => env('REDIS_HOST', '127.0.0.1'),
        'password' => env('REDIS_PASSWORD', null),
        'port' => env('REDIS_PORT', 6379),
        'database' => 0,
    ],

    'cache' => [
        'host' => env('REDIS_HOST', '127.0.0.1'),
        'password' => env('REDIS_PASSWORD', null),
        'port' => env('REDIS_PORT', 6379),
        'database' => 1,
    ],

    'session' => [
        'host' => env('REDIS_HOST', '127.0.0.1'),
        'password' => env('REDIS_PASSWORD', null),
        'port' => env('REDIS_PORT', 6379),
        'database' => 2,
    ],
],
```

### 2. Session Storage

**`.env`**:
```env
SESSION_DRIVER=redis
```

### 3. Queue Configuration

**`.env`**:
```env
QUEUE_CONNECTION=redis
```

**Test queue:**
```php
// routes/web.php
Route::get('/test-queue', function () {
    dispatch(new \App\Jobs\TestJob());
    return 'Job dispatched!';
});
```

```bash
# Run queue worker
php artisan queue:work redis
```

## WordPress Configuration

### 1. Object Cache

Install Redis Object Cache plugin or use Predis:

**`wp-config.php`**:
```php
define('WP_REDIS_HOST', '127.0.0.1');
define('WP_REDIS_PORT', 6379);
define('WP_REDIS_DATABASE', 0);
define('WP_REDIS_CLIENT', 'phpredis'); // or 'predis'

// Optional: prefix for multi-site
define('WP_REDIS_PREFIX', 'my-site');
```

### 2. Session Handler

```php
// wp-config.php (before wp-settings.php)
ini_set('session.save_handler', 'redis');
ini_set('session.save_path', 'tcp://127.0.0.1:6379?database=2');
```

## Direct PHP Usage

### 1. Using phpredis Extension

```php
<?php
// Connect to Redis
$redis = new Redis();
$redis->connect('127.0.0.1', 6379);

// Test connection
if ($redis->ping()) {
    echo "Connected to Redis!\n";
}

// Set key
$redis->set('user:1:name', 'John Doe');

// Get key
$name = $redis->get('user:1:name');
echo "Name: $name\n";

// Set with expiration (60 seconds)
$redis->setex('session:abc123', 60, json_encode(['user_id' => 1]));

// Hash operations
$redis->hSet('user:1', 'name', 'John Doe');
$redis->hSet('user:1', 'email', 'john@example.com');
$userData = $redis->hGetAll('user:1');

// List operations (queue)
$redis->rPush('queue:emails', json_encode(['to' => 'user@example.com']));
$job = $redis->lPop('queue:emails');

// Close connection
$redis->close();
```

### 2. Using Predis Library

```bash
# Install via Composer
composer require predis/predis
```

```php
<?php
require 'vendor/autoload.php';

$client = new Predis\Client([
    'scheme' => 'tcp',
    'host'   => '127.0.0.1',
    'port'   => 6379,
]);

// Same operations as phpredis
$client->set('key', 'value');
$value = $client->get('key');
```

## Common Use Cases

### 1. Page Caching

```php
<?php
$redis = new Redis();
$redis->connect('127.0.0.1', 6379);

$cacheKey = 'page:home';
$ttl = 3600; // 1 hour

// Try to get from cache
if ($redis->exists($cacheKey)) {
    $html = $redis->get($cacheKey);
    echo $html;
    exit;
}

// Generate page
ob_start();
include 'expensive-page.php';
$html = ob_get_clean();

// Store in cache
$redis->setex($cacheKey, $ttl, $html);

echo $html;
```

### 2. Rate Limiting

```php
<?php
function checkRateLimit($redis, $userId, $maxRequests = 100, $window = 60) {
    $key = "rate_limit:user:$userId";

    $current = $redis->incr($key);

    if ($current === 1) {
        $redis->expire($key, $window);
    }

    return $current <= $maxRequests;
}

$redis = new Redis();
$redis->connect('127.0.0.1', 6379);

if (checkRateLimit($redis, 123)) {
    // Process request
    echo "Request allowed";
} else {
    // Rate limited
    http_response_code(429);
    echo "Too many requests";
}
```

### 3. Session Storage

```php
<?php
ini_set('session.save_handler', 'redis');
ini_set('session.save_path', 'tcp://127.0.0.1:6379?database=2');

session_start();

$_SESSION['user_id'] = 123;
$_SESSION['username'] = 'johndoe';
```

### 4. Real-time Leaderboard

```php
<?php
$redis = new Redis();
$redis->connect('127.0.0.1', 6379);

// Add scores
$redis->zAdd('leaderboard', 1000, 'player1');
$redis->zAdd('leaderboard', 1500, 'player2');
$redis->zAdd('leaderboard', 800, 'player3');

// Get top 10 (highest to lowest)
$topPlayers = $redis->zRevRange('leaderboard', 0, 9, true);

foreach ($topPlayers as $player => $score) {
    echo "$player: $score\n";
}

// Get player rank
$rank = $redis->zRevRank('leaderboard', 'player2');
echo "player2 rank: " . ($rank + 1) . "\n";
```

### 5. Pub/Sub Messaging

**Publisher:**
```php
<?php
$redis = new Redis();
$redis->connect('127.0.0.1', 6379);

$message = json_encode([
    'event' => 'user.registered',
    'user_id' => 123,
    'timestamp' => time()
]);

$redis->publish('events', $message);
```

**Subscriber:**
```php
<?php
$redis = new Redis();
$redis->connect('127.0.0.1', 6379);

$redis->subscribe(['events'], function ($redis, $channel, $message) {
    echo "Received on $channel: $message\n";

    $data = json_decode($message, true);
    // Process event
});
```

## Performance Best Practices

### 1. Use Pipelining for Bulk Operations

```php
<?php
$redis = new Redis();
$redis->connect('127.0.0.1', 6379);

// Instead of this (slow - multiple round trips)
for ($i = 0; $i < 1000; $i++) {
    $redis->set("key:$i", "value:$i");
}

// Use pipeline (fast - single round trip)
$redis->multi(Redis::PIPELINE);
for ($i = 0; $i < 1000; $i++) {
    $redis->set("key:$i", "value:$i");
}
$redis->exec();
```

### 2. Set Appropriate TTLs

```php
<?php
// Short-lived data (API responses)
$redis->setex('api:weather', 300, $weatherData); // 5 minutes

// Medium-lived data (user sessions)
$redis->setex('session:xyz', 3600, $sessionData); // 1 hour

// Long-lived data (user profiles)
$redis->setex('user:123', 86400, $userData); // 24 hours
```

### 3. Use Appropriate Data Structures

```php
<?php
// ❌ Bad: Storing JSON for simple key-value
$redis->set('user:1', json_encode([
    'name' => 'John',
    'email' => 'john@example.com',
    'age' => 30
]));

// ✅ Good: Use hash for structured data
$redis->hMSet('user:1', [
    'name' => 'John',
    'email' => 'john@example.com',
    'age' => 30
]);

// Get single field
$email = $redis->hGet('user:1', 'email');

// Get all fields
$user = $redis->hGetAll('user:1');
```

## Monitoring & Debugging

### Check Redis Stats

```bash
# Connect via CLI
docker-compose exec php redis-cli -h 127.0.0.1

# Get server info
INFO

# Get memory usage
INFO memory

# Get statistics
INFO stats

# Monitor commands in real-time
MONITOR

# Check slow queries
SLOWLOG GET 10
```

### Using phpCacheAdmin

```bash
# Open in browser
open http://localhost/vendor/phpcacheadmin-2.4.1/

# Features:
# - View all keys
# - Search keys
# - View/edit/delete values
# - Server statistics
# - Memory usage graphs
```

### Common Redis Commands

```bash
# List all keys (use with caution in production!)
KEYS *

# List keys matching pattern
KEYS user:*

# Get key type
TYPE user:1

# Get TTL (time to live)
TTL session:abc123

# Check if key exists
EXISTS user:1

# Delete key
DEL user:1

# Flush database (delete all keys)
FLUSHDB

# Flush all databases
FLUSHALL
```

## Troubleshooting

### Connection Failed

```bash
# Check Redis is running
docker-compose ps redis

# Test from PHP container
docker-compose exec php redis-cli -h 127.0.0.1 ping

# Check port forwarding
docker-compose exec php netstat -tlnp | grep 6379

# Check Redis logs
docker-compose logs redis
```

### Memory Issues

```bash
# Check memory usage
docker-compose exec php redis-cli -h 127.0.0.1 INFO memory

# Set max memory in docker-compose.yml
services:
  redis:
    command: ["redis-server", "--maxmemory", "256mb", "--maxmemory-policy", "allkeys-lru"]
```

### Slow Performance

```bash
# Enable slow log
CONFIG SET slowlog-log-slower-than 10000

# Check slow queries
SLOWLOG GET 10

# Use pipelining for bulk operations
# Avoid KEYS command in production
# Set appropriate TTLs
# Use appropriate data structures
```

## Resources

- **Redis Commands**: https://redis.io/commands
- **phpredis Documentation**: https://github.com/phpredis/phpredis
- **Predis Documentation**: https://github.com/predis/predis
- **Laravel Redis**: https://laravel.com/docs/redis
- **Monitor in Devilbox**: http://localhost/vendor/phpcacheadmin-2.4.1/
