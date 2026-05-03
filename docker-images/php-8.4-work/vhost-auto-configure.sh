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

        # Skip if htdocs already exists
        [ -e "$project_path/htdocs" ] || [ -L "$project_path/htdocs" ] && continue

        if [ -d "$project_path/public" ]; then
            ln -s public "$project_path/htdocs"
            log "$project/htdocs -> public"
        elif [ -d "$project_path/web" ]; then
            ln -s web "$project_path/htdocs"
            log "$project/htdocs -> web"
        elif [ -d "$project_path/pub" ]; then
            ln -s pub "$project_path/htdocs"
            log "$project/htdocs -> pub"
        else
            ln -s . "$project_path/htdocs"
            log "$project/htdocs -> ."
        fi
    done
}

log "Starting vhost auto-configuration service..."
log "Checking every $CHECK_INTERVAL seconds for new projects"

while true; do
    scan_projects
    sleep $CHECK_INTERVAL
done
