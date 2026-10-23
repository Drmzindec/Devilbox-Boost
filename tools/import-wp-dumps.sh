#!/usr/bin/env bash
# Import production WordPress dumps into Devilbox-Boost and wire up each site.
#
# usage: tools/import-wp-dumps.sh <dump-dir> [project ...]
#        tools/import-wp-dumps.sh --file <dump.sql|.sql.gz|.zip> <project>
#
# Each dump file is matched to a project in data/www by its file stem:
#   shelterbox.sql | shelterbox.sql.gz | shelterbox.zip | shelterbox_2026-10-23.sql
# Put overrides in <dump-dir>/map.txt as "<file>=<project>" lines when a dump
# is named differently from its project directory.
#
# Per site: back up + drop the local DB if it has tables, create it, import,
# detect the table prefix + production siteurl from the dump, create or fix
# wp-config.php (DB creds, prefix, WP_HOME/WP_SITEURL -> https://<project>.loc),
# flush object cache, then probe https://<project>.loc.
set -u -o pipefail
export LC_ALL=C   # dumps may contain non-UTF-8 bytes; keep sed/grep byte-safe
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WWW="$ROOT/data/www"
SINGLE_FILE=""
if [ "${1:-}" = "--file" ]; then
  SINGLE_FILE="${2:?usage: $0 --file <dump> <project>}"; SINGLE_PROJECT="${3:?usage: $0 --file <dump> <project>}"
  DUMPS="$(cd "$(dirname "$SINGLE_FILE")" && pwd)"; ONLY=("$SINGLE_PROJECT")
else
  DUMPS="${1:?usage: $0 <dump-dir> [project ...]}"; shift || true
  ONLY=("$@")
fi
STAMP=$(date +%Y-%m-%d-%H%M)
BK="$ROOT/backups/$STAMP-pre-import"; mkdir -p "$BK"
REPORT="$ROOT/backups/$STAMP-import-report.txt"
cd "$ROOT"

