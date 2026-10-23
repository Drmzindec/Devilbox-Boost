#!/usr/bin/env bash
# Shared helpers for the Devilbox Boost tools (also used by bin/dbox and the companion app).
# Source this file; it must not be executed.
#
#   ROOT, WWW, TLD, SCHEME, VHOST_PORT, UI_URL, MYSQL_PW
#   log/ok/warn/fail, die
#   php_exec <workdir> <cmd...>   run inside the PHP container as devilbox
#   wpc <project> <args...>       wp-cli for a project (plugins/themes skipped)
#   sql [db] -e "..."             mariadb/mysql client inside the db container
#   db_exists <db>, ensure_db <db>
#   project_kind <dir>            laravel|statamic|wordpress|symfony|cakephp|craft|yii|drupal|magento|codeigniter|typo3|nette|slim|laminas|next.js|static|php|other
#   project_db <dir>              database name from wp-config.php / .env (empty if none)
#   project_url <name>            https://name.tld[:port]
#   list_projects                 project names, one per line

set -o pipefail
export LC_ALL=C

ROOT="${DEVILBOX_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
WWW="$ROOT/data/www"
[ -f "$ROOT/docker-compose.yml" ] || { echo "Devilbox root not found at $ROOT (set DEVILBOX_ROOT)" >&2; exit 1; }

_env() { grep -E "^$1=" "$ROOT/.env" 2>/dev/null | head -1 | cut -d= -f2- | tr -d '"'; }
TLD="$(_env TLD_SUFFIX)"; TLD="${TLD:-loc}"
_ssl="$(_env HTTPD_VHOST_SSL_TYPE)"; _ssl="${_ssl:-plain}"
_hp="$(_env HOST_PORT_HTTPD)"; _hp="${_hp:-80}"
_sp="$(_env HOST_PORT_HTTPD_SSL)"; _sp="${_sp:-443}"
case "$_ssl" in
  ssl|both|redir) SCHEME=https; VHOST_PORT=""; [ "$_sp" = 443 ] || VHOST_PORT=":$_sp" ;;
  *)              SCHEME=http;  VHOST_PORT=""; [ "$_hp" = 80 ]  || VHOST_PORT=":$_hp" ;;
esac
UI_URL="$SCHEME://localhost$VHOST_PORT"
MYSQL_PW="$(_env MYSQL_ROOT_PASSWORD)"; MYSQL_PW="${MYSQL_PW:-root}"

if [ -t 1 ]; then C_OK=$'\e[32m'; C_WARN=$'\e[33m'; C_FAIL=$'\e[31m'; C_DIM=$'\e[2m'; C_RST=$'\e[0m'; else C_OK=""; C_WARN=""; C_FAIL=""; C_DIM=""; C_RST=""; fi
log()  { printf '%s\n' "$*"; }
ok()   { printf '%sOK%s    %s\n' "$C_OK" "$C_RST" "$*"; }
warn() { printf '%sWARN%s  %s\n' "$C_WARN" "$C_RST" "$*"; }
fail() { printf '%sFAIL%s  %s\n' "$C_FAIL" "$C_RST" "$*"; }
die()  { printf '%sError:%s %s\n' "$C_FAIL" "$C_RST" "$*" >&2; exit 1; }
step() { printf '%s── %s%s\n' "$C_DIM" "$*" "$C_RST"; }

compose() { (cd "$ROOT" && docker compose "$@"); }
php_exec() { local wd="$1"; shift; compose exec -T --user devilbox -w "$wd" php "$@" </dev/null; }
php_exec_stdin() { local wd="$1"; shift; compose exec -T --user devilbox -w "$wd" php "$@"; }
wpc() { local p="$1"; shift; php_exec "/shared/httpd/$p" wp --skip-plugins --skip-themes "$@"; }
sql() { compose exec -T mysql sh -c "command -v mariadb >/dev/null 2>&1 && exec mariadb -uroot -p'$MYSQL_PW' \"\$@\" || exec mysql -uroot -p'$MYSQL_PW' \"\$@\"" sh "$@"; }
db_exists() { [ "$(sql -N -e "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name='$1'" 2>/dev/null </dev/null | tr -d '[:space:]')" = 1 ]; }
ensure_db() { sql -e "CREATE DATABASE IF NOT EXISTS \`$1\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci" </dev/null; }
db_name_for() { echo "${1//-/_}"; }

project_kind() {
  local d="$1"
  if   [ -f "$d/please" ]; then echo statamic
  elif [ -f "$d/artisan" ]; then echo laravel
  elif [ -f "$d/wp-config.php" ] || [ -d "$d/wp-content" ] || [ -f "$d/wp-config-sample.php" ]; then echo wordpress
  elif [ -f "$d/web/wp-config.php" ] || [ -d "$d/web/app" ] && grep -q "roots/bedrock\|roots/wordpress" "$d/composer.json" 2>/dev/null; then echo bedrock
  elif [ -f "$d/craft" ]; then echo craft
  elif [ -f "$d/bin/console" ]; then echo symfony
  elif [ -f "$d/bin/cake" ] || [ -d "$d/webroot" ]; then echo cakephp
  elif [ -f "$d/yii" ]; then echo yii
  elif [ -f "$d/web/core/lib/Drupal.php" ] || [ -f "$d/core/lib/Drupal.php" ]; then echo drupal
  elif [ -f "$d/bin/magento" ]; then echo magento
  elif [ -f "$d/spark" ]; then echo codeigniter
  elif [ -f "$d/vendor/bin/typo3" ] || grep -q "typo3/cms" "$d/composer.json" 2>/dev/null; then echo typo3
  elif [ -d "$d/www" ] && [ -f "$d/app/Bootstrap.php" ]; then echo nette
  elif grep -q '"slim/slim"' "$d/composer.json" 2>/dev/null; then echo slim
  elif grep -q '"laminas/laminas-mvc"' "$d/composer.json" 2>/dev/null; then echo laminas
  elif [ -f "$d/next.config.mjs" ] || [ -f "$d/next.config.js" ] || [ -f "$d/next.config.ts" ]; then echo next.js
  elif [ -f "$d/index.html" ] || [ -f "$d/dist/index.html" ]; then echo static
  elif [ -f "$d/index.php" ] || [ -f "$d/public/index.php" ] || [ -f "$d/www/index.php" ]; then echo php
  else echo other; fi
}

project_db() {
  local d="$1" v=""
  if [ -f "$d/wp-config.php" ]; then
    v=$(grep -oE "'DB_NAME', *'[^']*'" "$d/wp-config.php" | head -1 | sed -E "s/.*'DB_NAME', *'//; s/'$//")
  elif [ -f "$d/.env" ]; then
    v=$(grep -E "^(DB_DATABASE|DB_NAME|CRAFT_DB_DATABASE|database\.default\.database)=" "$d/.env" | head -1 | cut -d= -f2- | tr -d '"'"'"'')
  fi
  echo "$v"
}

project_url() { echo "$SCHEME://$1.$TLD$VHOST_PORT"; }
list_projects() { find "$WWW" -mindepth 1 -maxdepth 1 -type d ! -name '.*' -exec basename {} \; | sort; }
is_composer_kind() { case "$1" in laravel|statamic|symfony|cakephp|craft|yii|drupal|magento|codeigniter|typo3|nette|slim|laminas|bedrock) return 0;; *) return 1;; esac; }
