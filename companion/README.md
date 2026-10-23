# Devilbox Boost Companion

A small native menu bar app for the Devilbox Boost stack, built with
[Tauri 2](https://tauri.app). It replaces the browser tab you otherwise keep
open for the intranet. Created by [moducraft.io](https://moducraft.io), MIT licensed.

What it does:

- **Menu bar status** of the Docker stack with start, stop and restart, refreshed every 15 seconds.
- **Projects**: every folder in `data/www` with its detected type (Laravel, Statamic,
  WordPress, Symfony, Next.js, static), its `https://<name>.<tld>` URL and the database
  it uses. One click opens the site, the folder in your editor, a terminal, Finder, or
  the database in your SQL client.
- **Databases**: lists MariaDB databases and opens any of them in Sequel Ace or TablePlus
  (via the standard `mysql://` URL), in Adminer or in phpMyAdmin.
- **Tools**: phpMyAdmin, Adminer, phpPgAdmin, phpCacheAdmin, OPcache GUI, Mailpit,
  RabbitMQ, Meilisearch and the RustFS console. Each opens in its own window inside the
  app. Pages that allow framing (the intranet itself) are embedded in the main window.
- **Logs** for any service, straight from `docker compose logs`.
- **Alerts**: desktop notifications when a container stops on its own or PHP-FPM workers
  start crashing (segfault lines in the php log). Toggle in Settings.
- **Quick open**: `Cmd+Shift+D` from anywhere opens the project launcher; type to filter,
  Enter opens the first match in your browser.
- **Updates**: check and update a single site, or all WordPress sites, or all Composer
  projects in one click, and optionally every day at a set time while the app is open.
  WordPress: plugins, themes, core, database upgrade, translations, cache flush.
  Composer projects: `composer update`, `artisan migrate`, `please updates:run`, `npm install`
  and `npm run build` when present.
- **New project**: Laravel, Statamic, Symfony, WordPress or static HTML, created inside the
  PHP container with the database, `.env` or `wp-config.php`, app key, migrations, frontend
  build and `htdocs` symlink already in place.
- **Files**: browse any project folder, drop files or folders onto the window to copy them
  in, optionally extracting `.zip` and `.tar.gz` archives (for example a `wp-content/uploads`
  archive from production).
- **Project detail panel**: click any project for its database (SQL client, Adminer,
  phpMyAdmin, dump import, snapshots with restore), workers (queue, schedule, horizon,
  reverb inside the container) and a host-side Vite dev server, the production media
  fallback toggle, live logs, and an embedded terminal (bash in the PHP container, in the
  project folder).
- **Doctor**: stack and per-site health checks with the reason attached (missing plugin
  folders, incomplete plugins, missing databases, fatal errors, pending migrations, DNS/CA).
- **Import DB…** on any project or on the Databases page: pick a `.sql`, `.sql.gz` or `.zip`
  dump. WordPress projects go through `tools/import-wp-dumps.sh` (prefix and URL detection,
  wp-config.php); everything else is a plain import into the database you name.

Everything that touches sites runs through the scripts in `../tools/` (the same ones behind
`bin/dbox`), so the app and the CLI behave identically. Docker access shells out to
`docker compose` in the Devilbox folder, the same way the MCP server does.

## Requirements

- macOS 12 or newer (Linux and Windows builds are possible with Tauri but untested).
- Docker Desktop with the `docker compose` plugin.
- Rust (`rustup`) and Node 20+ to build from source.
- Sequel Ace or TablePlus for the "open in DB client" buttons (optional).

## Run from source

```bash
cd companion
npm install
npm run tauri dev
```

The first Rust build takes a few minutes. The app finds the Devilbox folder from, in
order: the saved setting, the `DEVILBOX_ROOT` environment variable, the checkout it was
built from, and a few common locations under your home folder. Change it in Settings.

## Build the app bundle

```bash
cd companion
npm run tauri build
```

The `.app` and `.dmg` land in `src-tauri/target/release/bundle/`. The bundle is ad-hoc
signed (no Apple developer account). When someone downloads it, macOS will warn the first
time; right-click the app and choose Open once, or run
`xattr -d com.apple.quarantine "/Applications/Devilbox Boost.app"`.

## Settings

Stored in `~/Library/Application Support/io.github.drmzindec.devilbox-boost/settings.json`.

| Key | Default | Purpose |
|---|---|---|
| `root` | auto-detected | Path of the Devilbox Boost checkout |
| `db_client` | `Sequel Ace` | macOS app name opened with `mysql://root:…@127.0.0.1:3306/<db>` |
| `editor` | first detected editor | App used by the Editor button (PhpStorm, VS Code, Cursor, Zed, … or any app name) |
| `terminal` | `Terminal` | App used by the Terminal button |
| `notifications` | `true` | Desktop alerts for stopped containers and PHP-FPM crashes |

The MariaDB root password, host port, TLD and SSL mode are read from the stack's `.env`.

## Layout

```
companion/
├── index.html, src/            Front end (vanilla TypeScript + Vite)
└── src-tauri/
    ├── src/lib.rs              Commands, tray, stack polling, alerts, project detection, dump import
    ├── src/sites.rs            Update checks/updates, schedule, new-project creation
    ├── src/files.rs            Project file explorer and uploader
    ├── tauri.conf.json         Window, bundle and identifier
    └── capabilities/           Permissions granted to the main window
```
