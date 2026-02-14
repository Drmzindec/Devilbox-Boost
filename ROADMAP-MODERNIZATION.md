# Devilbox Modernization Roadmap

## Vision
Create a modern, AI-friendly, developer-focused fork/enhancement of Devilbox that's actively maintained and easy to use.

---

## ✅ COMPLETED: Smart Vhost Auto-Detection

**Status**: DONE - Fully implemented and working!

The critical vhost auto-detection feature is now complete:
- Automatically detects Laravel, WordPress, Symfony, CakePHP, Yii, CodeIgniter
- Creates `.devilbox/apache24.yml` and `nginx.yml` with correct DocumentRoot
- Laravel projects work out-of-the-box with `/public` docroot
- WordPress projects work with proper permalink support
- Runs as background service, checks every 30 seconds
- Successfully detected and configured all 10 existing projects

**See**: `VHOST-AUTO-DETECT.md` for full documentation

---

## Phase 1: Setup Wizard (NEXT - Immediate Impact) ✨

### Interactive TUI Setup
Create `./setup-devilbox.sh` that handles everything:

```bash
#!/bin/bash
# Interactive Devilbox Setup Wizard

echo "🚀 Devilbox Modern Setup Wizard"
echo "================================"

# 1. PHP Version Selection
echo "Which PHP versions do you want? (8.3, 8.4, or both)"
read -p "Choice [both]: " php_choice

# 2. Basic Configuration
echo "Setting up .env configuration..."
read -p "MySQL Root Password [secret]: " mysql_pass
read -p "Host Port for HTTP [80]: " http_port
read -p "TLD Suffix [local]: " tld

# 3. Update .env file automatically
sed -i '' "s/^MYSQL_ROOT_PASSWORD=.*/MYSQL_ROOT_PASSWORD=${mysql_pass:-secret}/" .env
sed -i '' "s/^HOST_PORT_HTTPD=.*/HOST_PORT_HTTPD=${http_port:-80}/" .env
sed -i '' "s/^TLD_SUFFIX=.*/TLD_SUFFIX=${tld:-local}/" .env

# 4. PATH Setup
echo "Add command wrappers to PATH? (Recommended)"
read -p "[Y/n]: " add_path
if [[ "$add_path" != "n" ]]; then
    shell_rc="${HOME}/.zshrc"  # or detect bash
    echo 'export PATH="/Users/johanpretorius/devilbox/bin:$PATH"' >> "$shell_rc"
    echo "✅ Added to $shell_rc"
fi

# 5. Build PHP Images
echo "Build custom PHP images with modern tools? (Bun, Vite, etc.)"
read -p "[Y/n]: " build_php
if [[ "$build_php" != "n" ]]; then
    if [[ "$php_choice" == "both" ]] || [[ "$php_choice" == "8.4" ]]; then
        ./docker-images/build-php.sh 8.4
    fi
    if [[ "$php_choice" == "both" ]] || [[ "$php_choice" == "8.3" ]]; then
        ./docker-images/build-php.sh 8.3
    fi
fi

# 6. Start Devilbox
echo "Start Devilbox now?"
read -p "[Y/n]: " start_now
if [[ "$start_now" != "n" ]]; then
    docker-compose up -d
fi

# 7. Create First Project
echo "Create your first Laravel project?"
read -p "[y/N]: " create_laravel
if [[ "$create_laravel" == "y" ]]; then
    read -p "Project name: " project_name
    docker exec -it devilbox-php-1 laravel new "$project_name"
    ./setup-laravel-vhost.sh "$project_name"
    docker-compose restart httpd
    echo "✅ Laravel project created: http://$project_name.$tld"
fi

echo ""
echo "🎉 Devilbox setup complete!"
echo "📚 See QOL-SETUP.md for usage guide"
echo "🔧 See MODERN-TOOLS.md for available tools"
```

---

## Phase 2: Update Admin Tools (CRITICAL - Security & PHP 8.4)

### Problem
All admin tools showing PHP 8.4 deprecation warnings due to outdated versions.

### Solution: Auto-Update Script

Create `./update-admin-tools.sh`:

