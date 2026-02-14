#!/bin/bash
# Setup Laravel vhost configuration for a project
# Usage: ./setup-laravel-vhost.sh <project-name>

if [ -z "$1" ]; then
    echo "Usage: $0 <project-name>"
    echo "Example: $0 companychat"
    exit 1
fi

PROJECT="$1"
PROJECT_DIR="./data/www/$PROJECT"
VHOST_DIR="$PROJECT_DIR/.devilbox"

# Check if project exists
if [ ! -d "$PROJECT_DIR" ]; then
    echo "Error: Project directory not found: $PROJECT_DIR"
    exit 1
fi

# Check if it's a Laravel project
if [ ! -f "$PROJECT_DIR/artisan" ]; then
    echo "Warning: This doesn't appear to be a Laravel project (no artisan file found)"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Create .devilbox directory
mkdir -p "$VHOST_DIR"

# Create Apache vhost config
cat > "$VHOST_DIR/apache24.yml" << 'EOF'
vhost: |
  <VirtualHost __DEFAULT_VHOST__:__PORT__>
      ServerName   __VHOST_NAME__
      DocumentRoot "__DOCUMENT_ROOT__/public"

      <Directory "__DOCUMENT_ROOT__/public">
          DirectoryIndex index.php
          AllowOverride All
          Require all granted
      </Directory>

      # PHP-FPM
      <FilesMatch \.php$>
          SetHandler "proxy:fcgi://__PHP_ADDR__:__PHP_PORT__"
      </FilesMatch>

      CustomLog  "__ACCESS_LOG__" combined
      ErrorLog   "__ERROR_LOG__"
  </VirtualHost>
EOF

# Create Nginx vhost config
cat > "$VHOST_DIR/nginx.yml" << 'EOF'
vhost: |
  server {
      listen       __PORT____DEFAULT_VHOST__;
      server_name  __VHOST_NAME__;
      root         __DOCUMENT_ROOT__/public;
      index        index.php;

      location / {
          try_files $uri $uri/ /index.php?$query_string;
      }

      location ~ \.php$ {
          fastcgi_pass  __PHP_ADDR__:__PHP_PORT__;
          fastcgi_index index.php;
          fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
          include       fastcgi_params;
      }

      access_log "__ACCESS_LOG__" combined;
      error_log  "__ERROR_LOG__" warn;
  }
EOF

echo "✅ Laravel vhost configuration created for: $PROJECT"
echo ""
echo "The following files were created:"
echo "  - $VHOST_DIR/apache24.yml"
echo "  - $VHOST_DIR/nginx.yml"
echo ""
echo "Restart httpd to apply changes:"
echo "  docker-compose restart httpd"
echo ""
echo "Your Laravel app will now be accessible at:"
echo "  http://$PROJECT.local"
