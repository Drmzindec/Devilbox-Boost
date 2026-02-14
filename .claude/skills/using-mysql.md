# Using MySQL/MariaDB in Devilbox

MySQL and MariaDB are the most widely-used relational databases for web applications. Devilbox supports both with seamless compatibility.

## Quick Start

### Connection Details

```bash
Host: 127.0.0.1 (via port forwarding)
Port: 3306
Username: root
Password: root (set in .env)
Database: (create as needed)
```

### Test Connection

```bash
# From PHP container
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl -e "SELECT VERSION();"

# Enter interactive shell
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl

# Test with PHP
docker-compose exec php php -r '$mysqli = new mysqli("127.0.0.1", "root", "root"); echo $mysqli->server_info . "\n";'
```

### MySQL vs MariaDB

```bash
# Check which you're using
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl -e "SELECT VERSION();"

# MariaDB: 10.x.x-MariaDB
# MySQL: 8.x.x
```

## Laravel Configuration

### 1. Configure Connection

**`.env`**:
```env
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=my_database
DB_USERNAME=root
DB_PASSWORD=root
```

### 2. Create Database

```bash
# Create database
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl \
    -e "CREATE DATABASE IF NOT EXISTS my_database CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# Or via Laravel migration
php artisan migrate
```

### 3. Common Laravel Database Operations

**Migrations:**
```php
<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class CreateProductsTable extends Migration
{
    public function up()
    {
        Schema::create('products', function (Blueprint $table) {
            $table->id();
            $table->string('name');
            $table->string('slug')->unique();
            $table->decimal('price', 10, 2);
            $table->text('description')->nullable();
            $table->json('attributes')->nullable();
            $table->unsignedBigInteger('category_id');
            $table->timestamps();
            $table->softDeletes();

            $table->foreign('category_id')
                ->references('id')
                ->on('categories')
                ->onDelete('cascade');

            $table->index(['category_id', 'created_at']);
            $table->fullText(['name', 'description']);
        });
    }

    public function down()
    {
        Schema::dropIfExists('products');
    }
}
```

**Eloquent Models:**
```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\SoftDeletes;

class Product extends Model
{
    use SoftDeletes;

    protected $fillable = [
        'name',
        'slug',
        'price',
        'description',
        'attributes',
        'category_id',
    ];

    protected $casts = [
        'price' => 'decimal:2',
        'attributes' => 'array',
    ];

    // Relationships
    public function category()
    {
        return $this->belongsTo(Category::class);
    }

    public function orders()
    {
        return $this->belongsToMany(Order::class)->withPivot('quantity', 'price');
    }

    // Scopes
    public function scopeExpensive($query)
    {
        return $query->where('price', '>', 1000);
    }

    // Accessors
    public function getFormattedPriceAttribute()
    {
        return '$' . number_format($this->price, 2);
    }
}
```

**Query Examples:**
```php
<?php

// Basic queries
$products = Product::all();
$product = Product::find(1);
$product = Product::where('slug', 'laptop')->first();

// Eager loading (N+1 prevention)
$products = Product::with('category', 'orders')->get();

// Pagination
$products = Product::paginate(15);

// Complex queries
$products = Product::where('price', '>', 100)
    ->where('category_id', 5)
    ->orderBy('created_at', 'desc')
    ->limit(10)
    ->get();

// Joins
$products = Product::join('categories', 'products.category_id', '=', 'categories.id')
    ->select('products.*', 'categories.name as category_name')
    ->get();

// Aggregates
$avgPrice = Product::avg('price');
$count = Product::where('category_id', 5)->count();
$total = Product::sum('price');

// Raw queries
$products = DB::select('SELECT * FROM products WHERE price > ?', [100]);

// Transactions
DB::transaction(function () {
    $product = Product::create(['name' => 'New Product', 'price' => 99.99]);
    $product->orders()->attach($orderId, ['quantity' => 1]);
});
```

## WordPress Configuration

### 1. Create WordPress Database

```bash
# Create database
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl \
    -e "CREATE DATABASE IF NOT EXISTS wordpress CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
```

### 2. wp-config.php

