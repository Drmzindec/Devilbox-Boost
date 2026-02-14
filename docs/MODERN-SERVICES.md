# Modern Services Guide

Devilbox Boost includes optional modern services for search, email testing, message queues, and object storage.

---

## 🚀 Available Services

### 1. Meilisearch - Fast Search Engine

**What it is:** Lightning-fast, typo-tolerant search engine written in Rust.

**Use cases:**
- Full-text search for your applications
- Product catalogs
- Documentation search
- Real-time search-as-you-type

**Enable:**
```bash
cp compose/docker-compose.override.yml-meilisearch docker-compose.override.yml
docker compose up -d meilisearch
```

**Access:**
- API: `http://localhost:7700`
- Master Key: Set via `.env` → `MEILI_MASTER_KEY=your-secret-key`

**Usage in Laravel:**
```bash
composer require laravel/scout meilisearch/meilisearch-php
```

```php
// config/scout.php
'driver' => 'meilisearch',
'meilisearch' => [
    'host' => 'http://127.0.0.1:7700',
    'key' => env('MEILISEARCH_KEY'),
],
```

**PHP Example:**
```php
$client = new \MeiliSearch\Client('http://127.0.0.1:7700', 'masterKey');

// Create index
$index = $client->index('products');

// Add documents
$index->addDocuments([
    ['id' => 1, 'name' => 'Laptop', 'price' => 999],
    ['id' => 2, 'name' => 'Mouse', 'price' => 29],
]);

// Search
$results = $index->search('lap'); // Finds "Laptop" (typo-tolerant)
```

---

### 2. Mailpit - Modern Email Testing

**What it is:** Modern replacement for Mailhog with better UI and features.

**Use cases:**
- Test emails during development
- Preview emails without sending to real addresses
- Debug email templates
- Test transactional emails

**Enable:**
```bash
cp compose/docker-compose.override.yml-mailpit docker-compose.override.yml
docker compose up -d mailpit
```

**Access:**
- Web UI: `http://localhost:8025`
- SMTP: `127.0.0.1:1025`

**Configure in Laravel:**
```env
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_USERNAME=null
MAIL_PASSWORD=null
MAIL_ENCRYPTION=null
```

**Configure in WordPress:**
```php
// wp-config.php
define('SMTP_HOST', '127.0.0.1');
define('SMTP_PORT', 1025);
define('SMTP_AUTH', false);
```

**PHP Example:**
```php
$transport = (new Swift_SmtpTransport('127.0.0.1', 1025));
$mailer = new Swift_Mailer($transport);

$message = (new Swift_Message('Test Email'))
    ->setFrom(['dev@example.com' => 'Dev Team'])
    ->setTo(['user@example.com'])
    ->setBody('This is a test email!');

$mailer->send($message);
// Check http://localhost:8025 to see the email!
```

---

### 3. RabbitMQ - Message Queue

**What it is:** Reliable message broker for async tasks and microservices.

**Use cases:**
- Background job processing
- Async email sending
- Image/video processing queues
- Event-driven architectures
- Microservices communication

**Enable:**
```bash
cp compose/docker-compose.override.yml-rabbitmq docker-compose.override.yml
docker compose up -d rabbit
```

**Access:**
- Management UI: `http://localhost:15672`
- AMQP: `127.0.0.1:5672`
- Default credentials: `guest` / `guest`

**Configure in Laravel:**
```bash
composer require vladimir-yuldashev/laravel-queue-rabbitmq
```

```env
QUEUE_CONNECTION=rabbitmq
RABBITMQ_HOST=127.0.0.1
RABBITMQ_PORT=5672
RABBITMQ_USER=guest
RABBITMQ_PASSWORD=guest
RABBITMQ_VHOST=my_vhost
```

**PHP Example with php-amqplib:**
```bash
composer require php-amqplib/php-amqplib
```

```php
use PhpAmqpLib\Connection\AMQPStreamConnection;
use PhpAmqpLib\Message\AMQPMessage;

// Producer
$connection = new AMQPStreamConnection('127.0.0.1', 5672, 'guest', 'guest');
$channel = $connection->channel();
$channel->queue_declare('tasks', false, true, false, false);

$msg = new AMQPMessage('Process this image!');
$channel->basic_publish($msg, '', 'tasks');

echo " [x] Sent message\n";
$channel->close();
$connection->close();

// Consumer (worker)
$callback = function ($msg) {
    echo ' [x] Received ', $msg->body, "\n";
    // Process task...
    $msg->ack();
};

$channel->basic_consume('tasks', '', false, false, false, false, $callback);
while ($channel->is_consuming()) {
    $channel->wait();
}
```

---

### 4. MinIO - S3-Compatible Storage

**What it is:** High-performance object storage compatible with Amazon S3 API.

**Use cases:**
- File uploads and storage
- Media storage (images, videos)
- Backup storage
- S3-compatible development environment
- Document storage

**Enable:**
```bash
cp compose/docker-compose.override.yml-minio docker-compose.override.yml
docker compose up -d minio
```

**Access:**
- Console: `http://localhost:9001`
- API: `http://localhost:9000`
- Default credentials: `minioadmin` / `minioadmin`

**Configure in Laravel:**
```bash
composer require league/flysystem-aws-s3-v3 "^3.0" --with-all-dependencies
```

