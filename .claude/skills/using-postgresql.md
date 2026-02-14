# Using PostgreSQL in Devilbox

PostgreSQL is an advanced open-source relational database with support for JSON, full-text search, geospatial data, and advanced indexing.

## Quick Start

### Connection Details

```bash
Host: 127.0.0.1 (via port forwarding)
Port: 5432
Database: postgres
Username: postgres
Password: (set in .env)
```

### Test Connection

```bash
# From PHP container
docker-compose exec php psql -h 127.0.0.1 -U postgres -c "SELECT version();"

# Enter interactive shell
docker-compose exec php psql -h 127.0.0.1 -U postgres

# Test with PHP
docker-compose exec php php -r '$conn = pg_connect("host=127.0.0.1 port=5432 dbname=postgres user=postgres"); var_dump($conn);'
```

## Laravel Configuration

### 1. Configure Connection

**`.env`**:
```env
DB_CONNECTION=pgsql
DB_HOST=127.0.0.1
DB_PORT=5432
DB_DATABASE=my_database
DB_USERNAME=postgres
DB_PASSWORD=postgres
```

### 2. Create Database

```bash
# Create database
docker-compose exec php psql -h 127.0.0.1 -U postgres -c "CREATE DATABASE my_database;"

# Or via Laravel migration
php artisan migrate
```

### 3. Using PostgreSQL-Specific Features

**JSON Columns:**
```php
<?php

// Migration
Schema::create('products', function (Blueprint $table) {
    $table->id();
    $table->string('name');
    $table->json('attributes'); // JSON column
    $table->jsonb('metadata'); // JSONB (binary JSON, faster)
    $table->timestamps();
});

// Model
class Product extends Model
{
    protected $casts = [
        'attributes' => 'array',
        'metadata' => 'array',
    ];
}

// Usage
$product = Product::create([
    'name' => 'Laptop',
    'attributes' => [
        'brand' => 'TechCorp',
        'warranty' => '2 years',
    ],
    'metadata' => [
        'specs' => [
            'cpu' => 'Intel i7',
            'ram' => '16GB',
        ],
    ],
]);

// Query JSON fields
$products = Product::whereJsonContains('attributes->brand', 'TechCorp')->get();
$products = Product::where('metadata->specs->ram', '16GB')->get();
```

**Array Columns:**
```php
<?php

// Migration
Schema::create('articles', function (Blueprint $table) {
    $table->id();
    $table->string('title');
    $table->text('tags')->default('{}'); // Array column
    $table->timestamps();
});

DB::statement('ALTER TABLE articles ALTER COLUMN tags TYPE text[] USING tags::text[]');

// Insert
DB::table('articles')->insert([
    'title' => 'My Article',
    'tags' => '{php,laravel,postgresql}',
]);

// Query
$articles = DB::table('articles')
    ->whereRaw("'php' = ANY(tags)")
    ->get();
```

**Full-Text Search:**
```php
<?php

// Migration
Schema::create('posts', function (Blueprint $table) {
    $table->id();
    $table->string('title');
    $table->text('content');
    $table->timestamps();
});

// Add tsvector column and index
DB::statement('ALTER TABLE posts ADD COLUMN searchable tsvector');
DB::statement('CREATE INDEX posts_searchable_idx ON posts USING GIN(searchable)');

// Update trigger to maintain search column
DB::statement("
    CREATE TRIGGER posts_searchable_update BEFORE INSERT OR UPDATE
    ON posts FOR EACH ROW EXECUTE PROCEDURE
    tsvector_update_trigger(searchable, 'pg_catalog.english', title, content)
");

// Search
$posts = DB::table('posts')
    ->whereRaw("searchable @@ to_tsquery('english', 'postgresql & database')")
    ->get();

// With ranking
$posts = DB::table('posts')
    ->selectRaw('*, ts_rank(searchable, to_tsquery(?)) as rank', ['postgresql'])
    ->whereRaw("searchable @@ to_tsquery(?)", ['postgresql'])
    ->orderBy('rank', 'desc')
    ->get();
```

## Direct PHP Usage

### 1. Using PDO

