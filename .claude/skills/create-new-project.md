# Create New Project

Create a new project in Devilbox with automatic vhost configuration.

## Laravel Project

```bash
# Navigate to www directory
cd data/www

# Create new Laravel project
docker-compose exec php composer create-project laravel/laravel my-project

# Wait for vhost auto-detection (30 seconds)
sleep 35

# Check vhost was created
ls -la my-project/.devilbox/

# Visit project
open http://my-project.local
```

## WordPress Project

```bash
cd data/www

# Download WordPress
docker-compose exec php wp core download --path=my-wordpress

# Configure
docker-compose exec php wp config create \
    --path=my-wordpress \
    --dbname=my_wordpress \
    --dbuser=root \
    --dbpass=root \
    --dbhost=127.0.0.1

# Create database
docker-compose exec php mysql -h 127.0.0.1 -u root -proot --skip-ssl \
    -e "CREATE DATABASE IF NOT EXISTS my_wordpress;"

# Install WordPress
docker-compose exec php wp core install \
    --path=my-wordpress \
    --url=http://my-wordpress.local \
    --title="My WordPress Site" \
    --admin_user=admin \
    --admin_password=admin \
    --admin_email=admin@example.com

# Visit
open http://my-wordpress.local
```

## Custom PHP Project

```bash
cd data/www

# Create directory
mkdir my-custom-project
cd my-custom-project

# Create index.php
cat > index.php <<'EOF'
<?php
phpinfo();
EOF

# Vhost auto-detection will handle it
# No DocumentRoot subdirectory needed for custom PHP

# Visit
open http://my-custom-project.local
```

## Manual Vhost Configuration

If you need custom DocumentRoot or server name:

```bash
cd data/www/my-project

# Create .devilbox directory
mkdir -p .devilbox

# Create Apache config
cat > .devilbox/apache24.yml <<'EOF'
vhost: |
  <VirtualHost *:80>
    ServerName my-project.local
    ServerAlias www.my-project.local

    DocumentRoot "/shared/httpd/my-project/public"

    <Directory "/shared/httpd/my-project/public">
      DirectoryIndex index.php index.html
      AllowOverride All
      Require all granted
    </Directory>

    # PHP-FPM
    <FilesMatch "\.php$">
      SetHandler "proxy:fcgi://172.16.238.10:9000"
    </FilesMatch>

    ErrorLog  "/var/log/apache2/my-project-error.log"
    CustomLog "/var/log/apache2/my-project-access.log" combined
  </VirtualHost>
EOF

# Create Nginx config
cat > .devilbox/nginx.yml <<'EOF'
vhost: |
  server {
    listen 80;
    server_name my-project.local www.my-project.local;

    root /shared/httpd/my-project/public;
    index index.php index.html;

    location / {
      try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
      fastcgi_pass 172.16.238.10:9000;
      fastcgi_index index.php;
      include fastcgi_params;
      fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
    }

    access_log /var/log/nginx/my-project-access.log;
    error_log /var/log/nginx/my-project-error.log;
  }
EOF

# Restart httpd to apply
docker-compose restart httpd
```

## DNS Configuration

### Option 1: Auto DNS (Recommended)

Devilbox includes a DNS server:

```bash
# Check DNS is running
docker-compose ps bind

# Configure your system DNS
# macOS: System Preferences → Network → Advanced → DNS
# Add: 127.0.0.1 (with port 1053 if needed)

# Or use /etc/hosts
sudo nano /etc/hosts
# Add: 127.0.0.1 my-project.local
```

### Option 2: Manual /etc/hosts

```bash
sudo nano /etc/hosts
```

Add entries:
```
127.0.0.1 my-project.local
127.0.0.1 my-wordpress.local
127.0.0.1 my-custom-project.local
```

## Troubleshooting

**404 Not Found:**
- Check DocumentRoot in vhost config
- Verify files exist in correct directory
- Check Apache/Nginx logs

**502 Bad Gateway:**
- Ensure PHP-FPM is running
- Check PHP-FPM address (172.16.238.10:9000)
- Verify docker network

**Database Connection Failed:**
- Use 127.0.0.1 for host (not localhost)
- Verify port forwarding is active
- Check credentials (root/root)

## Post-Creation Checklist

- [ ] Project files created
- [ ] Vhost config exists (.devilbox/ directory)
- [ ] Database created (if needed)
- [ ] DNS/hosts file configured
- [ ] Site loads in browser
- [ ] No errors in logs
- [ ] SSL certificate generated (if using HTTPS)