```env
# .env
AWS_ACCESS_KEY_ID=minioadmin
AWS_SECRET_ACCESS_KEY=minioadmin
AWS_DEFAULT_REGION=us-east-1
AWS_BUCKET=my-bucket
AWS_ENDPOINT=http://127.0.0.1:9000
AWS_USE_PATH_STYLE_ENDPOINT=true
```

```php
// config/filesystems.php
's3' => [
    'driver' => 's3',
    'key' => env('AWS_ACCESS_KEY_ID'),
    'secret' => env('AWS_SECRET_ACCESS_KEY'),
    'region' => env('AWS_DEFAULT_REGION'),
    'bucket' => env('AWS_BUCKET'),
    'endpoint' => env('AWS_ENDPOINT'),
    'use_path_style_endpoint' => env('AWS_USE_PATH_STYLE_ENDPOINT', false),
],
```

**PHP Example with AWS SDK:**
```bash
composer require aws/aws-sdk-php
```

```php
use Aws\S3\S3Client;

$client = new S3Client([
    'version' => 'latest',
    'region' => 'us-east-1',
    'endpoint' => 'http://127.0.0.1:9000',
    'use_path_style_endpoint' => true,
    'credentials' => [
        'key' => 'minioadmin',
        'secret' => 'minioadmin',
    ],
]);

// Create bucket
$client->createBucket(['Bucket' => 'my-bucket']);

// Upload file
$client->putObject([
    'Bucket' => 'my-bucket',
    'Key' => 'photos/vacation.jpg',
    'SourceFile' => '/path/to/file.jpg',
]);

// Get file URL
$url = $client->getObjectUrl('my-bucket', 'photos/vacation.jpg');
echo $url; // http://127.0.0.1:9000/my-bucket/photos/vacation.jpg
```

---

## 📦 Enable All Services

To enable all 4 modern services at once:

```bash
cp compose/docker-compose.override.yml-modern-services docker-compose.override.yml
docker compose up -d meilisearch mailpit rabbit minio
```

Or use the setup wizard:
```bash
./setup-devilbox.sh
# Answer "yes" when asked about modern services
```

---

## 🔧 Configuration

All services support environment variables in `.env`:

```env
# Meilisearch
MEILI_SERVER=latest
HOST_PORT_MEILI=7700
MEILI_MASTER_KEY=your-secret-key

# Mailpit
MAILPIT_SERVER=latest
HOST_PORT_MAILPIT=8025
HOST_PORT_MAILPIT_SMTP=1025
MP_MAX_MESSAGES=500

# RabbitMQ
RABBIT_SERVER=management
HOST_PORT_RABBIT=5672
HOST_PORT_RABBIT_MGMT=15672
RABBIT_DEFAULT_USER=guest
RABBIT_DEFAULT_PASS=guest
RABBIT_DEFAULT_VHOST=my_vhost

# MinIO
MINIO_SERVER=latest
HOST_PORT_MINIO=9000
HOST_PORT_MINIO_CONSOLE=9001
MINIO_ROOT_USER=minioadmin
MINIO_ROOT_PASSWORD=minioadmin
MINIO_REGION=us-east-1
```

---

## 🛑 Stopping Services

```bash
# Stop specific service
docker compose stop meilisearch

# Remove service (keeps data)
docker compose rm -f meilisearch

# Remove service AND data
docker compose down
docker volume rm devilbox-meilisearch
```

---

## 📊 Resource Usage

Approximate resource usage per service:

| Service | RAM | Disk | CPU |
|---------|-----|------|-----|
| Meilisearch | 100-500MB | 1GB | Low-Medium |
| Mailpit | 20-50MB | 10MB | Very Low |
| RabbitMQ | 50-200MB | 100MB | Low |
| MinIO | 50-100MB | Variable | Low |

**Total:** ~200-850MB RAM (varies with usage)

---

## 🐛 Troubleshooting

### Port Conflicts

If ports are already in use, change them in `.env`:

```env
HOST_PORT_MEILI=7701
HOST_PORT_MAILPIT=8026
HOST_PORT_RABBIT_MGMT=15673
HOST_PORT_MINIO_CONSOLE=9002
```

### Service Won't Start

```bash
# Check logs
docker compose logs meilisearch

# Restart service
docker compose restart meilisearch

# Rebuild
docker compose up -d --force-recreate meilisearch
```

### Data Persistence

All services use Docker volumes for data persistence:
- `devilbox-meilisearch`
- `devilbox-mailpit`
- `devilbox-rabbit`
- `devilbox-minio`

Data survives container restarts but not `docker compose down -v`.

---

## 🎓 Next Steps

1. **Learn More:**
   - [Meilisearch Docs](https://docs.meilisearch.com/)
   - [Mailpit GitHub](https://github.com/axllent/mailpit)
   - [RabbitMQ Tutorials](https://www.rabbitmq.com/getstarted.html)
   - [MinIO Docs](https://min.io/docs/minio/linux/index.html)

2. **Integration Examples:**
   - Check `.claude/skills/` for framework-specific guides
   - See `docs/Examples.md` for more code samples

3. **Production:**
   - Change default passwords!
   - Enable authentication
   - Use managed services for production

---

**Questions?** Open an issue on [GitHub](https://github.com/Drmzindec/Devilbox-Boost/issues).
