# Devilbox Boost Companion

A small native menu bar app for the Devilbox Boost stack, built with
[Tauri 2](https://tauri.app). It replaces the browser tab you otherwise keep
open for the intranet.

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

All Docker access shells out to `docker compose` in the Devilbox folder, the same way the
MCP server does, so nothing new has to be installed in the stack.

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
| `editor` | `Visual Studio Code` | App used by the Editor button |
| `terminal` | `Terminal` | App used by the Terminal button |

The MariaDB root password, host port, TLD and SSL mode are read from the stack's `.env`.

## Layout

```
companion/
├── index.html, src/            Front end (vanilla TypeScript + Vite)
└── src-tauri/
    ├── src/lib.rs              Commands, tray, stack polling, project detection
    ├── tauri.conf.json         Window, bundle and identifier
    └── capabilities/           Permissions granted to the main window
```
