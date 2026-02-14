# Using MongoDB in Devilbox

MongoDB is a NoSQL document database perfect for flexible schemas, high-performance writes, and complex data structures.

## Quick Start

### Connection Details

```bash
Host: 127.0.0.1 (via port forwarding)
Port: 27017
Username: (none by default)
Password: (none by default)
```

### Test Connection

```bash
# From PHP container
docker-compose exec php mongo --host 127.0.0.1 --port 27017 --eval "db.version()"

# Or using mongosh (MongoDB Shell)
docker-compose exec php mongosh --host 127.0.0.1:27017

# Test with PHP
docker-compose exec php php -r '$m = new MongoDB\Driver\Manager("mongodb://127.0.0.1:27017"); var_dump($m);'
```

## Laravel Configuration

### 1. Install MongoDB Package

```bash
# Install Laravel MongoDB package
docker-compose exec php composer require mongodb/laravel-mongodb
```

### 2. Configure Connection

**`.env`**:
```env
DB_CONNECTION=mongodb
DB_HOST=127.0.0.1
DB_PORT=27017
DB_DATABASE=my_database
DB_USERNAME=
DB_PASSWORD=
```

**`config/database.php`**:
```php
'connections' => [
    'mongodb' => [
        'driver' => 'mongodb',
        'host' => env('DB_HOST', '127.0.0.1'),
        'port' => env('DB_PORT', 27017),
        'database' => env('DB_DATABASE', 'my_database'),
        'username' => env('DB_USERNAME', ''),
        'password' => env('DB_PASSWORD', ''),
        'options' => [
            'database' => env('DB_AUTHENTICATION_DATABASE', 'admin'),
        ],
    ],
],
```

### 3. Using Eloquent with MongoDB

**Model:**
```php
<?php

namespace App\Models;

use MongoDB\Laravel\Eloquent\Model;

class Product extends Model
{
    protected $connection = 'mongodb';
    protected $collection = 'products';

    protected $fillable = [
        'name',
        'price',
        'description',
        'tags',
        'metadata',
    ];

    protected $casts = [
        'price' => 'decimal:2',
        'tags' => 'array',
        'metadata' => 'array',
    ];
}
```

**Usage:**
```php
<?php

// Create document
$product = Product::create([
    'name' => 'Laptop',
    'price' => 999.99,
    'description' => 'High-performance laptop',
    'tags' => ['electronics', 'computers'],
    'metadata' => [
        'brand' => 'TechCorp',
        'warranty' => '2 years',
        'specs' => [
            'cpu' => 'Intel i7',
            'ram' => '16GB',
            'storage' => '512GB SSD',
        ],
    ],
]);

// Query
$products = Product::where('price', '<', 1000)
    ->whereIn('tags', ['electronics'])
    ->get();

// Update
Product::where('name', 'Laptop')
    ->update(['price' => 899.99]);

// Array operations
Product::where('_id', $id)->push('tags', 'featured');
```

## WordPress Configuration

### 1. Using MongoDB as Object Cache

Install plugin or use custom implementation:

**`wp-content/db.php`**:
```php
<?php
// Custom MongoDB implementation for WP
// Note: This is a simplified example

global $wpdb;

class WP_MongoDB {
    private $client;
    private $database;

    public function __construct() {
        $this->client = new MongoDB\Client('mongodb://127.0.0.1:27017');
        $this->database = $this->client->wordpress;
    }

    public function get($key) {
        $doc = $this->database->cache->findOne(['_id' => $key]);
        return $doc ? $doc['value'] : false;
    }

    public function set($key, $value, $expiration = 0) {
        $this->database->cache->updateOne(
            ['_id' => $key],
            [
                '$set' => [
                    'value' => $value,
                    'expires' => $expiration > 0 ? time() + $expiration : 0,
                ],
            ],
            ['upsert' => true]
        );
    }
}
```

### 2. Custom Post Type in MongoDB

```php
<?php

function store_form_submission($data) {
    $client = new MongoDB\Client('mongodb://127.0.0.1:27017');
    $collection = $client->mysite->submissions;

    $result = $collection->insertOne([
        'name' => $data['name'],
        'email' => $data['email'],
        'message' => $data['message'],
        'ip_address' => $_SERVER['REMOTE_ADDR'],
        'created_at' => new MongoDB\BSON\UTCDateTime(),
    ]);

    return $result->getInsertedId();
}
```

## Direct PHP Usage

### 1. Using MongoDB Extension

