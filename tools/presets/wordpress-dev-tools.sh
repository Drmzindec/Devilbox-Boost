# PRESET_TYPE: wordpress
# PRESET_DESC: Install and activate Query Monitor, WP Crontrol, Debug Bar and WP Mail logging; set permalinks
preset_run() {
  php_exec "$PROJECT_CT" wp plugin install query-monitor wp-crontrol debug-bar wp-mail-logging --activate
  php_exec "$PROJECT_CT" wp plugin delete hello akismet >/dev/null 2>&1 || true
  php_exec "$PROJECT_CT" wp option update blog_public 0
}