```php
<?php

define('DB_NAME', 'wordpress');
define('DB_USER', 'root');
define('DB_PASSWORD', 'root');
define('DB_HOST', '127.0.0.1');
define('DB_CHARSET', 'utf8mb4');
define('DB_COLLATE', 'utf8mb4_unicode_ci');

// Enable debug mode
define('WP_DEBUG', true);
define('WP_DEBUG_LOG', true);
define('WP_DEBUG_DISPLAY', false);
```

### 3. Common WordPress Database Operations

```php
<?php

global $wpdb;

// Insert
$wpdb->insert(
    $wpdb->prefix . 'custom_table',
    [
        'name' => 'John Doe',
        'email' => 'john@example.com',
    ],
    ['%s', '%s']
);

// Update
$wpdb->update(
    $wpdb->prefix . 'custom_table',
    ['name' => 'Jane Doe'],
    ['id' => 1],
    ['%s'],
    ['%d']
);

// Query
$results = $wpdb->get_results("
    SELECT * FROM {$wpdb->prefix}custom_table
    WHERE email LIKE '%@example.com'
");

// Get single value
$count = $wpdb->get_var("SELECT COUNT(*) FROM {$wpdb->prefix}posts WHERE post_status = 'publish'");

// Prepared statement
$results = $wpdb->get_results($wpdb->prepare("
    SELECT * FROM {$wpdb->prefix}posts
    WHERE post_author = %d AND post_status = %s
", 1, 'publish'));
```

## Direct PHP Usage

### 1. Using MySQLi

```php
<?php

// Connect
$mysqli = new mysqli('127.0.0.1', 'root', 'root', 'my_database');

if ($mysqli->connect_error) {
    die('Connection failed: ' . $mysqli->connect_error);
}

// Set charset
$mysqli->set_charset('utf8mb4');

// Create table
$mysqli->query('
    CREATE TABLE IF NOT EXISTS users (
        id INT AUTO_INCREMENT PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        email VARCHAR(100) UNIQUE NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
');

// Insert
$stmt = $mysqli->prepare('INSERT INTO users (name, email) VALUES (?, ?)');
$stmt->bind_param('ss', $name, $email);
$name = 'John Doe';
$email = 'john@example.com';
$stmt->execute();
$insertId = $stmt->insert_id;
$stmt->close();

// Query
$result = $mysqli->query('SELECT * FROM users');
while ($row = $result->fetch_assoc()) {
    echo $row['name'] . ': ' . $row['email'] . "\n";
}

// Prepared statement query
$stmt = $mysqli->prepare('SELECT * FROM users WHERE email = ?');
$stmt->bind_param('s', $email);
$email = 'john@example.com';
$stmt->execute();
$result = $stmt->get_result();
$user = $result->fetch_assoc();
$stmt->close();

// Transaction
$mysqli->begin_transaction();
try {
    $mysqli->query('UPDATE accounts SET balance = balance - 100 WHERE id = 1');
    $mysqli->query('UPDATE accounts SET balance = balance + 100 WHERE id = 2');
    $mysqli->commit();
} catch (Exception $e) {
    $mysqli->rollback();
    throw $e;
}

// Close
$mysqli->close();
```

### 2. Using PDO

```php
<?php

// Connect
$dsn = 'mysql:host=127.0.0.1;port=3306;dbname=my_database;charset=utf8mb4';
$pdo = new PDO($dsn, 'root', 'root', [
    PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
    PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
    PDO::ATTR_EMULATE_PREPARES => false,
]);

// Insert
$stmt = $pdo->prepare('INSERT INTO users (name, email) VALUES (?, ?)');
$stmt->execute(['John Doe', 'john@example.com']);
$insertId = $pdo->lastInsertId();

// Query
$stmt = $pdo->query('SELECT * FROM users');
$users = $stmt->fetchAll();

// Named parameters
$stmt = $pdo->prepare('SELECT * FROM users WHERE email = :email');
$stmt->execute(['email' => 'john@example.com']);
$user = $stmt->fetch();

// Transaction
$pdo->beginTransaction();
try {
    $pdo->exec('UPDATE accounts SET balance = balance - 100 WHERE id = 1');
    $pdo->exec('UPDATE accounts SET balance = balance + 100 WHERE id = 2');
    $pdo->commit();
} catch (Exception $e) {
    $pdo->rollback();
    throw $e;
}
```

