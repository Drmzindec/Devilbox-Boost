# PRESET_TYPE: laravel
# PRESET_DESC: Developer tooling: Pest, Laravel Pint, Larastan and Laravel Boost (AI MCP) as dev dependencies
preset_run() {
  php_exec "$PROJECT_CT" composer require --dev pestphp/pest pestphp/pest-plugin-laravel laravel/pint larastan/larastan laravel/boost --no-interaction --no-progress
  php_exec "$PROJECT_CT" php artisan boost:install --no-interaction >/dev/null 2>&1 || true
}