```bash
#!/bin/bash
# Auto-update database admin tools

VENDOR_DIR="./.devilbox/www/htdocs/vendor"

echo "🔄 Updating Devilbox admin tools..."

# 1. Update phpMyAdmin to 5.2.1
echo "Updating phpMyAdmin..."
wget https://files.phpmyadmin.net/phpMyAdmin/5.2.1/phpMyAdmin-5.2.1-all-languages.tar.gz
tar -xzf phpMyAdmin-5.2.1-all-languages.tar.gz
rm -rf "$VENDOR_DIR/phpmyadmin-5.2.1"
mv phpMyAdmin-5.2.1-all-languages "$VENDOR_DIR/phpmyadmin-5.2.1"
rm phpMyAdmin-5.2.1-all-languages.tar.gz

# 2. Update phpPgAdmin to 7.14.7
echo "Updating phpPgAdmin..."
wget https://github.com/phppgadmin/phppgadmin/releases/download/REL_7-14-7/phppgadmin-7.14.7.tar.gz
tar -xzf phppgadmin-7.14.7.tar.gz
rm -rf "$VENDOR_DIR/phppgadmin-7.14.7"
mv phppgadmin-7.14.7 "$VENDOR_DIR/phppgadmin-7.14.7"
rm phppgadmin-7.14.7.tar.gz

# 3. Replace phpMemcachedAdmin with modern alternative
echo "Replacing phpMemcachedAdmin with MemcacheDashboard..."
# TODO: Find modern replacement

# 4. Update Adminer (already latest)
echo "✅ Adminer is up-to-date (4.8.1)"

# 5. Update OpCache viewer
echo "Updating OpCache viewer..."
wget https://raw.githubusercontent.com/amnuts/opcache-gui/master/index.php -O "$VENDOR_DIR/ocp-modern.php"

echo "✅ Admin tools updated!"
echo "🔄 Restart containers: docker-compose restart"
```

### Alternative: Suppress Warnings
Add to PHP config:
```ini
error_reporting = E_ALL & ~E_DEPRECATED & ~E_STRICT
display_errors = Off
log_errors = On
```

---

## Phase 3: MCP Server for Devilbox (GAME CHANGER)

### What is MCP?
Model Context Protocol - allows Claude Code to directly interact with Devilbox.

### Features

**1. Devilbox Resources**
- List all projects/vhosts
- Show project configuration
- Display database lists
- Show running services

**2. Devilbox Tools**
```typescript
// Create new Laravel project
{
  "name": "create_laravel_project",
  "arguments": {
    "name": "myblog",
    "setup_vhost": true,
    "create_database": true
  }
}

// Run Artisan command
{
  "name": "run_artisan",
  "arguments": {
    "project": "myblog",
    "command": "migrate --seed"
  }
}

// Create database
{
  "name": "create_database",
  "arguments": {
    "name": "myblog_db",
    "type": "mysql"
  }
}

// Build assets
{
  "name": "build_assets",
  "arguments": {
    "project": "myblog",
    "tool": "bun"  // auto-detects vite/webpack
  }
}
```

**3. Smart Features**
- Auto-detect Laravel/WordPress projects
- Suggest database names based on project
- Auto-configure vhosts for detected frameworks
- Run migrations after creating databases
- Smart error detection and fixes

### Implementation

**File structure:**
```
devilbox-mcp/
├── package.json
├── src/
│   ├── index.ts          # MCP server entry
│   ├── resources.ts      # List projects, databases, etc.
│   ├── tools/
│   │   ├── projects.ts   # Create/manage projects
│   │   ├── databases.ts  # Create/manage databases
│   │   ├── artisan.ts    # Laravel commands
│   │   └── compose.ts    # Composer commands
│   └── utils/
│       ├── docker.ts     # Docker exec wrapper
│       └── detect.ts     # Framework detection
└── README.md
```

**Example `index.ts`:**
```typescript
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

const server = new Server(
  {
    name: "devilbox-mcp",
    version: "1.0.0",
  },
  {
    capabilities: {
      resources: {},
      tools: {},
    },
  }
);

// List all projects
server.setRequestHandler("resources/list", async () => {
  const projects = await getDevilboxProjects();
  return {
    resources: projects.map(p => ({
      uri: `devilbox://project/${p.name}`,
      name: p.name,
      mimeType: "application/json",
    })),
  };
});