## Common Use Cases

### 1. Full-Text Search

```php
<?php

// Create table with FULLTEXT index
$mysqli->query('
    CREATE TABLE articles (
        id INT AUTO_INCREMENT PRIMARY KEY,
        title VARCHAR(200),
        content TEXT,
        FULLTEXT INDEX ft_search (title, content)
    ) ENGINE=InnoDB
');

// Insert articles
$mysqli->query("
    INSERT INTO articles (title, content) VALUES
    ('MySQL Tutorial', 'Learn MySQL database basics and advanced features'),
    ('PHP Best Practices', 'Discover modern PHP development techniques')
");

// Search (boolean mode)
$result = $mysqli->query("
    SELECT *, MATCH(title, content) AGAINST('mysql database' IN BOOLEAN MODE) as relevance
    FROM articles
    WHERE MATCH(title, content) AGAINST('mysql database' IN BOOLEAN MODE)
    ORDER BY relevance DESC
");

// Search (natural language mode)
$result = $mysqli->query("
    SELECT *, MATCH(title, content) AGAINST('PHP programming') as score
    FROM articles
    WHERE MATCH(title, content) AGAINST('PHP programming')
    ORDER BY score DESC
");
```

### 2. JSON Columns (MySQL 5.7+)

```php
<?php

// Create table
$mysqli->query('
    CREATE TABLE products (
        id INT AUTO_INCREMENT PRIMARY KEY,
        name VARCHAR(100),
        attributes JSON
    ) ENGINE=InnoDB
');

// Insert with JSON
$stmt = $mysqli->prepare('INSERT INTO products (name, attributes) VALUES (?, ?)');
$stmt->bind_param('ss', $name, $attributes);
$name = 'Laptop';
$attributes = json_encode([
    'brand' => 'TechCorp',
    'specs' => [
        'cpu' => 'Intel i7',
        'ram' => '16GB',
        'storage' => '512GB SSD',
    ],
]);
$stmt->execute();

// Query JSON fields
$result = $mysqli->query("
    SELECT *
    FROM products
    WHERE JSON_EXTRACT(attributes, '$.brand') = 'TechCorp'
");

// Or using -> operator
$result = $mysqli->query("
    SELECT *
    FROM products
    WHERE attributes->'$.specs.ram' = '\"16GB\"'
");

// Update JSON field
$mysqli->query("
    UPDATE products
    SET attributes = JSON_SET(attributes, '$.warranty', '2 years')
    WHERE id = 1
");

// Array contains
$result = $mysqli->query("
    SELECT *
    FROM products
    WHERE JSON_CONTAINS(attributes->'$.tags', '\"featured\"')
");
```

### 3. Stored Procedures

```php
<?php

// Create stored procedure
$mysqli->query('
    DELIMITER //
    CREATE PROCEDURE GetUserOrders(IN userId INT)
    BEGIN
        SELECT
            o.id,
            o.total,
            o.created_at
        FROM orders o
        WHERE o.user_id = userId
        ORDER BY o.created_at DESC;
    END //
    DELIMITER ;
');

// Call procedure
$stmt = $mysqli->prepare('CALL GetUserOrders(?)');
$stmt->bind_param('i', $userId);
$userId = 123;
$stmt->execute();
$result = $stmt->get_result();
$orders = $result->fetch_all(MYSQLI_ASSOC);
```

### 4. Triggers

```php
<?php

// Create audit log table
$mysqli->query('
    CREATE TABLE audit_log (
        id INT AUTO_INCREMENT PRIMARY KEY,
        table_name VARCHAR(50),
        operation VARCHAR(10),
        record_id INT,
        old_data JSON,
        new_data JSON,
        changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        changed_by VARCHAR(50)
    ) ENGINE=InnoDB
');

// Create trigger
$mysqli->query('
    CREATE TRIGGER users_after_update
    AFTER UPDATE ON users
    FOR EACH ROW
    BEGIN
        INSERT INTO audit_log (table_name, operation, record_id, old_data, new_data, changed_by)
        VALUES (
            "users",
            "UPDATE",
            OLD.id,
            JSON_OBJECT("name", OLD.name, "email", OLD.email),
            JSON_OBJECT("name", NEW.name, "email", NEW.email),
            USER()
        );
    END
');
```

