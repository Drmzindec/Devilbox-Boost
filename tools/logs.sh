#!/usr/bin/env bash
# Tail the logs that matter for one project: web server access/error logs and
# the application log (wp-content/debug.log or storage/logs/laravel.log).
#
# usage: tools/logs.sh <project> [lines] [access|error|app|all]
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

P="${1:?usage: logs.sh <project> [lines] [access|error|app|all]}"; N="${2:-100}"; WHICH="${3:-all}"
D="$WWW/$P"; [ -d "$D" ] || die "project $P not found"
show() { # title file
  [ -f "$2" ] || return 0
  echo "### $1 ($2)"
  tail -n "$N" "$2"
  echo
}
want() { [ "$WHICH" = all ] || [ "$WHICH" = "$1" ]; }
if want error; then
  echo "### web server errors for $P.$TLD (docker compose logs httpd)"
  compose logs --no-color --tail 4000 httpd 2>/dev/null | grep -E "$P\.$TLD|/shared/httpd/$P/" | tail -n "$N"
  echo
  echo "### PHP-FPM errors for $P (docker compose logs php)"
  compose logs --no-color --tail 4000 php 2>/dev/null | grep -E "/shared/httpd/$P/" | tail -n "$N"
  echo
fi
if want access; then
  echo "### recent requests (docker compose logs httpd, all vhosts share one access log)"
  compose logs --no-color --tail 2000 httpd 2>/dev/null | grep -E '" [0-9]{3} ' | tail -n "$N"
  echo
fi
if want app; then
  show "wp-content/debug.log" "$D/wp-content/debug.log"
  show "storage/logs/laravel.log" "$D/storage/logs/laravel.log"
  show "var/log/dev.log" "$D/var/log/dev.log"
  show "logs/error.log" "$D/logs/error.log"
  show "storage/logs/web.log" "$D/storage/logs/web.log"
fi
exit 0
