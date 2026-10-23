#!/usr/bin/env bash
# Create a new project inside the Devilbox Boost stack.
#
# usage: tools/new-project.sh <type> <name> [options]
#   --title "Site title"        WordPress/Bedrock/Drupal/Craft/static
#   --admin-user U --admin-pass P --admin-email E
#   --preset <name>             run tools/presets/<name>.sh (or ~/.config/devilbox-boost/presets/<name>.sh) afterwards
#   --list                      list project types
#   --list-presets              list presets (TYPE|NAME|DESCRIPTION)
#
# The project is built in data/www/.new-<name> (ignored by the vhost watcher) and
# renamed into place once its document root exists, so the vhost comes up ready.
set -e
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

# type|package|docroot|needs_db|description
RECIPES='laravel|laravel/laravel|public|yes|Laravel 13 (composer create-project laravel/laravel)
laravel-livewire|laravel/livewire-starter-kit|public|yes|Laravel 13 + Livewire starter kit (Volt, Flux, auth)
laravel-react|laravel/react-starter-kit|public|yes|Laravel 13 + React starter kit (Inertia 2, TypeScript)
laravel-vue|laravel/vue-starter-kit|public|yes|Laravel 13 + Vue starter kit (Inertia 2, TypeScript)
statamic|statamic/statamic|public|no|Statamic 6 (flat-file CMS on Laravel)
symfony|symfony/skeleton|public|no|Symfony 8 skeleton (micro)
symfony-web|symfony/skeleton|public|yes|Symfony 8 web app (skeleton + webapp pack, Doctrine)
wordpress|-|.|yes|WordPress (wp core download + install)
bedrock|roots/bedrock|web|yes|WordPress via Roots Bedrock (Composer-managed, .env config)
drupal|drupal/recommended-project|web|yes|Drupal 11 (recommended project + drush, installed)
craft|craftcms/craft|web|yes|Craft CMS 5 (installed via craft install)
cakephp|cakephp/app|webroot|yes|CakePHP 5 application skeleton
codeigniter|codeigniter4/appstarter|public|yes|CodeIgniter 4 app starter
typo3|typo3/cms-base-distribution|public|yes|TYPO3 13 LTS (web installer on first visit)
yii|yiisoft/yii2-app-basic|web|yes|Yii 2 basic application
nette|nette/web-project|www|no|Nette web project
slim|slim/slim-skeleton|public|no|Slim 4 skeleton
laminas|laminas/laminas-mvc-skeleton|public|no|Laminas MVC skeleton
static|-|.|no|Static HTML site'

preset_dirs() { echo "$ROOT/tools/presets"; echo "${XDG_CONFIG_HOME:-$HOME/.config}/devilbox-boost/presets"; }
list_presets() {
  for d in $(preset_dirs); do
    [ -d "$d" ] || continue
    for f in "$d"/*.sh; do
      [ -f "$f" ] || continue
      local t n desc; n=$(basename "$f" .sh)
      t=$(grep -m1 -E "^# *PRESET_TYPE:" "$f" | sed -E 's/^# *PRESET_TYPE: *//'); desc=$(grep -m1 -E "^# *PRESET_DESC:" "$f" | sed -E 's/^# *PRESET_DESC: *//')
      printf '%s|%s|%s\n' "${t:-any}" "$n" "$desc"
    done
  done
}
find_preset() { for d in $(preset_dirs); do [ -f "$d/$1.sh" ] && { echo "$d/$1.sh"; return 0; }; done; return 1; }

case "${1:-}" in
  --list) echo "$RECIPES" | awk -F'|' '{printf "%-18s %s\n", $1, $5}'; exit 0;;
  --list-presets) list_presets; exit 0;;
  ""|-h|--help) sed -n 2,13p "$0" | sed 's/^# \{0,1\}//'; echo; echo "Types:"; echo "$RECIPES" | awk -F'|' '{printf "  %-18s %s\n", $1, $5}'; exit 0;;
esac