### 5. Partitioning (Large Tables)

```php
<?php

// Create partitioned table
$mysqli->query('
    CREATE TABLE logs (
        id BIGINT AUTO_INCREMENT,
        log_date DATE NOT NULL,
        message TEXT,
        PRIMARY KEY (id, log_date)
    ) ENGINE=InnoDB
    PARTITION BY RANGE (YEAR(log_date)) (
        PARTITION p2023 VALUES LESS THAN (2024),
        PARTITION p2024 VALUES LESS THAN (2025),
        PARTITION p2025 VALUES LESS THAN (2026),
        PARTITION pmax VALUES LESS THAN MAXVALUE
    )
');

// Query automatically uses partition pruning
$result = $mysqli->query("
    SELECT * FROM logs
    WHERE log_date BETWEEN '2024-01-01' AND '2024-12-31'
");
```

## Performance Optimization

### 1. Indexing Strategies

```php
<?php

// Single column index
$mysqli->query('CREATE INDEX idx_email ON users (email)');

// Composite index (order matters!)
$mysqli->query('CREATE INDEX idx_name_date ON products (category_id, created_at DESC)');

// Unique index
$mysqli->query('CREATE UNIQUE INDEX idx_slug ON products (slug)');

// Covering index (includes all queried columns)
$mysqli->query('CREATE INDEX idx_product_cover ON products (category_id, price, name)');

// Analyze index usage
$result = $mysqli->query('
    SHOW INDEX FROM products
');

// Check query execution plan
$result = $mysqli->query('
    EXPLAIN SELECT * FROM products WHERE category_id = 5 ORDER BY created_at DESC
');
```

### 2. Query Optimization

```php
<?php

// ❌ Bad: N+1 query problem
$products = $mysqli->query('SELECT * FROM products');
while ($product = $products->fetch_assoc()) {
    $category = $mysqli->query('SELECT * FROM categories WHERE id = ' . $product['category_id']);
    // Process...
}

// ✅ Good: JOIN to fetch in one query
$result = $mysqli->query('
    SELECT p.*, c.name as category_name
    FROM products p
    LEFT JOIN categories c ON p.category_id = c.id
');

// ❌ Bad: SELECT *
$result = $mysqli->query('SELECT * FROM products');

// ✅ Good: Select only needed columns
$result = $mysqli->query('SELECT id, name, price FROM products');

// ❌ Bad: No LIMIT on large tables
$result = $mysqli->query('SELECT * FROM logs');

// ✅ Good: Use LIMIT for pagination
$result = $mysqli->query('SELECT * FROM logs ORDER BY created_at DESC LIMIT 100 OFFSET 0');

// Use prepared statements to enable query caching
$stmt = $mysqli->prepare('SELECT * FROM products WHERE category_id = ?');
```

### 3. Connection Management

```php
<?php

// Use persistent connections
$mysqli = new mysqli('p:127.0.0.1', 'root', 'root', 'my_database');

// Or with PDO
$pdo = new PDO($dsn, 'root', 'root', [
    PDO::ATTR_PERSISTENT => true,
]);

// Connection pooling (Laravel)
// config/database.php
'mysql' => [
    'driver' => 'mysql',
    'host' => '127.0.0.1',
    'database' => 'my_database',
    'username' => 'root',
    'password' => 'root',
    'options' => [
        PDO::ATTR_PERSISTENT => true,
    ],
],
```

### 4. Caching Strategies

