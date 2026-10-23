#!/usr/bin/env bash
# Database snapshots per project (or raw database name).
#
# usage: tools/snapshot.sh <project|db> create [label]
#        tools/snapshot.sh <project|db> list              -> FILE|SIZE_BYTES|EPOCH
#        tools/snapshot.sh <project|db> restore <file>    (takes a safety snapshot first)
#        tools/snapshot.sh <project|db> delete <file>
#        tools/snapshot.sh <project|db> prune <keep>      (keep the newest N)
#
# Snapshots live in backups/snapshots/<db>/<timestamp>[-label].sql.gz (gitignored).
. "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

TARGET="${1:?usage: snapshot.sh <project|db> <create|list|restore|delete|prune> ...}"; ACTION="${2:-list}"; ARG="${3:-}"
if [ -d "$WWW/$TARGET" ]; then DB=$(project_db "$WWW/$TARGET"); DB="${DB:-$(db_name_for "$TARGET")}"; else DB="$TARGET"; fi
DIR="$ROOT/backups/snapshots/$DB"; mkdir -p "$DIR"

dump() { compose exec -T mysql sh -c "command -v mariadb-dump >/dev/null 2>&1 && exec mariadb-dump -uroot -p'$MYSQL_PW' --single-transaction --quick --routines --triggers '$DB' || exec mysqldump -uroot -p'$MYSQL_PW' --single-transaction --quick --routines --triggers '$DB'" </dev/null; }

case "$ACTION" in
  create)
    db_exists "$DB" || die "database $DB does not exist"
    label=""; [ -n "$ARG" ] && label="-$(printf '%s' "$ARG" | tr -c 'a-zA-Z0-9_-' '_')"
    f="$DIR/$(date +%Y-%m-%d-%H%M%S)$label.sql.gz"
    dump | gzip > "$f" || { rm -f "$f"; die "dump failed"; }
    echo "SNAPSHOT|$f|$(stat -f %z "$f" 2>/dev/null || stat -c %s "$f")"
    ;;
  list)
    for f in "$DIR"/*.sql.gz; do [ -f "$f" ] || continue; printf '%s|%s|%s\n' "$f" "$(stat -f %z "$f" 2>/dev/null || stat -c %s "$f")" "$(stat -f %m "$f" 2>/dev/null || stat -c %Y "$f")"; done | sort -t'|' -k3 -rn
    ;;
  restore)
    [ -f "$ARG" ] || die "snapshot file not found: $ARG"
    case "$(cd "$(dirname "$ARG")" && pwd)" in "$DIR") ;; *) die "refusing to restore a file from outside $DIR";; esac
    if db_exists "$DB"; then
      safety="$DIR/$(date +%Y-%m-%d-%H%M%S)-before-restore.sql.gz"; dump | gzip > "$safety" && echo "SAFETY|$safety"
    fi
    sql -e "DROP DATABASE IF EXISTS \`$DB\`; CREATE DATABASE \`$DB\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci" </dev/null
    gzip -dc "$ARG" | sql --max-allowed-packet=256M "$DB" || die "restore failed"
    echo "RESTORED|$ARG"
    ;;
  delete)
    case "$(cd "$(dirname "$ARG")" 2>/dev/null && pwd)" in "$DIR") rm -f "$ARG" && echo "DELETED|$ARG";; *) die "refusing to delete a file outside $DIR";; esac
    ;;
  prune)
    keep="${ARG:-5}"; i=0
    for f in $(ls -t "$DIR"/*.sql.gz 2>/dev/null); do i=$((i+1)); [ "$i" -le "$keep" ] || { rm -f "$f"; echo "DELETED|$f"; }; done
    ;;
  *) die "unknown action $ACTION";;
esac
