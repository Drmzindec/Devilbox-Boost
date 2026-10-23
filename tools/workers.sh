#!/usr/bin/env bash
# Long-running helpers per project, run inside the PHP container.
#
# usage: tools/workers.sh <project> status
#        tools/workers.sh <project> start|stop|restart <queue|schedule|horizon|reverb>
#        tools/workers.sh <project> logs <worker> [lines]
#
# status prints WORKER|<name>|running|<pid>  or  WORKER|<name>|stopped|
# Processes are tagged with "dbx-worker:<project>:<name>" so they can be found
# and stopped reliably. Output goes to storage/logs/worker-<name>.log in the project.
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

P="${1:?usage: workers.sh <project> <status|start|stop|restart|logs> [worker]}"; ACTION="${2:-status}"; W="${3:-}"
D="$WWW/$P"; WD="/shared/httpd/$P"
[ -d "$D" ] || die "project $P not found"

cmd_for() {
  case "$1" in
    queue)    echo "php artisan queue:work --tries=3 --timeout=120 --sleep=3";;
    schedule) echo "php artisan schedule:work";;
    horizon)  echo "php artisan horizon";;
    reverb)   echo "php artisan reverb:start";;
    *) die "unknown worker $1 (queue, schedule, horizon, reverb)";;
  esac
}
tag() { echo "dbx-worker:$P:$1"; }
# The PHP image has no procps; scan /proc for the tag instead.
# Workers are started with `exec -a <tag>`, so argv[0] (the start of /proc/<pid>/cmdline) is the tag.
pids_of() { compose exec -T php sh -c 'n=${#1}; for d in /proc/[0-9]*; do [ "$(head -c "$n" "$d/cmdline" 2>/dev/null)" = "$1" ] && echo "${d#/proc/}"; done' sh "$(tag "$1")" 2>/dev/null </dev/null | tr -d '\r'; }
pid_of() { pids_of "$1" | head -1 | tr -d '[:space:]'; }

case "$ACTION" in
  status)
    for w in queue schedule horizon reverb; do
      pid=$(pid_of "$w"); if [ -n "$pid" ]; then echo "WORKER|$w|running|$pid"; else echo "WORKER|$w|stopped|"; fi
    done;;
  start)
    [ -n "$W" ] || die "which worker?"
    [ -f "$D/artisan" ] || die "$P is not a Laravel project"
    [ "$W" = horizon ] && [ ! -d "$D/vendor/laravel/horizon" ] && die "laravel/horizon is not installed in $P"
    pid=$(pid_of "$W"); [ -n "$pid" ] && { echo "WORKER|$W|running|$pid"; exit 0; }
    mkdir -p "$D/storage/logs"
    # bash -c with the tag as $0 keeps the marker in the process list; setsid detaches from the exec.
    compose exec -T -d --user devilbox -w "$WD" php bash -c "exec -a '$(tag "$W")' bash -c \"$(cmd_for "$W") >> storage/logs/worker-$W.log 2>&1\"" </dev/null
    sleep 1; pid=$(pid_of "$W")
    [ -n "$pid" ] && echo "WORKER|$W|running|$pid" || { echo "WORKER|$W|stopped|"; tail -5 "$D/storage/logs/worker-$W.log" 2>/dev/null; exit 1; };;
  stop)
    [ -n "$W" ] || die "which worker?"
    pids=$(pids_of "$W" | tr '\n' ' ')
    [ -n "$pids" ] && compose exec -T php sh -c "kill -TERM $pids 2>/dev/null; sleep 1; kill -KILL $pids 2>/dev/null" </dev/null || true
    echo "WORKER|$W|stopped|";;
  restart)
    "$0" "$P" stop "$W" >/dev/null; "$0" "$P" start "$W";;
  logs)
    tail -n "${4:-100}" "$D/storage/logs/worker-$W.log" 2>/dev/null || echo "(no log yet)";;
  *) die "unknown action $ACTION";;
esac
