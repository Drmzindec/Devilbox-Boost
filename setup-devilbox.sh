#!/usr/bin/env bash
set -e

###############################################################################
# Devilbox Modern Setup Wizard
# Interactive TUI for configuring and starting Devilbox
###############################################################################

# Colors
RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
BLUE=$'\033[0;34m'
MAGENTA=$'\033[0;35m'
CYAN=$'\033[0;36m'
NC=$'\033[0m' # No Color
BOLD=$'\033[1m'

# Emojis
ROCKET="🚀"
CHECK="✅"
CROSS="❌"
GEAR="⚙️"
PACKAGE="📦"
BOOK="📚"
WRENCH="🔧"
SPARKLES="✨"
FIRE="🔥"

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

###############################################################################
# Helper Functions
###############################################################################

print_header() {
    echo ""
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}${BOLD}$1${NC}"
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_section() {
    echo ""
    echo -e "${MAGENTA}${BOLD}▸ $1${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}${CHECK} $1${NC}"
}

print_error() {
    echo -e "${RED}${CROSS} $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

ask_question() {
    local question="$1"
    local default="$2"
    local var_name="$3"

    if [ -n "$default" ]; then
        echo -ne "${CYAN}${question} ${NC}${BOLD}[${default}]${NC}: "
    else
        echo -ne "${CYAN}${question}${NC}: "
    fi

    read -r response

    if [ -z "$response" ] && [ -n "$default" ]; then
        eval "$var_name='$default'"
    else
        eval "$var_name='$response'"
    fi
}

ask_yes_no() {
    local question="$1"
    local default="$2"  # y or n

    if [ "$default" = "y" ]; then
        echo -ne "${CYAN}${question} ${NC}${BOLD}[Y/n]${NC}: "
    else
        echo -ne "${CYAN}${question} ${NC}${BOLD}[y/N]${NC}: "
    fi

    read -r response
    response=$(echo "$response" | tr '[:upper:]' '[:lower:]')

    if [ -z "$response" ]; then
        [ "$default" = "y" ] && return 0 || return 1
    fi

    [ "$response" = "y" ] || [ "$response" = "yes" ]
}

###############################################################################
# Welcome Screen
###############################################################################

clear
print_header "${ROCKET} Devilbox Modern Setup Wizard"

echo -e "${BOLD}Welcome to the Devilbox Modern Setup!${NC}"
echo ""
echo "This wizard will help you:"
echo "  • Configure your environment"
echo "  • Build custom PHP images (8.3, 8.4)"
echo "  • Set up your development stack"
echo "  • Create your first project"
echo "  • Install optional tools (MCP server)"
echo ""
echo -e "${YELLOW}This will take approximately 10-20 minutes.${NC}"
echo ""

if ! ask_yes_no "Ready to begin?" "y"; then
    echo ""
    print_info "Setup cancelled. Run ${BOLD}./setup-devilbox.sh${NC} when ready."
    exit 0
fi

###############################################################################
# Pre-flight Checks
###############################################################################

print_section "${GEAR} Pre-flight Checks"

# Check Docker
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed"
    echo "Please install Docker Desktop from: https://www.docker.com/products/docker-desktop"
    exit 1
fi
print_success "Docker is installed"

# Check Docker is running
if ! docker info &> /dev/null; then
    print_error "Docker is not running"
    echo "Please start Docker Desktop and try again"
    exit 1
fi
print_success "Docker is running"

# Check docker-compose
if ! docker compose version &> /dev/null; then
    print_error "Docker Compose is not available"
    exit 1
fi
print_success "Docker Compose is available"

# Check for .env file
if [ ! -f .env ]; then
    print_info "Creating .env from env-example"
    cp env-example .env
    print_success ".env file created"
else
    print_success ".env file exists"
fi

###############################################################################
# PHP Version Selection
###############################################################################

print_section "${PACKAGE} PHP Version Selection"

echo "Which PHP version(s) do you want to use?"
echo "  ${BOLD}1)${NC} PHP 8.4 only (recommended for new projects)"
echo "  ${BOLD}2)${NC} PHP 8.3 only"
echo "  ${BOLD}3)${NC} Both 8.3 and 8.4 (can switch between them)"
echo ""

while true; do
    ask_question "Enter choice" "1" "php_choice"

    case "$php_choice" in
        1)
            PHP_VERSIONS=("8.4")
            DEFAULT_PHP="8.4"
            print_success "Selected: PHP 8.4"
            break
            ;;
        2)
            PHP_VERSIONS=("8.3")
            DEFAULT_PHP="8.3"
            print_success "Selected: PHP 8.3"
            break
            ;;
        3)
            PHP_VERSIONS=("8.3" "8.4")
            DEFAULT_PHP="8.4"
            print_success "Selected: Both PHP 8.3 and 8.4 (default: 8.4)"
            break
            ;;
        *)
            print_error "Invalid choice. Please enter 1, 2, or 3"
            ;;
    esac
