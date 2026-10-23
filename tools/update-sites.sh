#!/usr/bin/env bash
# Check for or apply updates on the sites in data/www.
#
# usage: tools/update-sites.sh [--check] [--scope wordpress|php|all] [project ...]
#
#   --check   only report what is available. Machine-readable lines:
#               PLUGIN|<project>|<name>|<current>|<new>
#               THEME|<project>|<name>|<current>|<new>
#               CORE|<project>|<current>|<new>
#               PACKAGE|<project>|<name>|<current>|<new>
#               ERROR|<project>|<message>
#               DONE|<project>
#   (no flag) apply updates. WordPress: plugins, themes, core, database upgrade,
#             translations, cache flush. Composer projects: composer update,
#             artisan migrate / please updates:run, npm install + build.
#             Prints RESULT|<project>|ok|<message> or RESULT|<project>|fail|<message>.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

CHECK=0; SCOPE=all; ONLY=()
while [ $# -gt 0 ]; do
  case "$1" in
    --check) CHECK=1; shift;;
    --scope) SCOPE="$2"; shift 2;;
    -h|--help) sed -n 2,17p "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) ONLY+=("$1"); shift;;
  esac
done

in_scope() {
  case "$SCOPE" in
    wordpress) [ "$1" = wordpress ] || [ "$1" = bedrock ];;
    php) is_composer_kind "$1" && [ "$1" != bedrock ];;
    *) [ "$1" = wordpress ] || is_composer_kind "$1";;
  esac
}

wp_dir_for() { [ "$2" = bedrock ] && echo "/shared/httpd/$1/web/wp" || echo "/shared/httpd/$1"; }
wpp() { local p="$1" k="$2"; shift 2; php_exec "/shared/httpd/$p" wp --skip-plugins --skip-themes "$@"; }

json_rows() { # extract "name|version|update_version" rows from wp-cli json (python-free)
  tr -d '\n' | grep -oE '\{[^{}]*\}' | sed -E 's/.*"name":"([^"]*)".*"version":"([^"]*)".*"update_version":"([^"]*)".*/\1|\2|\3/' | grep -E '^[^|]+\|[^|]*\|[^|]*$'
}

check_one() {
  local p="$1" k="$2"
  case "$k" in
    wordpress|bedrock)
      local out
      out=$(wpp "$p" "$k" plugin list --update=available --fields=name,version,update_version --format=json 2>&1) || { echo "ERROR|$p|$(echo "$out" | grep -m1 -iE 'error' | cut -c1-200)"; echo "DONE|$p"; return; }
      echo "$out" | json_rows | sed "s/^/PLUGIN|$p|/"
      wpp "$p" "$k" theme list --update=available --fields=name,version,update_version --format=json 2>/dev/null | json_rows | sed "s/^/THEME|$p|/"
      local cur new
      cur=$(wpp "$p" "$k" core version 2>/dev/null | tr -d '[:space:]')
      new=$(wpp "$p" "$k" core check-update --field=version 2>/dev/null | head -1 | tr -d '[:space:]')
      if [[ "$new" =~ ^[0-9]+(\.[0-9]+)+$ ]] && [ "$new" != "$cur" ]; then echo "CORE|$p|$cur|$new"; fi
      ;;
    *)
      if is_composer_kind "$k"; then
        local out
        out=$(php_exec "/shared/httpd/$p" composer outdated --direct --format=json --no-interaction 2>/dev/null) || true
        echo "$out" | tr -d '\n' | sed -E 's/[[:space:]]+//g' | grep -oE '\{[^{}]*"name":"[^"]*"[^{}]*\}' | while IFS= read -r obj; do
          n=$(printf '%s' "$obj" | sed -E 's/.*"name":"([^"]*)".*/\1/'); v=$(printf '%s' "$obj" | sed -E 's/.*"version":"([^"]*)".*/\1/'); l=$(printf '%s' "$obj" | sed -E 's/.*"latest":"([^"]*)".*/\1/')
          [ -n "$n" ] && [ "$n" != "$obj" ] && [ "$l" != "$obj" ] && echo "PACKAGE|$p|$n|$v|$l"
        done
        [ -z "$out" ] && echo "ERROR|$p|composer outdated produced no output (is the stack running and vendor/ installed?)"
      fi;;
  esac
  echo "DONE|$p"
}

update_one() {
  local p="$1" k="$2" wd="/shared/httpd/$p" d="$WWW/$p" failed=0
  run() { local title="$1"; shift; step "$title"; if ! "$@"; then warn "$title reported a problem"; return 1; fi; }
  case "$k" in
    wordpress|bedrock)
      run "plugins"      wpp "$p" "$k" plugin update --all || true
      run "themes"       wpp "$p" "$k" theme update --all || true
      run "core"         wpp "$p" "$k" core update || true
      if grep -q "'MULTISITE', *true" "$d/wp-config.php" 2>/dev/null; then run "network database upgrade" wpp "$p" "$k" core update-db --network || true
      else run "database upgrade" wpp "$p" "$k" core update-db || true; fi
      run "translations" wpp "$p" "$k" language core update || true
      run "cache flush"  wpp "$p" "$k" cache flush || true
      ;;
    *)
      run "composer update" php_exec "$wd" composer update --no-interaction --no-progress --prefer-dist || failed=1
      [ -f "$d/artisan" ] && { run "php artisan migrate" php_exec "$wd" php artisan migrate --force || true; }
      [ -f "$d/please" ]  && { run "php please updates:run" php_exec "$wd" php please updates:run || true; }
      [ -f "$d/vendor/bin/drush" ] && { run "drush updatedb" php_exec "$wd" vendor/bin/drush updatedb -y || true; }
      [ -f "$d/craft" ] && { run "craft migrate/all" php_exec "$wd" php craft migrate/all --interactive=0 || true; }
      if [ -f "$d/package.json" ]; then
        run "npm install" php_exec "$wd" npm install --no-audit --no-fund || true
        grep -q '"build"' "$d/package.json" && { run "npm run build" php_exec "$wd" npm run build || true; }
      fi
      ;;
  esac
  [ "$failed" = 0 ] && echo "RESULT|$p|ok|updated" || echo "RESULT|$p|fail|composer update failed"
}

targets=()
if [ ${#ONLY[@]} -gt 0 ]; then targets=("${ONLY[@]}"); else while IFS= read -r p; do targets+=("$p"); done < <(list_projects); fi
for p in "${targets[@]}"; do
  [ -d "$WWW/$p" ] || { echo "ERROR|$p|not found"; continue; }
  k=$(project_kind "$WWW/$p")
  if [ ${#ONLY[@]} -eq 0 ]; then in_scope "$k" || continue; fi
  if [ "$CHECK" = 1 ]; then check_one "$p" "$k"; else log "══ $p ($k)"; update_one "$p" "$k"; fi
done
