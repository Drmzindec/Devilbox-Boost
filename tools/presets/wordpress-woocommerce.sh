# PRESET_TYPE: wordpress
# PRESET_DESC: Install and activate WooCommerce with the Storefront theme
preset_run() {
  php_exec "$PROJECT_CT" wp plugin install woocommerce --activate
  php_exec "$PROJECT_CT" wp theme install storefront --activate
}