// Create Laravel project tool
server.setRequestHandler("tools/call", async (request) => {
  if (request.params.name === "create_laravel_project") {
    const { name, setup_vhost, create_database } = request.params.arguments;

    // Run laravel new
    await exec(`docker exec devilbox-php-1 laravel new ${name}`);

    // Setup vhost
    if (setup_vhost) {
      await exec(`./setup-laravel-vhost.sh ${name}`);
    }

    // Create database
    if (create_database) {
      await exec(`docker exec devilbox-php-1 mysql -e "CREATE DATABASE ${name}"`);
    }

    return {
      content: [{
        type: "text",
        text: `✅ Laravel project '${name}' created at http://${name}.local`
      }]
    };
  }
});
```

**Installation for users:**
```bash
# In devilbox directory
npm install -g devilbox-mcp
devilbox-mcp install  # Adds to Claude Code MCP config
```

---

## Phase 4: Package as "Devilbox Modern"

### Distribution Options

**Option A: Fork & Maintain**
- Fork Devilbox repository
- Add all modern features
- Maintain separately
- Name: "Devilbox Modern" or "Devilbox 2026"

**Option B: Boost Pack (Recommended)**
- Layer on top of official Devilbox
- Installable via script
- Name: "Devilbox Boost"

**Option C: Hybrid**
- Contribute PHP 8.4 fixes upstream
- Maintain "Boost" pack for modern tools
- Best of both worlds

### Boost Pack Installation
```bash
curl -sSL https://raw.githubusercontent.com/user/devilbox-boost/main/install.sh | bash
```

**What it installs:**
- Modern PHP images (8.3, 8.4)
- QoL command wrappers
- Setup wizard
- Updated admin tools
- MCP server
- Documentation

---

## Implementation Priority

1. **✅ DONE**: Modern tools, port forwarding, PHP 8.4 fixes
2. **✅ DONE**: Smart vhost auto-detection for Laravel/WordPress/Symfony
3. **⏭️ NEXT**: Setup wizard (`setup-devilbox.sh`)
4. **🔥 URGENT**: Update admin tools (`update-admin-tools.sh`)
5. **🎯 HIGH**: Create MCP server
6. **📦 LATER**: Package as Boost Pack

---

## Benefits Over Official Devilbox

| Feature | Official Devilbox | Devilbox Modern |
|---------|-------------------|-----------------|
| PHP 8.4 Support | ❌ (last update 2023) | ✅ Full support |
| Modern Tools | ❌ Outdated (Grunt/Gulp era) | ✅ Bun, Vite, Pest |
| AI Integration | ❌ None | ✅ MCP server for Claude Code |
| Easy Setup | ⚠️ Manual .env editing | ✅ Interactive wizard |
| Command Wrappers | ❌ Must use shell.sh | ✅ Direct host commands |
| Laravel Support | ⚠️ Manual vhost setup | ✅ Auto-detect & configure |
| Admin Tools | ⚠️ Showing PHP warnings | ✅ Latest versions |
| Port Forwarding | ⚠️ Official images only | ✅ Built-in |
| Documentation | ⚠️ Outdated | ✅ Modern, comprehensive |

---

## Next Steps

### Completed ✅
- ✅ Modern tools (Bun, Vite, Pest, Laravel Installer, WP-CLI)
- ✅ Port forwarding for 127.0.0.1 database connections
- ✅ PHP 8.4 compatibility fixes
- ✅ Smart vhost auto-detection (Laravel, WordPress, Symfony)
- ✅ Command wrappers for easy CLI access
- ✅ Dashboard modernization

### Up Next 🚀
1. **Create setup wizard script** - Interactive TUI for easy Devilbox configuration
2. **Update admin tools** - Fix PHP 8.4 warnings in phpMyAdmin, phpPgAdmin, etc.
3. **Plan MCP server architecture** - Claude Code integration for AI-powered development
4. **Create GitHub repo for Boost Pack** - Package everything for easy distribution
5. **Write installation docs** - Complete setup and usage documentation

**The vhost auto-detection feature is working perfectly! Laravel projects now work out-of-the-box.**
