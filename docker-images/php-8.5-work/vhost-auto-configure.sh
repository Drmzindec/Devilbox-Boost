#!/bin/bash
# Vhost Auto-Configuration for Devilbox Boost
#
# Auto-creates htdocs symlinks so users can drop projects in data/www/
# without manual setup. The stock httpd watcherd handles actual vhost generation.
#
# Detection logic:
#   - Has public/  → htdocs -> public  (Laravel, Symfony, CakePHP)
#   - Has web/     → htdocs -> web     (Drupal 8+)
#   - Has pub/     → htdocs -> pub     (Magento 2)
#   - Otherwise    → htdocs -> .       (WordPress, generic)

HTTPD_DIR="/shared/httpd"
CHECK_INTERVAL=30

log() {
    echo "[vhost-auto-config] $1"
}

scan_projects() {
    log "Scanning for projects in $HTTPD_DIR..."

    [ -d "$HTTPD_DIR" ] || return

    for project_path in "$HTTPD_DIR"/*/; do
        [ -d "$project_path" ] || continue

        local project=$(basename "$project_path")
        [ "$project" = "." ] || [ "$project" = ".." ] && continue

        local desired="."
        if [ -d "$project_path/public" ]; then
            desired="public"
        elif [ -d "$project_path/web" ]; then
            desired="web"
        elif [ -d "$project_path/pub" ]; then
            desired="pub"
        fi

        if [ -L "$project_path/htdocs" ]; then
            local current
            current=$(readlink "$project_path/htdocs")
            [ "$current" = "$desired" ] && continue
            rm "$project_path/htdocs"
            log "$project/htdocs: $current -> $desired (corrected)"
        elif [ -e "$project_path/htdocs" ]; then
            # htdocs is a real directory, don't touch it
            continue
        fi

        ln -s "$desired" "$project_path/htdocs"
        [ -L "$project_path/htdocs" ] && log "$project/htdocs -> $desired"
    done
}

log "Starting vhost auto-configuration service..."
log "Checking every $CHECK_INTERVAL seconds for new projects"

while true; do
    scan_projects
    sleep $CHECK_INTERVAL
done