done

# Update .env with selected PHP version
sed -i.bak "s/^PHP_SERVER=.*/PHP_SERVER=${DEFAULT_PHP}/" .env
print_info "Set PHP_SERVER=${DEFAULT_PHP} in .env"

###############################################################################
# Basic Configuration
###############################################################################

print_section "${WRENCH} Basic Configuration"

# MySQL Root Password
ask_question "MySQL root password" "root" "MYSQL_PASS"
sed -i.bak "s/^MYSQL_ROOT_PASSWORD=.*/MYSQL_ROOT_PASSWORD=${MYSQL_PASS}/" .env
print_success "MySQL password configured"

# HTTP Port
ask_question "HTTP port (80 for localhost, 8000 for localhost:8000)" "80" "HTTP_PORT"
sed -i.bak "s/^HOST_PORT_HTTPD=.*/HOST_PORT_HTTPD=${HTTP_PORT}/" .env
print_success "HTTP port set to ${HTTP_PORT}"

# HTTPS Port
ask_question "HTTPS port" "443" "HTTPS_PORT"
sed -i.bak "s/^HOST_PORT_HTTPS=.*/HOST_PORT_HTTPS=${HTTPS_PORT}/" .env
print_success "HTTPS port set to ${HTTPS_PORT}"

# TLD Suffix
ask_question "TLD suffix for projects (e.g., .local, .test, .dev)" "local" "TLD"
sed -i.bak "s/^TLD_SUFFIX=.*/TLD_SUFFIX=${TLD}/" .env
print_success "TLD suffix set to .${TLD}"

print_info "Your projects will be accessible at: http://project-name.${TLD}"

# Clean up backup files
rm -f .env.bak

###############################################################################
# PATH Setup
###############################################################################

print_section "${SPARKLES} Development Tools Setup"

echo "Add Devilbox command wrappers to your PATH?"
echo "This allows you to run commands like:"
echo "  ${BOLD}composer install${NC} - runs in PHP container"
echo "  ${BOLD}npm install${NC} - runs in PHP container"
echo "  ${BOLD}mysql${NC} - connects to MySQL"
echo ""

if ask_yes_no "Add to PATH?" "y"; then
    # Detect shell
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="${HOME}/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        SHELL_RC="${HOME}/.bashrc"
    else
        SHELL_RC="${HOME}/.profile"
    fi

    PATH_LINE="export PATH=\"${SCRIPT_DIR}/bin:\$PATH\""

    # Check if already in PATH
    if grep -q "devilbox/bin" "$SHELL_RC" 2>/dev/null; then
        print_info "Already in PATH"
    else
        echo "" >> "$SHELL_RC"
        echo "# Devilbox command wrappers" >> "$SHELL_RC"
        echo "$PATH_LINE" >> "$SHELL_RC"
        print_success "Added to ${SHELL_RC}"
        print_warning "Run ${BOLD}source ${SHELL_RC}${NC} or restart your terminal"
    fi
fi

###############################################################################
# Build PHP Images
###############################################################################

print_section "${FIRE} Build Custom PHP Images"

echo "Build custom PHP images with modern tools?"
echo "Includes: Laravel, WP-CLI, Bun, Vite, Pest, React, Vue, Angular"
echo ""
echo -e "${YELLOW}This will take 10-15 minutes per image${NC}"
echo ""

if ask_yes_no "Build images now?" "y"; then
    for version in "${PHP_VERSIONS[@]}"; do
        print_info "Building PHP ${version} image..."
        if ./docker-images/build-php.sh "$version"; then
            print_success "PHP ${version} image built successfully"
        else
            print_error "Failed to build PHP ${version} image"
            echo "You can build it later with: ./docker-images/build-php.sh ${version}"
        fi
    done
else
    print_warning "Skipping image build"
    print_info "Build later with: ./docker-images/build-php.sh 8.4"
fi

###############################################################################
# Optional Modern Services
###############################################################################