```php
<?php

// Connect
$dsn = 'pgsql:host=127.0.0.1;port=5432;dbname=postgres';
$pdo = new PDO($dsn, 'postgres', 'postgres');
$pdo->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

// Create table
$pdo->exec('
    CREATE TABLE IF NOT EXISTS users (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100) NOT NULL,
        email VARCHAR(100) UNIQUE NOT NULL,
        metadata JSONB,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
');

// Insert
$stmt = $pdo->prepare('INSERT INTO users (name, email, metadata) VALUES (?, ?, ?)');
$stmt->execute([
    'John Doe',
    'john@example.com',
    json_encode(['role' => 'admin', 'permissions' => ['read', 'write']]),
]);

// Query
$stmt = $pdo->query('SELECT * FROM users');
while ($row = $stmt->fetch(PDO::FETCH_ASSOC)) {
    echo $row['name'] . ': ' . $row['email'] . "\n";
}

// Prepared statement
$stmt = $pdo->prepare('SELECT * FROM users WHERE email = ?');
$stmt->execute(['john@example.com']);
$user = $stmt->fetch(PDO::FETCH_ASSOC);

// Transaction
$pdo->beginTransaction();
try {
    $pdo->exec('UPDATE users SET name = \'Jane Doe\' WHERE email = \'john@example.com\'');
    $pdo->exec('INSERT INTO audit_log (action) VALUES (\'user_updated\')');
    $pdo->commit();
} catch (Exception $e) {
    $pdo->rollBack();
    throw $e;
}
```

### 2. Using pg_* Functions

```php
<?php

// Connect
$conn = pg_connect('host=127.0.0.1 port=5432 dbname=postgres user=postgres password=postgres');

if (!$conn) {
    die('Connection failed: ' . pg_last_error());
}

// Query
$result = pg_query($conn, 'SELECT * FROM users');

while ($row = pg_fetch_assoc($result)) {
    echo $row['name'] . "\n";
}

// Prepared statement
$result = pg_query_params(
    $conn,
    'SELECT * FROM users WHERE email = $1',
    ['john@example.com']
);

// Insert with RETURNING
$result = pg_query($conn, "
    INSERT INTO users (name, email)
    VALUES ('Alice', 'alice@example.com')
    RETURNING id
");
$row = pg_fetch_assoc($result);
echo "Inserted ID: " . $row['id'] . "\n";

// Close
pg_close($conn);
```

## Common Use Cases

### 1. Advanced Queries with CTEs

```php
<?php

// Recursive CTE for hierarchical data
$query = "
    WITH RECURSIVE category_tree AS (
        -- Base case
        SELECT id, name, parent_id, 1 AS level
        FROM categories
        WHERE parent_id IS NULL

        UNION ALL

        -- Recursive case
        SELECT c.id, c.name, c.parent_id, ct.level + 1
        FROM categories c
        INNER JOIN category_tree ct ON c.parent_id = ct.id
    )
    SELECT * FROM category_tree ORDER BY level, name
";

$result = DB::select($query);
```

### 2. Window Functions

```php
<?php

// Rank products by price within each category
$products = DB::select("
    SELECT
        name,
        category,
        price,
        RANK() OVER (PARTITION BY category ORDER BY price DESC) as price_rank,
        AVG(price) OVER (PARTITION BY category) as category_avg_price
    FROM products
    ORDER BY category, price_rank
");

// Running total
$orders = DB::select("
    SELECT
        order_date,
        amount,
        SUM(amount) OVER (ORDER BY order_date) as running_total
    FROM orders
    ORDER BY order_date
");
```

### 3. Upsert (INSERT ... ON CONFLICT)

```php
<?php

// Insert or update
DB::statement("
    INSERT INTO user_stats (user_id, page_views, last_visit)
    VALUES (?, ?, NOW())
    ON CONFLICT (user_id) DO UPDATE
    SET
        page_views = user_stats.page_views + 1,
        last_visit = EXCLUDED.last_visit
", [123, 1]);

// Laravel query builder version
DB::table('user_stats')->upsert(
    ['user_id' => 123, 'page_views' => 1, 'last_visit' => now()],
    ['user_id'], // Unique columns
    ['page_views' => DB::raw('user_stats.page_views + 1'), 'last_visit'] // Update columns
);
```

### 4. JSONB Queries

```php
<?php

// Query JSONB data
$products = DB::table('products')
    ->whereRaw("metadata->>'brand' = ?", ['TechCorp'])
    ->get();

// Update JSONB field
DB::table('products')
    ->where('id', 1)
    ->update([
        'metadata' => DB::raw("jsonb_set(metadata, '{warranty}', '\"3 years\"')")
    ]);

// Array contains
$products = DB::table('products')
    ->whereRaw("metadata->'tags' @> ?", [json_encode(['featured'])])
    ->get();

// JSONB aggregation
$stats = DB::table('products')
    ->selectRaw("metadata->>'brand' as brand, COUNT(*) as count")
    ->groupBy(DB::raw("metadata->>'brand'"))
    ->get();
```

