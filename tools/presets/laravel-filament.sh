# PRESET_TYPE: laravel
# PRESET_DESC: Add Filament 5 admin panel (filament:install --panels) with a default admin user
preset_run() {
  php_exec "$PROJECT_CT" composer require filament/filament:"^5.0" --no-interaction --no-progress
  php_exec "$PROJECT_CT" php artisan filament:install --panels --no-interaction
  php_exec "$PROJECT_CT" php artisan migrate --force
  php_exec "$PROJECT_CT" php artisan make:filament-user --name="$ADMIN_USER" --email="$ADMIN_EMAIL" --password="$ADMIN_PASS" >/dev/null 2>&1 || true
  log "  Filament admin: $PROJECT_URL/admin  ($ADMIN_EMAIL / $ADMIN_PASS)"
}