```php
<?php

// Connect to MongoDB
$manager = new MongoDB\Driver\Manager('mongodb://127.0.0.1:27017');

// Insert document
$bulk = new MongoDB\Driver\BulkWrite();
$document = [
    'name' => 'John Doe',
    'email' => 'john@example.com',
    'age' => 30,
    'tags' => ['developer', 'php'],
    'created_at' => new MongoDB\BSON\UTCDateTime(),
];
$bulk->insert($document);
$manager->executeBulkWrite('mydb.users', $bulk);

// Query documents
$filter = ['age' => ['$gte' => 18]];
$options = [
    'projection' => ['_id' => 0, 'name' => 1, 'email' => 1],
    'sort' => ['name' => 1],
    'limit' => 10,
];

$query = new MongoDB\Driver\Query($filter, $options);
$cursor = $manager->executeQuery('mydb.users', $query);

foreach ($cursor as $document) {
    echo $document->name . ': ' . $document->email . "\n";
}

// Update document
$bulk = new MongoDB\Driver\BulkWrite();
$bulk->update(
    ['email' => 'john@example.com'],
    ['$set' => ['age' => 31]],
    ['multi' => false, 'upsert' => false]
);
$manager->executeBulkWrite('mydb.users', $bulk);

// Delete document
$bulk = new MongoDB\Driver\BulkWrite();
$bulk->delete(['email' => 'john@example.com']);
$manager->executeBulkWrite('mydb.users', $bulk);
```

### 2. Using MongoDB Library (Recommended)

```bash
# Install via Composer
docker-compose exec php composer require mongodb/mongodb
```

```php
<?php

require 'vendor/autoload.php';

// Connect
$client = new MongoDB\Client('mongodb://127.0.0.1:27017');

// Get database and collection
$database = $client->mydb;
$collection = $database->users;

// Insert one
$result = $collection->insertOne([
    'name' => 'Jane Doe',
    'email' => 'jane@example.com',
    'age' => 28,
    'address' => [
        'street' => '123 Main St',
        'city' => 'New York',
        'zip' => '10001',
    ],
]);

echo "Inserted ID: " . $result->getInsertedId() . "\n";

// Insert many
$collection->insertMany([
    ['name' => 'Alice', 'age' => 25],
    ['name' => 'Bob', 'age' => 35],
    ['name' => 'Charlie', 'age' => 40],
]);

// Find one
$user = $collection->findOne(['email' => 'jane@example.com']);
echo $user['name'] . "\n";

// Find many
$cursor = $collection->find(
    ['age' => ['$gte' => 30]],
    [
        'sort' => ['age' => -1],
        'limit' => 10,
    ]
);

foreach ($cursor as $document) {
    echo $document['name'] . ': ' . $document['age'] . "\n";
}

// Update one
$collection->updateOne(
    ['email' => 'jane@example.com'],
    ['$set' => ['age' => 29]]
);

// Update many
$collection->updateMany(
    ['age' => ['$lt' => 30]],
    ['$inc' => ['age' => 1]]
);

// Delete one
$collection->deleteOne(['email' => 'jane@example.com']);

// Delete many
$collection->deleteMany(['age' => ['$lt' => 20]]);

// Count documents
$count = $collection->countDocuments(['age' => ['$gte' => 30]]);
echo "Users 30+: $count\n";
```

## Common Use Cases

### 1. Product Catalog with Flexible Schema

```php
<?php

$client = new MongoDB\Client('mongodb://127.0.0.1:27017');
$products = $client->ecommerce->products;

// Insert products with different attributes
$products->insertMany([
    [
        'type' => 'laptop',
        'name' => 'Pro Laptop',
        'price' => 1299.99,
        'specs' => [
            'cpu' => 'Intel i7',
            'ram' => '16GB',
            'storage' => '512GB SSD',
        ],
    ],
    [
        'type' => 'shirt',
        'name' => 'Cotton T-Shirt',
        'price' => 19.99,
        'attributes' => [
            'size' => 'M',
            'color' => 'blue',
            'material' => 'cotton',
        ],
    ],
]);

// Query by nested field
$laptops = $products->find([
    'type' => 'laptop',
    'specs.ram' => '16GB',
]);
```

### 2. User Activity Logging