print_section "${PACKAGE} Optional Modern Services"

echo "Enable additional services? (You can add these later)"
echo ""
echo "Available services:"
echo "  ${BOLD}Meilisearch${NC} - Lightning-fast search engine"
echo "  ${BOLD}Mailpit${NC} - Modern email testing (replaces Mailhog)"
echo "  ${BOLD}RabbitMQ${NC} - Message queue for async tasks"
echo "  ${BOLD}MinIO${NC} - S3-compatible object storage"
echo ""

if ask_yes_no "Configure modern services now?" "n"; then
    SELECTED_SERVICES=""

    # Meilisearch
    echo ""
    if ask_yes_no "Enable Meilisearch (search engine)?" "n"; then
        SELECTED_SERVICES="${SELECTED_SERVICES} meilisearch"
        print_success "Meilisearch will be enabled"
    fi

    # Mailpit
    if ask_yes_no "Enable Mailpit (email testing)?" "n"; then
        SELECTED_SERVICES="${SELECTED_SERVICES} mailpit"
        print_success "Mailpit will be enabled"
    fi

    # RabbitMQ
    if ask_yes_no "Enable RabbitMQ (message queue)?" "n"; then
        SELECTED_SERVICES="${SELECTED_SERVICES} rabbit"
        print_success "RabbitMQ will be enabled"
    fi

    # MinIO
    if ask_yes_no "Enable MinIO (S3 storage)?" "n"; then
        SELECTED_SERVICES="${SELECTED_SERVICES} minio"
        print_success "MinIO will be enabled"
    fi

    # Create docker-compose.override.yml if any services selected
    if [ -n "$SELECTED_SERVICES" ]; then
        if [ -f "docker-compose.override.yml" ]; then
            print_warning "docker-compose.override.yml already exists"
            if ask_yes_no "Overwrite it?" "n"; then
                cp compose/docker-compose.override.yml-modern-services docker-compose.override.yml
                print_success "Created docker-compose.override.yml with modern services"
            fi
        else
            cp compose/docker-compose.override.yml-modern-services docker-compose.override.yml
            print_success "Created docker-compose.override.yml with modern services"
        fi

        print_info "Services will start with Devilbox"
        print_info "Selected:${SELECTED_SERVICES}"
    else
        print_info "No services selected"
    fi
else
    print_info "Skipped. Enable later by copying from compose/ directory"
    echo ""
    echo "Examples:"
    echo "  ${BOLD}cp compose/docker-compose.override.yml-meilisearch docker-compose.override.yml${NC}"
    echo "  ${BOLD}cp compose/docker-compose.override.yml-modern-services docker-compose.override.yml${NC} (all 4)"
fi

###############################################################################
# Start Devilbox
###############################################################################

print_section "${ROCKET} Start Devilbox"

if ask_yes_no "Start Devilbox now?" "y"; then
    print_info "Starting Devilbox containers..."

    if docker compose up -d; then
        print_success "Devilbox is running!"
        echo ""
        echo "Access points:"
        if [ "$HTTP_PORT" = "80" ]; then
            echo "  Dashboard: ${BOLD}http://localhost${NC}"
        else
            echo "  Dashboard: ${BOLD}http://localhost:${HTTP_PORT}${NC}"
        fi
        echo "  phpMyAdmin: ${BOLD}http://localhost/vendor/phpmyadmin-5.2.3/${NC}"
        echo "  Adminer: ${BOLD}http://localhost/vendor/adminer-5.4.2-devilbox.php${NC}"

        # Show modern service links if enabled
        if [ -n "$SELECTED_SERVICES" ]; then
            echo ""
            echo "Modern services:"
            if [[ "$SELECTED_SERVICES" == *"meilisearch"* ]]; then
                echo "  Meilisearch: ${BOLD}http://localhost:7700${NC}"
            fi
            if [[ "$SELECTED_SERVICES" == *"mailpit"* ]]; then
                echo "  Mailpit: ${BOLD}http://localhost:8025${NC}"
            fi
            if [[ "$SELECTED_SERVICES" == *"rabbit"* ]]; then
                echo "  RabbitMQ: ${BOLD}http://localhost:15672${NC} (guest/guest)"
            fi
            if [[ "$SELECTED_SERVICES" == *"minio"* ]]; then
                echo "  MinIO Console: ${BOLD}http://localhost:9001${NC} (minioadmin/minioadmin)"
                echo "  MinIO API: ${BOLD}http://localhost:9000${NC}"
            fi
        fi
    else
        print_error "Failed to start Devilbox"
        exit 1
    fi
