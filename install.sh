#!/usr/bin/env bash
set -e

BOOST_REPO="https://github.com/Drmzindec/Devilbox-Boost"
BOOST_BRANCH="main"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[Boost]${NC} $1"; }
ok()    { echo -e "${GREEN}[Boost]${NC} $1"; }
warn()  { echo -e "${YELLOW}[Boost]${NC} $1"; }
err()   { echo -e "${RED}[Boost]${NC} $1"; exit 1; }

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║        Devilbox Boost Installer          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""

# Check prerequisites
command -v git  >/dev/null 2>&1 || err "git is required but not installed."
command -v docker >/dev/null 2>&1 || err "docker is required but not installed."

# Detect if we're inside an existing devilbox directory
if [ -f "env-example" ] && [ -d ".devilbox" ]; then
    INSTALL_DIR="$(pwd)"
    info "Detected existing Devilbox at: ${INSTALL_DIR}"
    MODE="upgrade"
elif [ -f "docker-compose.yml" ] && grep -q "devilbox" docker-compose.yml 2>/dev/null; then
    INSTALL_DIR="$(pwd)"
    info "Detected Devilbox project at: ${INSTALL_DIR}"
    MODE="upgrade"
else
    INSTALL_DIR="$(pwd)/devilbox"
    MODE="fresh"
fi

# Fresh install: clone stock devilbox first
if [ "$MODE" = "fresh" ]; then
    if [ -d "$INSTALL_DIR" ]; then
        warn "Directory ${INSTALL_DIR} already exists."
        read -rp "Continue and overlay Boost into it? [y/N] " confirm
        [ "$confirm" = "y" ] || [ "$confirm" = "Y" ] || exit 0
    else
        info "Cloning stock Devilbox..."
        git clone https://github.com/cytopia/devilbox.git "$INSTALL_DIR"
    fi
    cd "$INSTALL_DIR"
fi

# Backup .env if it exists
if [ -f ".env" ]; then
    cp .env .env.pre-boost-backup
    ok "Backed up .env to .env.pre-boost-backup"
fi

# Download Boost into a temp directory
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

info "Downloading Devilbox Boost..."
git clone --depth 1 --branch "$BOOST_BRANCH" "$BOOST_REPO" "$TMPDIR/boost" 2>&1 | tail -1

# Overlay Boost files
info "Installing Boost enhancements..."

# Directories to overlay
for dir in docker-images bin compose mcp-server autostart supervisor cfg; do
    if [ -d "$TMPDIR/boost/$dir" ]; then
        cp -R "$TMPDIR/boost/$dir" .
    fi
done

# Dashboard
if [ -d "$TMPDIR/boost/.devilbox" ]; then
    cp -R "$TMPDIR/boost/.devilbox" .
fi

# Scripts
for script in setup-devilbox.sh shell.sh check-config.sh; do
    if [ -f "$TMPDIR/boost/$script" ]; then
        cp "$TMPDIR/boost/$script" .
        chmod +x "$script"
    fi
done

# Docker Compose files
for compose_file in docker-compose.yml docker-compose.lean.yml; do
    if [ -f "$TMPDIR/boost/$compose_file" ]; then
        cp "$TMPDIR/boost/$compose_file" .
    fi
done

# Documentation
for doc in README.md CHANGELOG.md CONTRIBUTING.md MIGRATION.md QUICKSTART.md SETUP-WIZARD.md; do
    if [ -f "$TMPDIR/boost/$doc" ]; then
        cp "$TMPDIR/boost/$doc" .
    fi
done

# env-example (don't overwrite .env)
if [ -f "$TMPDIR/boost/env-example" ]; then
    cp "$TMPDIR/boost/env-example" .
fi

# Create .env from env-example if it doesn't exist
if [ ! -f ".env" ]; then
    cp env-example .env
    ok "Created .env from env-example"
fi

# Restore backed-up .env
if [ -f ".env.pre-boost-backup" ]; then
    mv .env.pre-boost-backup .env
    ok "Restored your existing .env"
fi

ok "Boost files installed!"
echo ""

# Build PHP image
info "Building custom PHP image..."
if [ -f "./docker-images/build-php.sh" ]; then
    chmod +x ./docker-images/build-php.sh
    PHP_VER=$(grep '^PHP_SERVER=' .env 2>/dev/null | cut -d= -f2 | tr -d '"' | tr -d "'" || echo "8.4")
    PHP_VER=${PHP_VER:-8.4}
    info "Building PHP ${PHP_VER} image (this may take 5-10 minutes)..."
    ./docker-images/build-php.sh "$PHP_VER"
fi

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║      Devilbox Boost is installed! 🚀     ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""
echo -e "Next steps:"
echo -e "  ${CYAN}1.${NC} Run the setup wizard:  ${YELLOW}./setup-devilbox.sh${NC}"
echo -e "  ${CYAN}2.${NC} Start the stack:       ${YELLOW}docker compose up -d${NC}"
echo -e "  ${CYAN}3.${NC} Open the dashboard:    ${YELLOW}http://localhost${NC}"
echo ""
echo -e "  Enter the container:   ${YELLOW}./shell.sh${NC}"
echo -e "  Validate config:       ${YELLOW}./check-config.sh${NC}"
echo ""