### 5. Geospatial Queries with PostGIS

```bash
# Enable PostGIS extension
docker-compose exec php psql -h 127.0.0.1 -U postgres -c "CREATE EXTENSION IF NOT EXISTS postgis;"
```

```php
<?php

// Create table with geometry
DB::statement('
    CREATE TABLE locations (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100),
        coordinates GEOGRAPHY(POINT, 4326)
    )
');

// Insert location
DB::statement("
    INSERT INTO locations (name, coordinates)
    VALUES (?, ST_SetSRID(ST_MakePoint(?, ?), 4326))
", ['Store A', -73.97, 40.77]); // longitude, latitude

// Find nearby locations (within 1km)
$nearby = DB::select("
    SELECT
        name,
        ST_Distance(
            coordinates,
            ST_SetSRID(ST_MakePoint(?, ?), 4326)
        ) as distance
    FROM locations
    WHERE ST_DWithin(
        coordinates,
        ST_SetSRID(ST_MakePoint(?, ?), 4326),
        1000
    )
    ORDER BY distance
", [-73.97, 40.77, -73.97, 40.77]);
```

## Performance Optimization

### 1. Indexing

```php
<?php

// Migration
Schema::table('users', function (Blueprint $table) {
    // B-tree index (default)
    $table->index('email');

    // Unique index
    $table->unique('username');

    // Composite index
    $table->index(['last_name', 'first_name']);

    // Partial index
    DB::statement('CREATE INDEX active_users_idx ON users (email) WHERE active = true');

    // GIN index for JSONB
    DB::statement('CREATE INDEX user_metadata_idx ON users USING GIN (metadata)');

    // GiST index for full-text search
    DB::statement('CREATE INDEX posts_content_idx ON posts USING GiST (to_tsvector(\'english\', content))');
});

// Check indexes
$indexes = DB::select("
    SELECT
        tablename,
        indexname,
        indexdef
    FROM pg_indexes
    WHERE tablename = 'users'
");
```

### 2. Query Optimization

```php
<?php

// Use EXPLAIN to analyze queries
$explain = DB::select("
    EXPLAIN ANALYZE
    SELECT * FROM users WHERE email = 'john@example.com'
");

// VACUUM to reclaim space and update statistics
DB::statement('VACUUM ANALYZE users');

// Materialized views for expensive queries
DB::statement('
    CREATE MATERIALIZED VIEW user_stats AS
    SELECT
        user_id,
        COUNT(*) as order_count,
        SUM(amount) as total_spent
    FROM orders
    GROUP BY user_id
');

// Refresh materialized view
DB::statement('REFRESH MATERIALIZED VIEW user_stats');
```

### 3. Connection Pooling

```php
<?php

// Use persistent connections
$pdo = new PDO(
    'pgsql:host=127.0.0.1;port=5432;dbname=postgres',
    'postgres',
    'postgres',
    [PDO::ATTR_PERSISTENT => true]
);

// Laravel: Increase max connections in config/database.php
'pgsql' => [
    'driver' => 'pgsql',
    'host' => env('DB_HOST', '127.0.0.1'),
    'port' => env('DB_PORT', '5432'),
    'database' => env('DB_DATABASE', 'postgres'),
    'username' => env('DB_USERNAME', 'postgres'),
    'password' => env('DB_PASSWORD', ''),
    'options' => [
        PDO::ATTR_PERSISTENT => true,
    ],
],
```

## Advanced Features

### 1. Triggers

```php
<?php

// Create audit log trigger
DB::statement('
    CREATE TABLE audit_log (
        id SERIAL PRIMARY KEY,
        table_name VARCHAR(50),
        operation VARCHAR(10),
        old_data JSONB,
        new_data JSONB,
        changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        changed_by VARCHAR(50)
    )
');

DB::statement('
    CREATE OR REPLACE FUNCTION audit_trigger_func()
    RETURNS TRIGGER AS $$
    BEGIN
        INSERT INTO audit_log (table_name, operation, old_data, new_data, changed_by)
        VALUES (
            TG_TABLE_NAME,
            TG_OP,
            CASE WHEN TG_OP IN (\'UPDATE\', \'DELETE\') THEN row_to_json(OLD) ELSE NULL END,
            CASE WHEN TG_OP IN (\'INSERT\', \'UPDATE\') THEN row_to_json(NEW) ELSE NULL END,
            current_user
        );
        RETURN NEW;
    END;
    $$ LANGUAGE plpgsql
');

DB::statement('
    CREATE TRIGGER users_audit_trigger
    AFTER INSERT OR UPDATE OR DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_func()
');
```

