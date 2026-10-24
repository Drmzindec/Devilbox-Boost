#!/usr/bin/env bash
# Health check for the Devilbox Boost stack and every site in data/www.
#
# usage: tools/doctor.sh [--stack] [--sites] [project ...]
#
# Prints one finding per line:  LEVEL|SCOPE|MESSAGE   (LEVEL = OK, WARN or FAIL;
# SCOPE = "stack" or a project name). Human-readable colouring when on a TTY.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

DO_STACK=1; DO_SITES=1; ONLY=()
while [ $# -gt 0 ]; do
  case "$1" in
    --stack) DO_SITES=0; shift;;
    --sites) DO_STACK=0; shift;;
    -h|--help) sed -n 2,8p "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) ONLY+=("$1"); shift;;
  esac
done
[ ${#ONLY[@]} -gt 0 ] && DO_STACK=0

emit() { # level scope message
  if [ -t 1 ]; then case "$1" in OK) ok "[$2] $3";; WARN) warn "[$2] $3";; FAIL) fail "[$2] $3";; esac
  else printf '%s|%s|%s\n' "$1" "$2" "$3"; fi
}

probe() { curl -sk -o /dev/null -m 25 -w '%{http_code}' "$1" 2>/dev/null || echo 000; }

if [ "$DO_STACK" = 1 ]; then
  if docker info >/dev/null 2>&1; then emit OK stack "Docker is running"; else emit FAIL stack "Docker is not running or not reachable"; fi
  running=$(compose ps --status running --format '{{.Service}}' 2>/dev/null | wc -l | tr -d ' ')
  total=$(compose ps --all --format '{{.Service}}' 2>/dev/null | wc -l | tr -d ' ')
  if [ "$total" = 0 ]; then emit FAIL stack "No containers exist for this stack (docker compose up -d)"
  elif [ "$running" = "$total" ]; then emit OK stack "$running/$total containers running"
  else emit WARN stack "$running/$total containers running: $(compose ps --all --format '{{.Service}} {{.State}}' | grep -v running | tr '\n' ',' | sed 's/,$//')"; fi
  for s in $(compose ps --all --format '{{.Service}}|{{.Health}}' 2>/dev/null); do
    svc=${s%%|*}; h=${s#*|}; case "$h" in healthy|"") ;; *) emit WARN stack "$svc health: $h";; esac
  done
  if [ "$(uname)" != Darwin ]; then
    if getent hosts "doctor-probe.$TLD" >/dev/null 2>&1 || nslookup "doctor-probe.$TLD" 127.0.0.1 >/dev/null 2>&1; then emit OK stack "*.$TLD names resolve"
    else emit FAIL stack "*.$TLD does not resolve: point your resolver at the bind container (127.0.0.1:1053) or add entries to /etc/hosts"; fi
    if [ -f /etc/ssl/certs/devilbox-ca.pem ] || [ -f /usr/local/share/ca-certificates/devilbox-ca.crt ] || [ -f /etc/pki/ca-trust/source/anchors/devilbox-ca.crt ]; then emit OK stack "Devilbox CA installed system-wide"
    else emit WARN stack "Devilbox CA not installed: copy ca/devilbox-ca.crt to /usr/local/share/ca-certificates/ and run update-ca-certificates (Debian/Ubuntu)"; fi
  fi
  if [ "$(uname)" = Darwin ]; then
    if [ -f "/etc/resolver/$TLD" ]; then emit OK stack "/etc/resolver/$TLD present (.$TLD names resolve to the stack)"
    else emit FAIL stack "/etc/resolver/$TLD missing: run ./setup-devilbox.sh or create it (nameserver 127.0.0.1, port 1053)"; fi
    if security find-certificate -c "Devilbox" "$HOME/Library/Keychains/login.keychain-db" >/dev/null 2>&1 || security find-certificate -c "Devilbox" /Library/Keychains/System.keychain >/dev/null 2>&1; then
      emit OK stack "Devilbox CA is in the keychain"
    else emit WARN stack "Devilbox CA not found in the keychain: security add-trusted-cert -r trustRoot -k ~/Library/Keychains/login.keychain-db ca/devilbox-ca.crt"; fi
  fi
  if [ "$(probe "$UI_URL/")" = 200 ]; then emit OK stack "Intranet answers at $UI_URL"; else emit FAIL stack "Intranet not reachable at $UI_URL"; fi
  if [ -x "$ROOT/check-config.sh" ]; then
    if out=$("$ROOT/check-config.sh" 2>&1); then emit OK stack "check-config.sh passes"
    else emit WARN stack "check-config.sh: $(printf '%s\n' "$out" | sed -E $'s/\x1b\\[[0-9;]*m//g' | grep -E '\[ERR\]' | grep -vE 'Found [0-9]+ error|might not work' | head -1 | sed -E 's/^\[ERR\] *//' | cut -c1-160)"; fi
  fi
  if compose logs --since 10m --no-color php 2>/dev/null | grep -qE "exited on signal 11|SIGSEGV"; then emit FAIL stack "PHP-FPM workers segfaulted in the last 10 minutes (check opcache.jit in cfg/php-ini-*/custom.ini)"; fi
  du=$(docker system df --format '{{.Type}} {{.Reclaimable}}' 2>/dev/null | grep -E '^Images' | awk '{print $2}')
  [ -n "$du" ] && emit OK stack "Reclaimable image space: $du (docker image prune)"
fi

if [ "$DO_SITES" = 1 ]; then
  targets=()
  if [ ${#ONLY[@]} -gt 0 ]; then targets=("${ONLY[@]}"); else while IFS= read -r p; do targets+=("$p"); done < <(list_projects); fi
  for p in "${targets[@]}"; do
    d="$WWW/$p"; [ -d "$d" ] || { emit FAIL "$p" "not found"; continue; }
    k=$(project_kind "$d"); url=$(project_url "$p")
    [ -e "$d/htdocs" ] || emit WARN "$p" "no htdocs symlink yet (vhost-auto-configure creates it within 30s)"
    code=$(probe "$url/")
    case "$code" in
      200|301|302|303|307|308|401|403|410) emit OK "$p" "$k · $url → $code";;
      5*) err=$(tail -200 "$ROOT/log/nginx-stable/$p-error.log" 2>/dev/null | grep -m1 -oE "PHP message: [^\"]{0,160}" | tail -1); emit FAIL "$p" "$k · $url → $code ${err:+· $err}";;
      000) emit FAIL "$p" "$k · $url unreachable (DNS/vhost missing?)";;
      *) emit WARN "$p" "$k · $url → $code";;
    esac
    db=$(project_db "$d")
    if [ -n "$db" ]; then
      if db_exists "$db"; then
        n=$(sql -N -e "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='$db'" </dev/null 2>/dev/null | tr -d '[:space:]')
        [ "${n:-0}" = 0 ] && emit WARN "$p" "database $db exists but is empty"
      else emit FAIL "$p" "database $db is missing (config points to it)"; fi
    fi
    case "$k" in
      wordpress)
        [ -f "$d/wp-config.php" ] || emit WARN "$p" "no wp-config.php"
        # plugins active in the database but missing or incomplete on disk
        active=$(wpc "$p" option get active_plugins --format=json 2>/dev/null | grep -oE '"[^"]+\.php"' | tr -d '"' | sed 's|\\/|/|g')
        for pl in $active; do
          dir=${pl%%/*}
          if [ "$dir" = "$pl" ]; then [ -f "$d/wp-content/plugins/$pl" ] || emit FAIL "$p" "active plugin file missing: $pl"
          else [ -d "$d/wp-content/plugins/$dir" ] || emit FAIL "$p" "active plugin folder missing: $dir (premium/gitignored? copy it in)"
               [ -f "$d/wp-content/plugins/$pl" ] || { [ -d "$d/wp-content/plugins/$dir" ] && emit FAIL "$p" "plugin $dir is incomplete (main file $pl missing)"; }; fi
        done
        theme=$(wpc "$p" option get stylesheet 2>/dev/null | tr -d '[:space:]')
        [ -n "$theme" ] && [ ! -d "$d/wp-content/themes/$theme" ] && emit FAIL "$p" "active theme folder missing: $theme"
        if [ -f "$d/wp-content/debug.log" ]; then
          fat=$(grep -c "PHP Fatal error" "$d/wp-content/debug.log" 2>/dev/null || true)
          [ "$fat" -gt 0 ] && emit WARN "$p" "$fat fatal error line(s) in wp-content/debug.log: $(grep 'PHP Fatal error' "$d/wp-content/debug.log" | tail -1 | sed -E 's/^\[[^]]*\] //' | cut -c1-140)"
        fi
        if [ -f "$d/.gitignore" ] && grep -qxE "email|Email" "$d/.gitignore"; then emit WARN "$p" ".gitignore has a bare 'email' rule that hides plugin folders named Email on case-insensitive disks"; fi
        [ -d "$d/wp-content/uploads" ] || emit WARN "$p" "no wp-content/uploads folder (enable the production media fallback or copy uploads in)"
        ;;
      laravel|statamic)
        [ -f "$d/.env" ] || emit FAIL "$p" "no .env file"
        [ -d "$d/vendor" ] || emit FAIL "$p" "vendor/ missing (composer install)"
        [ -f "$d/package.json" ] && [ ! -f "$d/public/build/manifest.json" ] && [ ! -f "$d/public/hot" ] && emit WARN "$p" "no Vite build (npm run build) and no dev server running"
        if [ -f "$d/storage/logs/laravel.log" ]; then
          recent=$(grep -c "\.ERROR\|\.CRITICAL\|\.EMERGENCY" "$d/storage/logs/laravel.log" 2>/dev/null || true)
          [ "$recent" -gt 0 ] && emit WARN "$p" "$recent error line(s) in storage/logs/laravel.log"
        fi
        [ -f "$d/artisan" ] && { pend=$(php_exec "/shared/httpd/$p" php artisan migrate:status --pending 2>/dev/null | grep -c "Pending" || true); [ "${pend:-0}" -gt 0 ] && emit WARN "$p" "$pend pending migration(s)"; }
        ;;
    esac
  done
fi
