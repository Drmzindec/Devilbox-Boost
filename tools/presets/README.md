# Presets

A preset is a bash file run right after a project has been created by
`tools/new-project.sh` (or the companion's New project view). Put your own in
`~/.config/devilbox-boost/presets/<name>.sh`; the ones here ship with Devilbox Boost.

```bash
# PRESET_TYPE: laravel          # project type it applies to, or "any"
# PRESET_DESC: One line shown in the UI
preset_run() {
  php_exec "$PROJECT_CT" composer require vendor/package
}
```

Available inside `preset_run`: `PROJECT`, `PROJECT_DIR` (host path), `PROJECT_CT`
(path inside the PHP container), `PROJECT_URL`, `PROJECT_DB`, `PROJECT_TYPE`,
`ADMIN_USER`, `ADMIN_PASS`, `ADMIN_EMAIL`, plus the helpers from `tools/lib.sh`
(`php_exec`, `wpc`, `sql`, `set_env`, `log`, `step`, `warn`).
