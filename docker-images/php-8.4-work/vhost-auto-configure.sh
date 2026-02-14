#!/bin/bash
# Smart Vhost Auto-Configuration for Devilbox
# Automatically detects framework type and configures vhost docroot

HTTPD_DIR="/shared/httpd"
CHECK_INTERVAL=30  # Check every 30 seconds

log() {
    echo "[vhost-auto-config] $1"
}

detect_framework() {
    local project_dir="$1"

    # Laravel - has artisan file
    if [ -f "$project_dir/artisan" ]; then
        echo "laravel"
        return 0
    fi

    # WordPress - has wp-config.php or wp-config-sample.php
    if [ -f "$project_dir/wp-config.php" ] || [ -f "$project_dir/wp-config-sample.php" ]; then
        echo "wordpress"
        return 0
    fi

    # Symfony - has symfony.lock or bin/console
    if [ -f "$project_dir/symfony.lock" ] || [ -f "$project_dir/bin/console" ]; then
        echo "symfony"
        return 0
    fi

    # CodeIgniter - has system/core/CodeIgniter.php
    if [ -f "$project_dir/system/core/CodeIgniter.php" ]; then
        echo "codeigniter"
        return 0
    fi

    # Yii - has yii file
    if [ -f "$project_dir/yii" ]; then
        echo "yii"
        return 0
    fi

    # CakePHP - has bin/cake
    if [ -f "$project_dir/bin/cake" ]; then
        echo "cakephp"
        return 0
    fi

    # Generic PHP project
    echo "generic"
    return 0
}

get_docroot() {
    local framework="$1"

    case "$framework" in
        laravel|symfony|cakephp|yii)
            echo "public"
            ;;
        wordpress|codeigniter|generic)
            echo ""
            ;;
        *)
            echo ""
            ;;
    esac
}

create_vhost_config() {
    local project="$1"
    local framework="$2"
    local docroot="$3"
    local vhost_dir="$HTTPD_DIR/$project/.devilbox"

    # Create .devilbox directory if it doesn't exist
    mkdir -p "$vhost_dir"

    # Determine docroot path
    if [ -n "$docroot" ]; then
        local docroot_path="__DOCUMENT_ROOT__/$docroot"
    else
        local docroot_path="__DOCUMENT_ROOT__"
    fi

    # Create Apache config if it doesn't exist
    if [ ! -f "$vhost_dir/apache24.yml" ]; then
        log "Creating Apache vhost config for $project ($framework)"
        cat > "$vhost_dir/apache24.yml" << EOF
vhost: |
  <VirtualHost __DEFAULT_VHOST__:__PORT__>
      ServerName   __VHOST_NAME__
      DocumentRoot "$docroot_path"

      <Directory "$docroot_path">
          DirectoryIndex index.php index.html index.htm
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
    fi

    # Create Nginx config if it doesn't exist
    if [ ! -f "$vhost_dir/nginx.yml" ]; then
        log "Creating Nginx vhost config for $project ($framework)"

        # Different Nginx configs based on framework
        if [ "$framework" = "laravel" ] || [ "$framework" = "symfony" ]; then
            # Laravel/Symfony style - try_files for routing
            cat > "$vhost_dir/nginx.yml" << EOF
vhost: |
  server {
      listen       __PORT____DEFAULT_VHOST__;
      server_name  __VHOST_NAME__;
      root         $docroot_path;
      index        index.php index.html index.htm;

      location / {
          try_files \$uri \$uri/ /index.php?\$query_string;
      }

      location ~ \.php$ {
          fastcgi_pass  __PHP_ADDR__:__PHP_PORT__;
          fastcgi_index index.php;
          fastcgi_param SCRIPT_FILENAME \$document_root\$fastcgi_script_name;
          include       fastcgi_params;
      }

      access_log "__ACCESS_LOG__" combined;
      error_log  "__ERROR_LOG__" warn;
  }
EOF
        elif [ "$framework" = "wordpress" ]; then
            # WordPress style - specific permalink handling
            cat > "$vhost_dir/nginx.yml" << EOF
vhost: |
  server {
      listen       __PORT____DEFAULT_VHOST__;
      server_name  __VHOST_NAME__;
      root         $docroot_path;
      index        index.php;

      location / {
          try_files \$uri \$uri/ /index.php?\$args;
      }

      location ~ \.php$ {
          fastcgi_pass  __PHP_ADDR__:__PHP_PORT__;
          fastcgi_index index.php;
          fastcgi_param SCRIPT_FILENAME \$document_root\$fastcgi_script_name;
          include       fastcgi_params;
      }

      access_log "__ACCESS_LOG__" combined;
      error_log  "__ERROR_LOG__" warn;
  }
EOF
        else
            # Generic PHP
            cat > "$vhost_dir/nginx.yml" << EOF
vhost: |
  server {
      listen       __PORT____DEFAULT_VHOST__;
      server_name  __VHOST_NAME__;
      root         $docroot_path;
      index        index.php index.html index.htm;

      location ~ \.php$ {
          fastcgi_pass  __PHP_ADDR__:__PHP_PORT__;
          fastcgi_index index.php;
          fastcgi_param SCRIPT_FILENAME \$document_root\$fastcgi_script_name;
          include       fastcgi_params;
      }

      access_log "__ACCESS_LOG__" combined;
      error_log  "__ERROR_LOG__" warn;
  }
EOF
        fi
    fi
}

scan_projects() {
    log "Scanning for projects in $HTTPD_DIR..."

    # Skip if httpd directory doesn't exist yet
    if [ ! -d "$HTTPD_DIR" ]; then
        return
    fi

    # Scan each directory in /shared/httpd
    for project_path in "$HTTPD_DIR"/*; do
        # Skip if not a directory
        if [ ! -d "$project_path" ]; then
            continue
        fi

        # Get project name
        local project=$(basename "$project_path")

        # Skip special directories
        if [ "$project" = "." ] || [ "$project" = ".." ]; then
            continue
        fi

        # Skip if vhost config already exists
        if [ -f "$project_path/.devilbox/apache24.yml" ] && [ -f "$project_path/.devilbox/nginx.yml" ]; then
            continue
        fi

        # Detect framework
        local framework=$(detect_framework "$project_path")
        local docroot=$(get_docroot "$framework")

        # Create vhost config
        create_vhost_config "$project" "$framework" "$docroot"

        log "✅ Configured $project as $framework project"
    done
}

# Main loop
log "Starting vhost auto-configuration service..."
log "Checking every $CHECK_INTERVAL seconds for new projects"

while true; do
    scan_projects
    sleep $CHECK_INTERVAL
done