sql()  { docker compose exec -T mysql mariadb -uroot -proot "$@"; }
wpc()  { local p=$1; shift; docker compose exec -T --user devilbox -w "/shared/httpd/$p" php wp --skip-plugins --skip-themes "$@"; }
reader(){ case "$1" in *.gz) gzip -dc "$1";; *.zip) unzip -p "$1" '*.sql';; *) cat "$1";; esac; }
want() { [ ${#ONLY[@]} -eq 0 ] && return 0; for o in "${ONLY[@]}"; do [ "$o" = "$1" ] && return 0; done; return 1; }

printf "%-28s %-22s %-8s %-40s %s\n" PROJECT DB PREFIX PROD_SITEURL RESULT | tee "$REPORT"
sql -e "SET GLOBAL max_allowed_packet=268435456" >/dev/null 2>&1

shopt -s nullglob
if [ -n "$SINGLE_FILE" ]; then FILES=("$SINGLE_FILE"); else FILES=("$DUMPS"/*.sql "$DUMPS"/*.sql.gz "$DUMPS"/*.zip); fi
for f in "${FILES[@]}"; do
  base=$(basename "$f")
  if [ -n "$SINGLE_FILE" ]; then proj="$SINGLE_PROJECT"; else
    proj=$(grep -E "^$base=" "$DUMPS/map.txt" 2>/dev/null | head -1 | cut -d= -f2-)
    [ -z "$proj" ] && proj=$(echo "$base" | sed -E 's/\.(sql\.gz|sql|zip)$//; s/[_-]?[0-9]{4}-?[0-9]{2}-?[0-9]{2}.*$//')
  fi
  want "$proj" || continue
  if [ ! -d "$WWW/$proj/wp-content" ]; then
    printf "%-28s %-22s %-8s %-40s %s\n" "$proj" - - - "SKIP: no WordPress project dir for $base" | tee -a "$REPORT"; continue
  fi
  db=${proj//-/_}
  if [ -f "$WWW/$proj/wp-config.php" ]; then   # keep an existing local DB name (e.g. eatoutthebox -> eotb)
    existing=$(grep -oE "'DB_NAME', *'[^']*'" "$WWW/$proj/wp-config.php" | head -1 | sed -E "s/.*'DB_NAME', *'//; s/'$//")
    [ -n "$existing" ] && db=$existing
  fi
  prefix=$(reader "$f" | grep -m1 -oE 'CREATE TABLE `[A-Za-z0-9_]*options`' | sed -E 's/CREATE TABLE `//; s/options`//')
  siteurl=$(reader "$f" | grep -m1 -oE "'siteurl', *'[^']*'" | sed -E "s/.*'siteurl', *'//; s/'$//")
  [ -z "$prefix" ] && { printf "%-28s %-22s %-8s %-40s %s\n" "$proj" "$db" - - "SKIP: no *options table in $base" | tee -a "$REPORT"; continue; }

  # backup + drop existing local DB if it has tables
  n=$(sql -N -e "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='$db'" 2>/dev/null || echo 0)
  if [ "${n:-0}" -gt 0 ]; then
    sql mariadb-dump --single-transaction --quick "$db" 2>/dev/null | gzip > "$BK/$db.sql.gz" || true
  fi
  sql -e "DROP DATABASE IF EXISTS \`$db\`; CREATE DATABASE \`$db\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci" || { printf "%-28s %s\n" "$proj" "FAIL: create db" | tee -a "$REPORT"; continue; }

  # import, ignoring any CREATE DATABASE/USE for the production db name
  if ! reader "$f" | sed -E '/^(CREATE DATABASE|USE `)/d; s/DEFAULT x AS `([0-9A-Fa-f]+)`/DEFAULT x'"'"'\1'"'"'/g' | sql --max-allowed-packet=256M "$db"; then
    printf "%-28s %-22s %-8s %-40s %s\n" "$proj" "$db" "$prefix" "$siteurl" "FAIL: import" | tee -a "$REPORT"; continue
  fi

  # wp-config.php
  local_url="https://$proj.loc"
  extra="define('WP_HOME', '$local_url');
define('WP_SITEURL', '$local_url');
define('WP_DEBUG', true);
define('WP_DEBUG_DISPLAY', false);
define('WP_DEBUG_LOG', true);
define('WP_ENVIRONMENT_TYPE', 'local');
define('DISALLOW_FILE_MODS', false);
define('AUTOMATIC_UPDATER_DISABLED', true);"
  if [ ! -f "$WWW/$proj/wp-config.php" ]; then
    wpc "$proj" config create --dbname="$db" --dbuser=root --dbpass=root --dbhost=127.0.0.1 --dbprefix="$prefix" --extra-php <<<"$extra" >/dev/null \
      || { printf "%-28s %-22s %-8s %-40s %s\n" "$proj" "$db" "$prefix" "$siteurl" "FAIL: wp config create" | tee -a "$REPORT"; continue; }
  else
    cp "$WWW/$proj/wp-config.php" "$BK/$proj.wp-config.php"
    wpc "$proj" config set DB_NAME "$db" --quiet
    wpc "$proj" config set DB_USER root --quiet
    wpc "$proj" config set DB_PASSWORD root --quiet
    wpc "$proj" config set DB_HOST 127.0.0.1 --quiet
    wpc "$proj" config set table_prefix "$prefix" --type=variable --quiet
    wpc "$proj" config set WP_HOME "$local_url" --quiet
    wpc "$proj" config set WP_SITEURL "$local_url" --quiet
  fi
  wpc "$proj" cache flush >/dev/null 2>&1 || true
  wpc "$proj" transient delete --all >/dev/null 2>&1 || true

  code=$(curl -sk -o /dev/null -m 90 -w '%{http_code}' "$local_url/")
  admin=$(curl -sk -o /dev/null -m 90 -w '%{http_code}' "$local_url/wp-login.php")
  printf "%-28s %-22s %-8s %-40s %s\n" "$proj" "$db" "$prefix" "${siteurl:--}" "home=$code login=$admin" | tee -a "$REPORT"
done
echo; echo "Report: $REPORT"; echo "Pre-import backups: $BK"