else
    print_info "Start Devilbox later with: docker compose up -d"
fi

###############################################################################
# Create First Project
###############################################################################

print_section "${SPARKLES} Create Your First Project"

if ask_yes_no "Create a starter project?" "y"; then
    echo ""
    echo "What type of project?"
    echo "  ${BOLD}1)${NC} Laravel (PHP framework)"
    echo "  ${BOLD}2)${NC} WordPress (CMS)"
    echo "  ${BOLD}3)${NC} Custom PHP"
    echo "  ${BOLD}4)${NC} Skip"
    echo ""

    ask_question "Enter choice" "1" "project_type"

    case "$project_type" in
        1)
            ask_question "Project name (e.g., my-blog)" "" "project_name"
            if [ -n "$project_name" ]; then
                print_info "Creating Laravel project: ${project_name}"
                docker compose exec php laravel new "$project_name"
                print_success "Laravel project created!"
                print_info "Access at: ${BOLD}http://${project_name}.${TLD}${NC}"
                print_info "Wait 30 seconds for vhost auto-detection to complete"
            fi
            ;;
        2)
            ask_question "Project name (e.g., my-site)" "" "project_name"
            if [ -n "$project_name" ]; then
                print_info "Creating WordPress project: ${project_name}"
                docker compose exec php wp core download --path="$project_name"
                print_success "WordPress downloaded!"
                print_info "Access at: ${BOLD}http://${project_name}.${TLD}${NC}"
                print_info "Complete setup in browser"
            fi
            ;;
        3)
            ask_question "Project name" "" "project_name"
            if [ -n "$project_name" ]; then
                mkdir -p "data/www/$project_name"
                cat > "data/www/$project_name/index.php" <<'EOF'
<?php
phpinfo();
EOF
                print_success "Custom PHP project created!"
                print_info "Access at: ${BOLD}http://${project_name}.${TLD}${NC}"
            fi
            ;;
        *)
            print_info "Skipping project creation"
            ;;
    esac
fi

###############################################################################
# MCP Server Installation
###############################################################################

print_section "${BOOK} Optional: Claude Code Integration"

echo "Install MCP server for Claude Code integration?"
echo "This enables AI-assisted development with Devilbox"
echo ""

if ask_yes_no "Install MCP server?" "n"; then
    if [ -f mcp-server/install.sh ]; then
        print_info "Running MCP installer..."
        cd mcp-server
        ./install.sh
        cd "$SCRIPT_DIR"
    else
        print_warning "MCP server not found in mcp-server/"
        print_info "Install manually later with: cd mcp-server && ./install.sh"
    fi
fi

###############################################################################
# Completion
###############################################################################

clear
print_header "${CHECK} Setup Complete!"

echo -e "${GREEN}${BOLD}Devilbox is ready!${NC}"
echo ""
echo "Quick Reference:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "${BOLD}Access Points:${NC}"
if [ "$HTTP_PORT" = "80" ]; then
    echo "  Dashboard:    http://localhost"
else
    echo "  Dashboard:    http://localhost:${HTTP_PORT}"
fi
echo "  phpMyAdmin:   http://localhost/vendor/phpmyadmin-5.2.3/"
echo "  Adminer:      http://localhost/vendor/adminer-5.4.2-devilbox.php"
echo ""
echo "${BOLD}Common Commands:${NC}"
echo "  ${CYAN}docker compose ps${NC}              - View running containers"
echo "  ${CYAN}docker compose logs -f php${NC}     - View PHP logs"
echo "  ${CYAN}docker compose restart${NC}         - Restart all containers"
echo "  ${CYAN}docker compose stop${NC}            - Stop Devilbox"
echo ""
echo "${BOLD}Create New Projects:${NC}"
echo "  ${CYAN}docker compose exec php laravel new my-project${NC}"
echo "  ${CYAN}docker compose exec php wp core download --path=my-wp${NC}"
echo ""
echo "${BOLD}Documentation:${NC}"
echo "  ${CYAN}.claude/README.md${NC}              - Development guidelines"
echo "  ${CYAN}.claude/skills/${NC}                - How-to guides"
echo "  ${CYAN}ROADMAP-MODERNIZATION.md${NC}       - Project roadmap"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}Happy coding! ${SPARKLES}${NC}"
echo ""