### 2. Custom Types

```php
<?php

// Create enum type
DB::statement("
    CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user', 'guest')
");

// Use in table
DB::statement('
    CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100),
        role user_role DEFAULT \'user\'
    )
');

// Insert
DB::statement("INSERT INTO users (name, role) VALUES (?, ?::user_role)", ['John', 'admin']);
```

### 3. Stored Procedures

```php
<?php

// Create stored procedure
DB::statement('
    CREATE OR REPLACE FUNCTION get_user_order_summary(p_user_id INT)
    RETURNS TABLE (
        total_orders BIGINT,
        total_amount NUMERIC,
        avg_amount NUMERIC
    ) AS $$
    BEGIN
        RETURN QUERY
        SELECT
            COUNT(*)::BIGINT,
            SUM(amount),
            AVG(amount)
        FROM orders
        WHERE user_id = p_user_id;
    END;
    $$ LANGUAGE plpgsql
');

// Call procedure
$result = DB::select('SELECT * FROM get_user_order_summary(?)', [123]);
```

## Monitoring & Debugging

### Check Database Stats

```bash
# Connect to PostgreSQL
docker-compose exec php psql -h 127.0.0.1 -U postgres

# List databases
\l

# List tables
\dt

# Describe table
\d users

# Database size
SELECT pg_size_pretty(pg_database_size('postgres'));

# Table sizes
SELECT
    tablename,
    pg_size_pretty(pg_total_relation_size(tablename::text))
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(tablename::text) DESC;

# Active connections
SELECT
    datname,
    count(*)
FROM pg_stat_activity
GROUP BY datname;

# Current queries
SELECT
    pid,
    usename,
    datname,
    query,
    state
FROM pg_stat_activity
WHERE state != 'idle';

# Kill slow query
SELECT pg_terminate_backend(12345); -- use pid from above
```

### Using phpPgAdmin

```bash
# Open in browser
open http://localhost/vendor/phppgadmin-7.13.0/

# Login:
# Server: 127.0.0.1:5432
# Username: postgres
# Password: postgres

# Features:
# - Database browser
# - Query editor
# - Table management
# - Import/Export
# - User management
```

### Performance Monitoring

```sql
-- Top 10 slowest queries
SELECT
    query,
    calls,
    total_exec_time,
    mean_exec_time,
    max_exec_time
FROM pg_stat_statements
ORDER BY total_exec_time DESC
LIMIT 10;

-- Table bloat
SELECT
    tablename,
    pg_size_pretty(pg_total_relation_size(tablename::text)) as size,
    n_dead_tup
FROM pg_stat_user_tables
ORDER BY n_dead_tup DESC;

-- Index usage
SELECT
    indexrelname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC;
```

## Troubleshooting

### Connection Failed

```bash
# Check PostgreSQL is running
docker-compose ps pgsql

# Test from PHP container
docker-compose exec php psql -h 127.0.0.1 -U postgres -c "SELECT 1;"

# Check port forwarding
docker-compose exec php netstat -tlnp | grep 5432

# Check logs
docker-compose logs pgsql
```

### Database Encoding Issues

```bash
# Create database with UTF-8
CREATE DATABASE mydb
    WITH ENCODING 'UTF8'
    LC_COLLATE='en_US.UTF-8'
    LC_CTYPE='en_US.UTF-8'
    TEMPLATE=template0;
```

### Disk Space Issues

```bash
# Check table bloat
VACUUM FULL ANALYZE;

# Drop unused tables/indexes
DROP TABLE old_table;
DROP INDEX unused_index;

# Truncate logs
TRUNCATE pg_stat_statements;
```

## PostgreSQL vs MySQL

### Use PostgreSQL When:

✅ Complex queries with JOINs and subqueries
✅ Need JSONB, arrays, or custom types
✅ Full-text search requirements
✅ Geospatial data (PostGIS)
✅ Advanced indexing (GiST, GIN)
✅ Window functions and CTEs
✅ Strict data integrity

### Use MySQL When:

✅ Simple read-heavy workloads
✅ Replication is priority
✅ Wide hosting compatibility
✅ WordPress or similar apps

## Resources

- **PostgreSQL Documentation**: https://www.postgresql.org/docs/
- **PostgreSQL Tutorial**: https://www.postgresqltutorial.com/
- **PostGIS Documentation**: https://postgis.net/documentation/
- **Laravel PostgreSQL**: https://laravel.com/docs/database
- **pgAdmin**: https://www.pgadmin.org/
- **phpPgAdmin**: http://localhost/vendor/phppgadmin-7.13.0/