TYPE="$1"; NAME="$2"; shift 2 || die "usage: new-project.sh <type> <name>"
TITLE=""; ADMIN_USER="admin"; ADMIN_PASS="admin"; ADMIN_EMAIL=""; PRESET=""
while [ $# -gt 0 ]; do
  case "$1" in
    --title) TITLE="$2"; shift 2;;
    --admin-user) ADMIN_USER="$2"; shift 2;;
    --admin-pass) ADMIN_PASS="$2"; shift 2;;
    --admin-email) ADMIN_EMAIL="$2"; shift 2;;
    --preset) PRESET="$2"; shift 2;;
    *) die "unknown option $1";;
  esac
done

recipe=$(echo "$RECIPES" | grep -E "^$TYPE\|" || true)
[ -n "$recipe" ] || die "unknown type '$TYPE' (see --list)"
IFS='|' read -r _ PACKAGE DOCROOT NEEDS_DB DESC <<<"$recipe"
[[ "$NAME" =~ ^[a-z0-9][a-z0-9-]{0,59}$ ]] && [[ "$NAME" != *- ]] || die "name must be lowercase letters, digits and dashes"
[ -e "$WWW/$NAME" ] && die "data/www/$NAME already exists"
if [ -n "$PRESET" ]; then PRESET_FILE=$(find_preset "$PRESET") || die "preset '$PRESET' not found"; fi

URL="$(project_url "$NAME")"; DB="$(db_name_for "$NAME")"; TITLE="${TITLE:-$NAME}"; ADMIN_EMAIL="${ADMIN_EMAIL:-admin@$NAME.$TLD}"
TMP=".new-$NAME"; TMPDIR_HOST="$WWW/$TMP"; TMPDIR_CT="/shared/httpd/$TMP"
rm -rf "$TMPDIR_HOST"
cleanup() { [ -d "$TMPDIR_HOST" ] && { fail "aborted, removing $TMP"; rm -rf "$TMPDIR_HOST"; }; }
trap cleanup ERR

log "Creating $DESC"
log "  name: $NAME   url: $URL   docroot: $DOCROOT$([ "$NEEDS_DB" = yes ] && echo "   db: $DB")"

set_env() { # file key value : replace KEY= or # KEY= line, else append
  local f="$1" k="$2" v="$3"
  if grep -qE "^#? ?$k=" "$f" 2>/dev/null; then
    awk -v k="$k" -v v="$v" 'BEGIN{done=0} { line=$0; sub(/^#? ?/,"",line); if(!done && index(line,k"=")==1){print k"="v; done=1} else print $0 }' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
  else
    printf '%s=%s\n' "$k" "$v" >> "$f"
  fi
}
dotenv_common() { # $1 = env file; sets APP_URL/mail/db for Laravel-style envs
  local f="$1"
  [ -f "$f" ] || { [ -f "$(dirname "$f")/.env.example" ] && cp "$(dirname "$f")/.env.example" "$f"; }
  [ -f "$f" ] || return 0
  set_env "$f" APP_URL "$URL"
  set_env "$f" MAIL_MAILER smtp; set_env "$f" MAIL_HOST 127.0.0.1; set_env "$f" MAIL_PORT 1025
  if [ "$NEEDS_DB" = yes ]; then
    set_env "$f" DB_CONNECTION mysql; set_env "$f" DB_HOST 127.0.0.1; set_env "$f" DB_PORT 3306
    set_env "$f" DB_DATABASE "$DB"; set_env "$f" DB_USERNAME root; set_env "$f" DB_PASSWORD "$MYSQL_PW"
  fi
}
composer_create() { step "composer create-project $PACKAGE"; php_exec /shared/httpd composer create-project "$PACKAGE" "$TMP" --no-interaction --prefer-dist --no-progress "$@"; }
npm_build() {
  [ -f "$TMPDIR_HOST/package.json" ] || return 0
  step "npm install && npm run build"
  php_exec "$TMPDIR_CT" sh -c 'npm install --no-audit --no-fund && { grep -q "\"build\"" package.json && npm run build || true; }' || warn "frontend build failed, continuing"
}
[ "$NEEDS_DB" = yes ] && { step "create database $DB"; ensure_db "$DB"; }

