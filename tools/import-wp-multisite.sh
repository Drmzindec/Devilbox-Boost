#!/usr/bin/env bash
# Import a WordPress MULTISITE dump and rewrite its network domains to local ones.
#
# usage: tools/import-wp-multisite.sh <project> <dump> <prefix> <site_id> <subdomain 0|1> <old=new> [<old=new> ...]
#   domain pairs are applied in the order given: put the most specific first
#   (core.shelterbox.org=core.shelterbox.loc before shelterbox.org=shelterbox.loc)
set -u -o pipefail
export LC_ALL=C   # dumps may contain non-UTF-8 bytes; keep sed/grep byte-safe
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; WWW="$ROOT/data/www"; cd "$ROOT"
proj=$1 dump=$2 prefix=$3 site_id=$4 subdomain=$5; shift 5
STAMP=$(date +%Y-%m-%d-%H%M); BK="$ROOT/backups/$STAMP-pre-import"; mkdir -p "$BK"
sql(){ docker compose exec -T mysql mariadb -uroot -proot "$@"; }
wpc(){ docker compose exec -T --user devilbox -w "/shared/httpd/$proj" php wp --skip-plugins --skip-themes --url="https://$proj.loc" "$@"; }
reader(){ case "$1" in *.gz) gzip -dc "$1";; *.zip) unzip -p "$1" '*.sql';; *) cat "$1";; esac; }
db=${proj//-/_}
[ -f "$WWW/$proj/wp-config.php" ] && { e=$(grep -oE "'DB_NAME', *'[^']*'" "$WWW/$proj/wp-config.php" | head -1 | sed -E "s/.*'DB_NAME', *'//; s/'$//"); [ -n "$e" ] && db=$e; }
echo "== $proj: db=$db prefix=$prefix site_id=$site_id subdomain=$subdomain"
n=$(sql -N -e "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='$db'" 2>/dev/null || echo 0)
[ "${n:-0}" -gt 0 ] && sql mariadb-dump --single-transaction --quick "$db" 2>/dev/null | gzip > "$BK/$db.sql.gz"
sql -e "SET GLOBAL max_allowed_packet=268435456; DROP DATABASE IF EXISTS \`$db\`; CREATE DATABASE \`$db\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci" || exit 1
echo "-- importing $(basename "$dump")"
reader "$dump" | sed -E '/^(CREATE DATABASE|USE `)/d; s/DEFAULT x AS `([0-9A-Fa-f]+)`/DEFAULT x'"'"'\1'"'"'/g' | sql --max-allowed-packet=256M "$db" || { echo "FAIL import"; exit 1; }
echo "-- fixing network tables"
for pair in "$@"; do o=${pair%%=*}; nw=${pair#*=}
  sql "$db" -e "UPDATE \`${prefix}site\` SET domain=REPLACE(domain,'$o','$nw'); UPDATE \`${prefix}blogs\` SET domain=REPLACE(domain,'$o','$nw');"
done
sql "$db" -e "SELECT blog_id, domain, path FROM \`${prefix}blogs\`; SELECT id, domain FROM \`${prefix}site\`"
echo "-- wp-config.php"
[ -f "$WWW/$proj/wp-config.php" ] && { cp "$WWW/$proj/wp-config.php" "$BK/$proj.wp-config.php"; rm "$WWW/$proj/wp-config.php"; }
sub=false; [ "$subdomain" = 1 ] && sub=true
wpc config create --dbname="$db" --dbuser=root --dbpass=root --dbhost=127.0.0.1 --dbprefix="$prefix" --skip-check --extra-php <<EOC
define('WP_ALLOW_MULTISITE', true);
define('MULTISITE', true);
define('SUBDOMAIN_INSTALL', $sub);
define('DOMAIN_CURRENT_SITE', '$proj.loc');
define('PATH_CURRENT_SITE', '/');
define('SITE_ID_CURRENT_SITE', $site_id);
define('BLOG_ID_CURRENT_SITE', 1);
define('WP_DEBUG', true);
define('WP_DEBUG_DISPLAY', false);
define('WP_DEBUG_LOG', true);
define('WP_ENVIRONMENT_TYPE', 'local');
define('AUTOMATIC_UPDATER_DISABLED', true);
EOC
echo "-- search-replace"
for pair in "$@"; do o=${pair%%=*}; nw=${pair#*=}
  wpc search-replace "$o" "$nw" --all-tables-with-prefix --skip-columns=guid --report-changed-only 2>&1 | tail -1
done
wpc cache flush >/dev/null 2>&1; wpc transient delete --all --network >/dev/null 2>&1
wpc site list --fields=blog_id,url 2>&1
echo "== done $proj"