```php
<?php

$client = new MongoDB\Client('mongodb://127.0.0.1:27017');
$logs = $client->analytics->user_activity;

function logActivity($userId, $action, $metadata = []) {
    global $logs;

    $logs->insertOne([
        'user_id' => $userId,
        'action' => $action,
        'metadata' => $metadata,
        'ip_address' => $_SERVER['REMOTE_ADDR'],
        'user_agent' => $_SERVER['HTTP_USER_AGENT'],
        'timestamp' => new MongoDB\BSON\UTCDateTime(),
    ]);
}

// Log activity
logActivity(123, 'page_view', ['page' => '/products']);
logActivity(123, 'add_to_cart', ['product_id' => 456]);

// Query recent activity
$recentActivity = $logs->find(
    ['user_id' => 123],
    [
        'sort' => ['timestamp' => -1],
        'limit' => 10,
    ]
);
```

### 3. Full-Text Search

```php
<?php

$client = new MongoDB\Client('mongodb://127.0.0.1:27017');
$articles = $client->blog->articles;

// Create text index (run once)
$articles->createIndex(['title' => 'text', 'content' => 'text']);

// Insert articles
$articles->insertMany([
    [
        'title' => 'Getting Started with MongoDB',
        'content' => 'MongoDB is a NoSQL database...',
        'tags' => ['mongodb', 'database'],
    ],
    [
        'title' => 'PHP Best Practices',
        'content' => 'Learn how to write better PHP code...',
        'tags' => ['php', 'programming'],
    ],
]);

// Search
$results = $articles->find([
    '$text' => ['$search' => 'mongodb database'],
]);

foreach ($results as $article) {
    echo $article['title'] . "\n";
}
```

### 4. Aggregation Pipeline

```php
<?php

$client = new MongoDB\Client('mongodb://127.0.0.1:27017');
$orders = $client->ecommerce->orders;

// Calculate total revenue by product category
$pipeline = [
    [
        '$match' => [
            'status' => 'completed',
            'created_at' => [
                '$gte' => new MongoDB\BSON\UTCDateTime(strtotime('-30 days') * 1000),
            ],
        ],
    ],
    [
        '$group' => [
            '_id' => '$category',
            'total_revenue' => ['$sum' => '$amount'],
            'order_count' => ['$sum' => 1],
            'avg_order' => ['$avg' => '$amount'],
        ],
    ],
    [
        '$sort' => ['total_revenue' => -1],
    ],
];

$results = $orders->aggregate($pipeline);

foreach ($results as $stat) {
    printf(
        "Category: %s | Revenue: $%.2f | Orders: %d | Avg: $%.2f\n",
        $stat['_id'],
        $stat['total_revenue'],
        $stat['order_count'],
        $stat['avg_order']
    );
}
```

### 5. Geospatial Queries

```php
<?php

$client = new MongoDB\Client('mongodb://127.0.0.1:27017');
$locations = $client->maps->stores;

// Create geospatial index
$locations->createIndex(['location' => '2dsphere']);

// Insert stores with coordinates
$locations->insertMany([
    [
        'name' => 'Store A',
        'location' => [
            'type' => 'Point',
            'coordinates' => [-73.97, 40.77], // [longitude, latitude]
        ],
    ],
    [
        'name' => 'Store B',
        'location' => [
            'type' => 'Point',
            'coordinates' => [-73.98, 40.76],
        ],
    ],
]);

// Find stores near a point
$nearbyStores = $locations->find([
    'location' => [
        '$near' => [
            '$geometry' => [
                'type' => 'Point',
                'coordinates' => [-73.97, 40.77],
            ],
            '$maxDistance' => 1000, // meters
        ],
    ],
]);

foreach ($nearbyStores as $store) {
    echo $store['name'] . "\n";
}
```

## Schema Design Best Practices

### 1. Embedding vs Referencing

**Embed** when:
- Data is always needed together
- Data doesn't change often
- One-to-few relationships

```php
<?php
// Good: Embed addresses in user document
$users->insertOne([
    'name' => 'John Doe',
    'addresses' => [
        ['type' => 'home', 'street' => '123 Main St', 'city' => 'NYC'],
        ['type' => 'work', 'street' => '456 Office Blvd', 'city' => 'NYC'],
    ],
]);
```

**Reference** when:
- Data is accessed independently
- Data changes frequently
- One-to-many or many-to-many relationships

```php
<?php
// Good: Reference authors in books
$authors->insertOne(['_id' => 1, 'name' => 'John Doe']);
$books->insertOne([
    'title' => 'My Book',
    'author_id' => 1, // Reference
]);
```

### 2. Avoid Deep Nesting

```php
<?php
// ❌ Bad: Too deeply nested
$document = [
    'user' => [
        'profile' => [
            'address' => [
                'location' => [
                    'coordinates' => [
                        'lat' => 40.77,
                        'lng' => -73.97,
                    ],
                ],
            ],
        ],
    ],
];

// ✅ Good: Flattened structure
$document = [
    'user_id' => 123,
    'address_street' => '123 Main St',
    'address_city' => 'NYC',
    'location_lat' => 40.77,
    'location_lng' => -73.97,
];
```