```php
<?php

// Query result caching (MySQL query cache deprecated in 8.0)
// Use application-level caching instead

// Laravel cache example
use Illuminate\Support\Facades\Cache;

$products = Cache::remember('products.featured', 3600, function () {
    return Product::where('featured', true)->get();
});

// Manual caching with Redis
$redis = new Redis();
$redis->connect('127.0.0.1', 6379);

$cacheKey = 'products:category:5';
$products = $redis->get($cacheKey);

if (!$products) {
    $result = $mysqli->query('SELECT * FROM products WHERE category_id = 5');
    $products = $result->fetch_all(MYSQLI_ASSOC);
    $redis->setex($cacheKey, 300, json_encode($products));
}
```

## Monitoring & Debugging

### Check Database Stats

```bash
# Connect to MySQL
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl

# Show databases
SHOW DATABASES;

# Use database
USE my_database;

# Show tables
SHOW TABLES;

# Describe table
DESCRIBE users;
SHOW CREATE TABLE users;

# Database size
SELECT
    table_schema AS 'Database',
    ROUND(SUM(data_length + index_length) / 1024 / 1024, 2) AS 'Size (MB)'
FROM information_schema.tables
GROUP BY table_schema;

# Table sizes
SELECT
    table_name,
    ROUND(((data_length + index_length) / 1024 / 1024), 2) AS 'Size (MB)'
FROM information_schema.tables
WHERE table_schema = 'my_database'
ORDER BY (data_length + index_length) DESC;

# Active connections
SHOW PROCESSLIST;

# Variables
SHOW VARIABLES LIKE 'max_connections';
SHOW VARIABLES LIKE '%cache%';

# Status
SHOW STATUS LIKE 'Threads_connected';
SHOW STATUS LIKE '%slow%';
```

### Using phpMyAdmin

```bash
# Open in browser
open http://localhost/vendor/phpmyadmin-5.2.3/

# Login:
# Server: 127.0.0.1
# Username: root
# Password: root

# Features:
# - Database browser
# - SQL query editor
# - Import/Export
# - User management
# - Performance monitoring
```

### Slow Query Log

```bash
# Enable slow query log
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl -e "
    SET GLOBAL slow_query_log = 'ON';
    SET GLOBAL long_query_time = 1;
    SET GLOBAL log_queries_not_using_indexes = 'ON';
"

# View slow queries (in container)
docker-compose exec mysql-8.0 tail -f /var/log/mysql/slow.log
```

## Troubleshooting

### Connection Failed

```bash
# Check MySQL is running
docker-compose ps mysql

# Test from PHP container
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl -e "SELECT 1;"

# Check port forwarding
docker-compose exec php netstat -tlnp | grep 3306

# Check logs
docker-compose logs mysql
```

### Character Encoding Issues

```sql
-- Check database charset
SHOW CREATE DATABASE my_database;

-- Change database charset
ALTER DATABASE my_database CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- Change table charset
ALTER TABLE users CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- Set connection charset
SET NAMES utf8mb4;
```

### Lock Wait Timeout

```sql
-- Check locked tables
SHOW OPEN TABLES WHERE In_use > 0;

-- Check current transactions
SELECT * FROM information_schema.innodb_trx;

-- Kill blocking transaction
KILL <process_id>;

-- Increase timeout
SET GLOBAL innodb_lock_wait_timeout = 120;
```

### Table Corruption

```sql
-- Check table
CHECK TABLE users;

-- Repair table
REPAIR TABLE users;

-- Optimize table
OPTIMIZE TABLE users;
```

## MySQL 8 vs MariaDB Differences

### MySQL 8 Features:
- Window functions
- CTEs (WITH clause)
- Better JSON support
- Document store (X DevAPI)
- Improved performance

### MariaDB Features:
- Better backward compatibility
- More storage engines
- Galera cluster built-in
- Oracle compatibility mode
- Often faster in specific workloads

## Resources

- **MySQL Documentation**: https://dev.mysql.com/doc/
- **MariaDB Documentation**: https://mariadb.com/kb/en/
- **Laravel Database**: https://laravel.com/docs/database
- **WordPress Database**: https://developer.wordpress.org/apis/wpdb/
- **phpMyAdmin**: http://localhost/vendor/phpmyadmin-5.2.3/
- **Adminer**: http://localhost/vendor/adminer-5.4.2-devilbox.php
