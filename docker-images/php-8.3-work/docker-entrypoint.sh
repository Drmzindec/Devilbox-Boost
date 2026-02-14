#!/bin/bash
set -e

# Setup port forwarding if FORWARD_PORTS_TO_LOCALHOST is set
if [ -n "$FORWARD_PORTS_TO_LOCALHOST" ]; then
    echo "[INFO] Setting up port forwarding..."

    # Split by comma
    IFS=',' read -ra FORWARDS <<< "$FORWARD_PORTS_TO_LOCALHOST"

    for forward in "${FORWARDS[@]}"; do
        # Parse: local_port:remote_host:remote_port
        IFS=':' read -ra PARTS <<< "$forward"
        LOCAL_PORT="${PARTS[0]}"
        REMOTE_HOST="${PARTS[1]}"
        REMOTE_PORT="${PARTS[2]}"

        echo "[INFO] Forwarding 127.0.0.1:${LOCAL_PORT} -> ${REMOTE_HOST}:${REMOTE_PORT}"

        # Start socat in background
        socat TCP-LISTEN:${LOCAL_PORT},bind=127.0.0.1,fork,reuseaddr TCP:${REMOTE_HOST}:${REMOTE_PORT} &
    done

    echo "[INFO] Port forwarding setup complete"
fi

# Execute the main command (PHP-FPM)
exec "$@"