## Performance Optimization

### 1. Indexing

```php
<?php

$collection = $client->mydb->users;

// Single field index
$collection->createIndex(['email' => 1], ['unique' => true]);

// Compound index
$collection->createIndex(['age' => 1, 'name' => 1]);

// Text index
$collection->createIndex(['bio' => 'text']);

// TTL index (auto-delete after expiration)
$collection->createIndex(
    ['created_at' => 1],
    ['expireAfterSeconds' => 3600]
);

// List indexes
foreach ($collection->listIndexes() as $index) {
    echo $index['name'] . "\n";
}
```

### 2. Projection (Select Only Needed Fields)

```php
<?php

// ❌ Bad: Fetch everything
$users = $collection->find(['age' => ['$gte' => 18]]);

// ✅ Good: Fetch only needed fields
$users = $collection->find(
    ['age' => ['$gte' => 18]],
    ['projection' => ['name' => 1, 'email' => 1, '_id' => 0]]
);
```

### 3. Bulk Operations

```php
<?php

// ❌ Bad: Multiple single operations
for ($i = 0; $i < 1000; $i++) {
    $collection->insertOne(['value' => $i]);
}

// ✅ Good: Bulk insert
$documents = [];
for ($i = 0; $i < 1000; $i++) {
    $documents[] = ['value' => $i];
}
$collection->insertMany($documents);
```

## Monitoring & Debugging

### Check MongoDB Stats

```bash
# Connect to MongoDB shell
docker-compose exec php mongosh --host 127.0.0.1:27017

# Show databases
show dbs

# Use database
use mydb

# Show collections
show collections

# Collection stats
db.users.stats()

# Database stats
db.stats()

# Current operations
db.currentOp()

# Server status
db.serverStatus()
```

### Using MongoDB Compass

```bash
# Open MongoDB Compass (GUI tool)
# Connection string: mongodb://127.0.0.1:27017

# Features:
# - Visual query builder
# - Schema analysis
# - Index management
# - Performance monitoring
# - Aggregation pipeline builder
```

### Query Profiling

```php
<?php

$client = new MongoDB\Client('mongodb://127.0.0.1:27017');
$database = $client->mydb;

// Enable profiling (2 = all operations)
$database->command(['profile' => 2]);

// Run queries...
$collection = $database->users;
$collection->find(['age' => ['$gte' => 30]]);

// View slow queries
$profile = $database->system->profile;
$slowQueries = $profile->find(
    ['millis' => ['$gt' => 100]],
    ['sort' => ['ts' => -1], 'limit' => 10]
);

foreach ($slowQueries as $query) {
    print_r($query);
}

// Disable profiling
$database->command(['profile' => 0]);
```

## Troubleshooting

### Connection Failed

```bash
# Check MongoDB is running
docker-compose ps mongo

# Test from PHP container
docker-compose exec php mongosh --host 127.0.0.1:27017

# Check port forwarding
docker-compose exec php netstat -tlnp | grep 27017

# Check logs
docker-compose logs mongo
```

### Slow Queries

```bash
# Enable slow query log in mongosh
db.setProfilingLevel(1, { slowms: 100 })

# View slow queries
db.system.profile.find().limit(5).sort({ ts: -1 }).pretty()

# Check if indexes are being used
db.users.find({ email: 'test@example.com' }).explain('executionStats')
```

### Memory Issues

```bash
# Check memory usage
docker stats devilbox-mongo-1

# Increase in docker-compose.override.yml
services:
  mongo:
    command: ["--wiredTigerCacheSizeGB", "2"]
```

## MongoDB vs SQL - When to Use

### Use MongoDB When:

✅ Schema changes frequently
✅ Need horizontal scalability
✅ Complex nested data structures
✅ Rapid development / prototyping
✅ Large amounts of unstructured data
✅ High write throughput

### Use SQL When:

✅ Complex relationships and joins
✅ ACID transactions critical
✅ Established schema is stable
✅ Reporting and analytics
✅ Regulatory compliance needs

## Resources

- **MongoDB Manual**: https://www.mongodb.com/docs/manual/
- **MongoDB PHP Library**: https://www.mongodb.com/docs/php-library/current/
- **MongoDB University**: https://university.mongodb.com/
- **Laravel MongoDB**: https://github.com/mongodb/laravel-mongodb
- **MongoDB Compass**: https://www.mongodb.com/products/compass