case "$TYPE" in
  laravel|laravel-livewire|laravel-react|laravel-vue|statamic)
    composer_create
    step "configure .env"; dotenv_common "$TMPDIR_HOST/.env"
    php_exec "$TMPDIR_CT" php artisan key:generate --force >/dev/null || true
    if [ "$NEEDS_DB" = yes ]; then step "php artisan migrate"; php_exec "$TMPDIR_CT" php artisan migrate --force; fi
    npm_build;;
  symfony|symfony-web)
    composer_create
    if [ "$TYPE" = symfony-web ]; then
      step "composer require webapp"; php_exec "$TMPDIR_CT" composer require webapp --no-interaction --no-progress
      step "configure DATABASE_URL"; echo "DATABASE_URL=\"mysql://root:$MYSQL_PW@127.0.0.1:3306/$DB?serverVersion=11.8.0-MariaDB&charset=utf8mb4\"" >> "$TMPDIR_HOST/.env.local"
    fi;;
  wordpress)
    mkdir -p "$TMPDIR_HOST"
    step "wp core download"; php_exec /shared/httpd wp core download --path="$TMP"
    step "wp config create"
    php_exec_stdin "$TMPDIR_CT" wp config create --dbname="$DB" --dbuser=root --dbpass="$MYSQL_PW" --dbhost=127.0.0.1 --skip-check --extra-php <<EOF
