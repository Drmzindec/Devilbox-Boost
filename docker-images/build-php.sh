#!/usr/bin/env bash

set -e
set -u
set -o pipefail

# Build custom PHP images for Devilbox
# Usage: ./docker-images/build-php.sh [VERSION]
# Example: ./docker-images/build-php.sh 8.3

VERSION="${1:-8.3}"
IMAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/php-${VERSION}-work"

if [ ! -d "${IMAGE_DIR}" ]; then
    echo "Error: Directory ${IMAGE_DIR} does not exist"
    echo "Available versions:"
    ls -1d "$(dirname "${IMAGE_DIR}")"/php-*-work 2>/dev/null | xargs -n1 basename | sed 's/php-//;s/-work//' || echo "  (none)"
    exit 1
fi

echo "Building PHP ${VERSION} work image..."
echo "Image directory: ${IMAGE_DIR}"
echo ""

# Get host UID/GID
HOST_UID="$(id -u)"
HOST_GID="$(id -g)"

echo "Building with:"
echo "  UID: ${HOST_UID}"
echo "  GID: ${HOST_GID}"
echo ""

# Build the image
docker build \
    --build-arg NEW_UID="${HOST_UID}" \
    --build-arg NEW_GID="${HOST_GID}" \
    -t "devilbox-php-${VERSION}:work" \
    "${IMAGE_DIR}/"

echo ""
echo "✓ Successfully built devilbox-php-${VERSION}:work"
echo ""
echo "To use this image with Devilbox:"
echo "1. Set PHP_SERVER=${VERSION} in .env"
echo "2. Modify docker-compose.yml to use: image: devilbox-php-${VERSION}:work"
echo "   (or use docker-compose.override.yml)"
echo "3. Run: docker-compose up httpd php mysql"