define('WP_HOME', '$URL');
define('WP_SITEURL', '$URL');
define('WP_DEBUG', true);
define('WP_DEBUG_DISPLAY', false);
define('WP_DEBUG_LOG', true);
define('WP_ENVIRONMENT_TYPE', 'local');
EOF
    step "wp core install"
    php_exec "$TMPDIR_CT" wp core install --url="$URL" --title="$TITLE" --admin_user="$ADMIN_USER" --admin_password="$ADMIN_PASS" --admin_email="$ADMIN_EMAIL" --skip-email
    php_exec "$TMPDIR_CT" wp rewrite structure '/%postname%/' --hard >/dev/null || true
    log "  admin: $URL/wp-login.php  ($ADMIN_USER)";;
  bedrock)
    composer_create
    step "configure .env"
    f="$TMPDIR_HOST/.env"; [ -f "$f" ] || cp "$TMPDIR_HOST/.env.example" "$f"
    set_env "$f" DB_NAME "$DB"; set_env "$f" DB_USER root; set_env "$f" DB_PASSWORD "$MYSQL_PW"; set_env "$f" DB_HOST 127.0.0.1
    set_env "$f" WP_ENV development; set_env "$f" WP_HOME "$URL"; set_env "$f" WP_SITEURL "$URL/wp"
    php_exec "$TMPDIR_CT" wp dotenv salts regenerate >/dev/null 2>&1 || php_exec "$TMPDIR_CT" sh -c 'command -v wp >/dev/null && wp package install aaemnnosttv/wp-cli-dotenv-command:^2.0 >/dev/null 2>&1 && wp dotenv salts regenerate' >/dev/null 2>&1 || warn "could not generate salts automatically; edit .env"
    step "wp core install"
    php_exec "$TMPDIR_CT" wp core install --url="$URL" --title="$TITLE" --admin_user="$ADMIN_USER" --admin_password="$ADMIN_PASS" --admin_email="$ADMIN_EMAIL" --skip-email
    log "  admin: $URL/wp/wp-login.php  ($ADMIN_USER)";;
  drupal)
    composer_create
    step "composer require drush/drush"; php_exec "$TMPDIR_CT" composer require drush/drush --no-interaction --no-progress
    step "drush site:install"
    php_exec "$TMPDIR_CT" vendor/bin/drush site:install standard --db-url="mysql://root:$MYSQL_PW@127.0.0.1:3306/$DB" --site-name="$TITLE" --account-name="$ADMIN_USER" --account-pass="$ADMIN_PASS" --account-mail="$ADMIN_EMAIL" -y
    log "  admin: $URL/user/login  ($ADMIN_USER)";;
  craft)
    composer_create
    step "configure .env"
    f="$TMPDIR_HOST/.env"; [ -f "$f" ] || cp "$TMPDIR_HOST/.env.example.dev" "$f" 2>/dev/null || true
    set_env "$f" CRAFT_DB_DRIVER mysql; set_env "$f" CRAFT_DB_SERVER 127.0.0.1; set_env "$f" CRAFT_DB_PORT 3306
    set_env "$f" CRAFT_DB_DATABASE "$DB"; set_env "$f" CRAFT_DB_USER root; set_env "$f" CRAFT_DB_PASSWORD "$MYSQL_PW"
    set_env "$f" PRIMARY_SITE_URL "$URL"; set_env "$f" CRAFT_ENVIRONMENT dev
    step "craft install"
    php_exec "$TMPDIR_CT" php craft setup/security-key >/dev/null
    php_exec "$TMPDIR_CT" php craft install --interactive=0 --email="$ADMIN_EMAIL" --username="$ADMIN_USER" --password="$ADMIN_PASS" --siteName="$TITLE" --siteUrl="$URL" --language=en-GB
    log "  admin: $URL/admin  ($ADMIN_USER)";;
  cakephp)
    composer_create
    step "configure config/app_local.php"
    f="$TMPDIR_HOST/config/app_local.php"
    [ -f "$f" ] || cp "$TMPDIR_HOST/config/app_local.example.php" "$f"
    sed -i.bak -E "s/'host' => 'localhost'/'host' => '127.0.0.1'/; s/'username' => 'my_app'/'username' => 'root'/; s/'password' => 'secret'/'password' => '$MYSQL_PW'/; s/'database' => 'my_app'/'database' => '$DB'/" "$f" && rm -f "$f.bak"
    step "bin/cake migrations migrate"; php_exec "$TMPDIR_CT" bin/cake migrations migrate 2>/dev/null || warn "no migrations yet";;
  codeigniter)
    composer_create
    step "configure .env"
    f="$TMPDIR_HOST/.env"; cp "$TMPDIR_HOST/env" "$f"
    set_env "$f" CI_ENVIRONMENT development
    set_env "$f" app.baseURL "'$URL/'"
    set_env "$f" database.default.hostname 127.0.0.1; set_env "$f" database.default.database "$DB"
    set_env "$f" database.default.username root; set_env "$f" database.default.password "$MYSQL_PW"; set_env "$f" database.default.DBDriver MySQLi;;
  typo3)
    composer_create
    step "prepare web installer"; touch "$TMPDIR_HOST/public/FIRST_INSTALL"
    log "  visit $URL to run the TYPO3 installer (db 127.0.0.1 / root / $MYSQL_PW / $DB)";;
  yii)
    composer_create
    step "configure config/db.php"
    sed -i.bak -E "s#'dsn' => '[^']*'#'dsn' => 'mysql:host=127.0.0.1;dbname=$DB'#; s#'username' => '[^']*'#'username' => 'root'#; s#'password' => '[^']*'#'password' => '$MYSQL_PW'#" "$TMPDIR_HOST/config/db.php" && rm -f "$TMPDIR_HOST/config/db.php.bak";;
  nette|slim|laminas)
    composer_create;;
  static)
    mkdir -p "$TMPDIR_HOST"
    cat > "$TMPDIR_HOST/index.html" <<EOF
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>$TITLE</title>
<style>body{font-family:system-ui,sans-serif;max-width:40rem;margin:4rem auto;padding:0 1rem;color:#1c2029}</style>
</head>
<body>
<h1>$TITLE</h1>
<p>Served by Devilbox Boost at <a href="$URL">$URL</a>.</p>
</body>
</html>
EOF
    ;;
esac

if [ -n "$PRESET" ]; then
  step "preset $PRESET"
  export PROJECT="$NAME" PROJECT_DIR="$TMPDIR_HOST" PROJECT_CT="$TMPDIR_CT" PROJECT_URL="$URL" PROJECT_DB="$DB" PROJECT_TYPE="$TYPE" ADMIN_USER ADMIN_PASS ADMIN_EMAIL
  # shellcheck disable=SC1090
  . "$PRESET_FILE"
  declare -F preset_run >/dev/null && preset_run
fi

step "htdocs -> $DOCROOT"
[ -e "$TMPDIR_HOST/htdocs" ] || ln -s "$DOCROOT" "$TMPDIR_HOST/htdocs"
mv "$TMPDIR_HOST" "$WWW/$NAME"
trap - ERR
ok "Project ready: $URL  (vhost appears within a few seconds)"
