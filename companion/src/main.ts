import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

// ----- types mirrored from src-tauri -------------------------------------------------
interface Settings {
  root: string; db_client: string; editor: string; terminal: string; notifications: boolean;
  schedule_enabled: boolean; schedule_time: string; schedule_scope: string; schedule_last_run: string;
}
interface Environment {
  root: string; tld: string; scheme: string; vhost_port: string; ui_url: string;
  php: string; httpd: string; mysql: string; pgsql: string; redis: string; mongo: string;
  mysql_root_password: string; mysql_host_port: string; shortcut: string;
}
interface PortMap { published: number; target: number; url: string }
interface ServiceStatus { service: string; name: string; state: string; health: string; status: string; image: string; ports: PortMap[] }
interface Project { name: string; kind: string; url: string; path: string; db: string }
interface UpdateInfo { project: string; plugins: string[]; themes: string[]; core: string | null; packages: string[]; error: string | null }
interface BulkResult { project: string; ok: boolean; message: string }
interface Entry { name: string; path: string; is_dir: boolean; is_symlink: boolean; size: number; modified: number }
interface Listing { path: string; relative: string; parent: string | null; entries: Entry[] }
interface UploadReport { copied: number; extracted: string[]; errors: string[] }
interface ProjectType { id: string; description: string }
interface Preset { kind: string; name: string; description: string }
interface Finding { level: string; scope: string; message: string }
interface Snapshot { file: string; name: string; size: number; modified: number }
interface Worker { name: string; running: boolean; pid: string }
interface DevStatus { running: boolean; url: string }
interface PlanItem { source: string; name: string; action: string; target: string; note: string }
interface GitStatus { is_repo: boolean; branch: string; dirty: number; untracked: number; ahead: number; behind: number; remote: string; last_commit: string }

// ----- state -----------------------------------------------------------------------------
let settings: Settings = { root: "", db_client: "", editor: "Visual Studio Code", terminal: "Terminal", notifications: true, schedule_enabled: false, schedule_time: "03:00", schedule_scope: "wordpress", schedule_last_run: "" };
let env: Environment | null = null;
let status: ServiceStatus[] = [];
let projectList: Project[] = [];
let projectFilter = "";
let dbClients: string[] = [];
let editors: string[] = [];
let terminals: string[] = [];
let gitApps: string[] = [];
let fileManager = "Finder";
const updateInfo = new Map<string, UpdateInfo>();
const jobBuffers = new Map<string, string[]>();
const COMPOSER_KINDS = ["laravel", "statamic", "symfony", "cakephp", "craft", "yii", "drupal", "magento", "codeigniter", "typo3", "nette", "slim", "laminas", "bedrock"];
const UPDATABLE = ["wordpress", ...COMPOSER_KINDS];

const $ = <T extends HTMLElement>(sel: string, root: ParentNode = document) => root.querySelector(sel) as T;
const esc = (s: string) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c] as string));
const fmtSize = (n: number) => n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : n < 1073741824 ? `${(n / 1048576).toFixed(1)} MB` : `${(n / 1073741824).toFixed(2)} GB`;
const fmtDate = (epoch: number) => (epoch ? new Date(epoch * 1000).toLocaleString() : "");
const hasDbClient = () => !!settings.db_client && dbClients.includes(settings.db_client);

function toast(msg: string, error = false) {
  const t = $("#toast");
  t.textContent = msg;
  t.className = "toast" + (error ? " error" : "");
  t.hidden = false;
  window.setTimeout(() => (t.hidden = true), error ? 7000 : 2500);
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    toast(String(e), true);
    return undefined;
  }
}

async function loadProjectList() {
  if (!env) return;
  projectList = (await call<Project[]>("projects")) ?? projectList;
}

// ----- job output stream -----------------------------------------------------------------
listen<{ job: string; line: string }>("job-output", (e) => {
  const { job, line } = e.payload;
  const buf = jobBuffers.get(job) ?? [];
  buf.push(line); if (buf.length > 3000) buf.shift();
  jobBuffers.set(job, buf);
  const pre = document.querySelector<HTMLPreElement>(`pre[data-job="${CSS.escape(job)}"]`);
  if (pre) { pre.textContent = buf.join("\n"); pre.scrollTop = pre.scrollHeight; }
});
function jobPane(job: string, title = "Output") {
  return `<div class="job"><h2>${esc(title)}</h2><pre class="log" data-job="${esc(job)}">${esc((jobBuffers.get(job) ?? []).join("\n"))}</pre></div>`;
}

// ----- help -----------------------------------------------------------------------------------
const HELP: Record<string, { title: string; body: string }> = {
  overview: { title: "Overview", body: `<p>The state of the Docker stack. Each card is one container: its image, uptime and published ports. <b>Start / Stop / Restart</b> act on the whole stack; <b>Recreate</b> runs <code>docker compose up -d --force-recreate</code>, which you need after changing <code>.env</code> or the compose file. <b>Logs</b> shows the last 200 lines of that container.</p><p>The menu bar icon mirrors this: ● all running, ◐ degraded, ○ stopped. It also carries Tools and Projects submenus so you rarely need this window.</p>` },
  projects: { title: "Projects", body: `<p>Every folder in <code>data/www</code> is a site at <code>https://&lt;folder&gt;.${"${tld}"}</code>. The type is detected from the files (artisan, wp-content, composer.json …) and the database from <code>wp-config.php</code> or <code>.env</code>.</p><p>Click a project name for its detail panel: database and snapshots, workers and dev server, media fallback, logs and a terminal. <b>Editor</b>, <b>Terminal</b> and <b>Finder</b> use the apps chosen in Settings.</p><p><kbd>${"${shortcut}"}</kbd> opens this list from anywhere: type to filter, <kbd>Enter</kbd> opens the first match.</p>` },
  project: { title: "Project detail", body: `<p><b>Database</b>: open the project's database in your SQL client, Adminer or phpMyAdmin, import a dump (WordPress dumps get prefix and URL detection and a wp-config.php), and take or restore <b>snapshots</b>, which are compressed dumps in <code>backups/snapshots/&lt;db&gt;/</code>. A restore takes a safety snapshot first.</p><p><b>Workers &amp; dev server</b>: start <code>queue:work</code>, <code>schedule:work</code>, Horizon or Reverb inside the PHP container, and <code>npm run dev</code> on this Mac. The dev server is plain http, so browse the http URL while it runs.</p><p><b>Media fallback</b>: uploads are rarely in git. With this on, missing <code>/wp-content/uploads/…</code> files redirect to production so images keep working.</p><p><b>Logs</b>: web server and PHP errors for this host plus the application log, live. <b>Terminal</b>: bash in the PHP container, already in the project folder, with composer, artisan, wp and npm available.</p>` },
  databases: { title: "Databases", body: `<p>All MariaDB databases on <code>127.0.0.1:${"${port}"}</code> (user root). The Project column links back to the site using each database. Buttons open the database in your installed SQL client (only shown when one is detected in /Applications), in Adminer or in phpMyAdmin, each in its own window. <b>Import dump…</b> loads a <code>.sql</code>, <code>.sql.gz</code> or <code>.zip</code> into any database.</p>` },
  tools: { title: "Tools", body: `<p>The bundled web tools, each opening in its own window inside the app rather than a browser tab. Services that are not running show as such. The same list sits under the menu bar icon → Tools.</p>` },
  doctor: { title: "Doctor", body: `<p>Runs <code>dbox doctor</code>. <b>Stack</b>: Docker, container health, the <code>.${"${tld}"}</code> DNS resolver, the Devilbox CA in your keychain, the intranet, <code>check-config.sh</code>, recent PHP-FPM crashes and reclaimable image space. <b>Sites</b>: each vhost is probed and a 5xx comes with the matching error-log line; the configured database must exist and have tables; WordPress plugins that are active in the database but missing or incomplete on disk are listed (the usual cause is a gitignored premium plugin), as are fatal errors in debug.log and missing uploads; Laravel projects report missing vendor/, missing builds and pending migrations.</p>` },
  updates: { title: "Updates", body: `<p><b>Check</b> lists what is available without changing anything. <b>Update</b> applies it: for WordPress, plugins, themes, core, the database upgrade, translations and a cache flush via wp-cli; for Composer projects, <code>composer update</code>, migrations (<code>artisan migrate</code>, <code>please updates:run</code>, <code>drush updatedb</code>, <code>craft migrate/all</code>) and <code>npm install</code> + <code>npm run build</code> when there is a package.json.</p><p>The bulk buttons walk every site of that kind. The <b>schedule</b> runs the chosen scope once a day at the set time while the app is open and notifies you with the result. Take a snapshot first if a site matters.</p>` },
  create: { title: "New project", body: `<p>Creates a project inside the PHP container, so it uses the stack's PHP, Composer, wp-cli and Node. The database is created, <code>.env</code> / <code>wp-config.php</code> written for the stack (127.0.0.1, root), keys generated, migrations run, frontend built and the <code>htdocs</code> symlink set, then the folder is renamed into place so the vhost appears ready.</p><p><b>Presets</b> are small bash files run afterwards: ship-with ones live in <code>tools/presets/</code>, yours in <code>~/.config/devilbox-boost/presets/</code>. The same thing from the terminal: <code>dbox new &lt;type&gt; &lt;name&gt; --preset &lt;name&gt;</code>.</p>` },
  files: { title: "Files", body: `<p>A file browser limited to <code>data/www</code>. Drop files or folders anywhere on the window while this view is open to copy them into the current folder. Before anything is copied you get a plan: a folder dropped onto a folder with the same name (for example <code>wp-content</code> onto <code>wp-content</code>) merges instead of nesting, an archive whose only top-level folder matches the destination is unpacked straight into it, and existing files are listed as replaced. Untick <b>Extract archives</b> to copy a zip as a file.</p>` },
  intranet: { title: "Intranet", body: `<p>The original Devilbox dashboard, page by page. <b>Open here</b> embeds the page in this window with a bar to pop it out or open it in the browser; <b>Window</b> opens it on its own. The pages are grouped the same way as the intranet's own menu: Overview, Databases, Info, Logs &amp; config.</p>` },
  settings: { title: "Settings", body: `<p>Pick the Devilbox Boost checkout, your editor, terminal and SQL client (only installed apps are offered, with a custom name as fallback), and whether to receive desktop alerts. The schedule for automatic updates lives on the Updates page. Settings are stored in <code>~/Library/Application Support/io.github.drmzindec.devilbox-boost/settings.json</code>.</p>` },
};
const fillHelp = (body: string) => body.split("${tld}").join(env?.tld ?? "loc").split("${shortcut}").join(env?.shortcut ?? "Cmd+Shift+D").split("${port}").join(env?.mysql_host_port ?? "3306");
function helpButton(view: string) { return `<button class="help-btn" data-help="${esc(view)}" title="What is this page?">?</button>`; }
function wireHelp(root: HTMLElement) {
  root.querySelectorAll<HTMLButtonElement>("[data-help]").forEach((b) => b.addEventListener("click", () => {
    const h = HELP[b.dataset.help!]; if (!h) return;
    const existing = root.querySelector(".help-panel");
    if (existing) { existing.remove(); return; }
    const body = fillHelp(h.body);
    b.closest("h1")!.insertAdjacentHTML("afterend", `<div class="help-panel card"><h3>${esc(h.title)}</h3>${body}<div class="actions"><button class="btn small" id="help-more">Open the Help page</button><button class="btn small help-close">Close</button></div></div>`);
    root.querySelector(".help-close")!.addEventListener("click", () => root.querySelector(".help-panel")?.remove());
    root.querySelector("#help-more")!.addEventListener("click", () => showView("help"));
  }));
}
function renderHelp() {
  const v = $("#view-help");
  const order = ["overview", "projects", "project", "databases", "tools", "intranet", "doctor", "updates", "create", "files", "settings"];
  v.innerHTML = `<h1>Help</h1><p class="sub">What each part of Devilbox Boost does. Every page also has a <span class="help-btn" style="pointer-events:none">?</span> button next to its title.</p>
    ${order.map((k) => `<div class="card" style="margin-bottom:12px"><h3>${esc(HELP[k].title)}</h3>${fillHelp(HELP[k].body)}</div>`).join("")}
    <div class="card" style="margin-bottom:12px"><h3>Command line</h3><p>Everything here is also <code>bin/dbox</code> in the Devilbox folder: <code>dbox status</code>, <code>dbox doctor</code>, <code>dbox new</code>, <code>dbox update</code>, <code>dbox snapshot</code>, <code>dbox workers</code>, <code>dbox media</code>, <code>dbox logs</code>, <code>dbox import</code>, <code>dbox shell</code>. Run <code>dbox help</code> for the full list.</p></div>
    <div class="card"><h3>Where things live</h3><p>Sites: <code>data/www/&lt;name&gt;</code> · databases: MariaDB on 127.0.0.1:${esc(env?.mysql_host_port ?? "3306")} as root · snapshots: <code>backups/snapshots/</code> · presets: <code>tools/presets/</code> and <code>~/.config/devilbox-boost/presets/</code> · per-site web server tweaks: <code>&lt;project&gt;/.devilbox/nginx.yml</code> · shared Composer/npm caches: <code>cache/</code>.</p><p>Devilbox Boost is open source (MIT), created by <a class="link" data-open="https://moducraft.io">moducraft.io</a>.</p></div>`;
  wireCommonActions(v);
}

// ----- intranet ---------------------------------------------------------------------------------
const INTRANET_PAGES: { group: string; title: string; path: string; note: string }[] = [
  { group: "Overview", title: "Dashboard", path: "/index.php", note: "Service status, versions and connection details" },
  { group: "Overview", title: "Virtual hosts", path: "/vhosts.php", note: "Every project with its DNS and reachability check" },
  { group: "Overview", title: "Emails", path: "/mail.php", note: "Mail caught by the stack" },
  { group: "Overview", title: "Command & control", path: "/cnc.php", note: "Run commands inside the PHP container" },
  { group: "Databases", title: "MySQL / MariaDB", path: "/db_mysql.php", note: "Databases, tables and sizes" },
  { group: "Databases", title: "PostgreSQL", path: "/db_pgsql.php", note: "Databases and schemas" },
  { group: "Databases", title: "MongoDB", path: "/db_mongo.php", note: "Databases and collections" },
  { group: "Databases", title: "Redis", path: "/db_redis.php", note: "Keys per database" },
  { group: "Databases", title: "Memcached", path: "/db_memcd.php", note: "Slabs and stats" },
  { group: "Info", title: "Httpd info", path: "/info_httpd.php", note: "Web server configuration" },
  { group: "Info", title: "PHP info", path: "/info_php.php", note: "phpinfo() of the running PHP" },
  { group: "Info", title: "Xdebug info", path: "/info_xdebug.php", note: "Xdebug settings" },
  { group: "Info", title: "MySQL info", path: "/info_mysql.php", note: "Server variables" },
  { group: "Info", title: "PostgreSQL info", path: "/info_pgsql.php", note: "Server settings" },
  { group: "Info", title: "MongoDB info", path: "/info_mongo.php", note: "Server info" },
  { group: "Info", title: "Redis info", path: "/info_redis.php", note: "INFO output" },
  { group: "Info", title: "Memcached info", path: "/info_memcd.php", note: "Stats" },
  { group: "Logs & config", title: "PHP error log", path: "/log_php.php", note: "Global PHP-FPM error log" },
  { group: "Logs & config", title: "PHP config", path: "/config_php.php", note: "Effective php.ini values" },
  { group: "Logs & config", title: "Httpd config", path: "/config_httpd.php", note: "Effective web server config" },
];

function renderIntranet() {
  const v = $("#view-intranet");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  const groups = [...new Set(INTRANET_PAGES.map((p) => p.group))];
  const ui = env.ui_url;
  v.innerHTML = `<h1>Intranet ${helpButton("intranet")}</h1>
    <p class="sub">The Devilbox dashboard pages at ${esc(ui)}. <b>Open here</b> shows a page inside this window, <b>Window</b> gives it its own.</p>
    <div class="row" style="margin-bottom:8px"><button class="btn" data-tool="intranet" data-title="Devilbox intranet" data-url="${esc(ui)}">Whole intranet in a window</button><button class="btn" data-open="${esc(ui)}">Browser</button></div>
    ${groups.map((g) => `<h2>${esc(g)}</h2><div class="grid">${INTRANET_PAGES.filter((p) => p.group === g).map((p) => `<div class="card">
      <h3>${esc(p.title)}</h3><p>${esc(p.note)}</p>
      <div class="actions"><button class="btn small primary" data-frame="${esc(p.path)}" data-title="${esc(p.title)}">Open here</button><button class="btn small" data-tool="intranet-${esc(p.path.replace(/[^a-z]/g, ""))}" data-title="${esc(p.title)}" data-url="${esc(ui + p.path)}">Window</button></div></div>`).join("")}</div>`).join("")}`;
  v.querySelectorAll<HTMLButtonElement>("[data-frame]").forEach((b) => b.addEventListener("click", () => showFrame(b.dataset.frame!, b.dataset.title!)));
  v.querySelectorAll<HTMLButtonElement>("[data-tool]").forEach((b) => b.addEventListener("click", () => call("open_tool", { label: b.dataset.tool, title: b.dataset.title, url: b.dataset.url })));
  wireCommonActions(v);
}

// ----- navigation -------------------------------------------------------------------------
let currentProject: string | null = null;

function activate(viewId: string, navButton?: HTMLElement | null) {
  document.querySelectorAll<HTMLElement>(".view").forEach((v) => v.classList.toggle("active", v.id === viewId));
  document.querySelectorAll<HTMLElement>(".nav-item").forEach((b) => b.classList.toggle("active", b === navButton));
  if (viewId !== "view-project") closeTerminal();
}

function showView(name: string) {
  activate(`view-${name}`, document.querySelector<HTMLElement>(`.nav-item[data-view="${name}"]`));
  if (name === "projects") loadProjects();
  if (name === "databases") loadDatabases();
  if (name === "tools") renderTools();
  if (name === "settings") renderSettings();
  if (name === "overview") renderOverview();
  if (name === "updates") renderUpdates();
  if (name === "create") renderCreate();
  if (name === "files") renderFiles();
  if (name === "doctor") renderDoctor();
  if (name === "help") renderHelp();
  if (name === "intranet") renderIntranet();
}

function showFrame(path: string, title: string) {
  if (!env) return toast("Devilbox folder not set", true);
  activate("view-frame", document.querySelector<HTMLElement>('.nav-item[data-view="intranet"]'));
  const frame = $<HTMLIFrameElement>("#intranet-frame");
  const target = env.ui_url + path;
  if (frame.src !== target) frame.src = target;
  $("#frame-title").textContent = `${title} · ${target}`;
  const win = $<HTMLButtonElement>("#frame-window"), br = $<HTMLButtonElement>("#frame-browser");
  win.onclick = () => call("open_tool", { label: `intranet-${path.replace(/[^a-z]/g, "")}`, title, url: target });
  br.onclick = () => call("open_url", { url: target });
}
document.querySelector("#frame-back")?.addEventListener("click", () => showView("intranet"));

document.querySelectorAll<HTMLElement>(".nav-item[data-view]").forEach((b) => b.addEventListener("click", () => showView(b.dataset.view!)));
document.querySelectorAll<HTMLElement>(".sidebar [data-open]").forEach((b) => b.addEventListener("click", () => call("open_url", { url: b.dataset.open })));

listen("quick-open", async () => {
  showView("projects");
  await loadProjects();
  const s = $<HTMLInputElement>("#project-search");
  s.focus(); s.select();
});

// ----- status -------------------------------------------------------------------------------
function stateClass(s: ServiceStatus) {
  if (s.state !== "running") return "bad";
  if (s.health && s.health !== "healthy") return "warn";
  return "ok";
}

function applyStatus(list: ServiceStatus[]) {
  status = list;
  const running = list.filter((s) => s.state === "running").length;
  const dot = $("#brand-dot"), sub = $("#brand-sub");
  if (!env) { dot.className = "brand-dot"; sub.textContent = "folder not set"; }
  else if (list.length === 0) { dot.className = "brand-dot bad"; sub.textContent = "stack is down"; }
  else if (running === list.length) { dot.className = "brand-dot ok"; sub.textContent = `${running} services running`; }
  else if (running === 0) { dot.className = "brand-dot bad"; sub.textContent = "stopped"; }
  else { dot.className = "brand-dot warn"; sub.textContent = `${running} of ${list.length} running`; }
  if ($("#view-overview").classList.contains("active")) renderOverview();
}

async function refreshStatus() {
  if (!env) return applyStatus([]);
  const list = await call<ServiceStatus[]>("stack_status");
  if (list) applyStatus(list);
}
listen<ServiceStatus[]>("stack-status", (e) => applyStatus(e.payload));

// ----- overview --------------------------------------------------------------------------------
function portUrl(s: ServiceStatus): string | null {
  const web: Record<string, number> = { mailpit: 8025, rabbitmq: 15672, rabbit: 15672, meilisearch: 7700, rustfs: 9001, minio: 9001, httpd: 443 };
  const target = web[s.service];
  if (!target) return null;
  const p = s.ports.find((x) => x.target === target);
  return p ? `${s.service === "httpd" ? "https" : "http"}://localhost:${p.published}` : null;
}
function imageLabel(s: ServiceStatus) {
  const img = s.image.split("@")[0];
  if (img.startsWith("sha256:")) return s.service === "php" && env ? `devilbox-php-${env.php}:work (local build)` : `local image ${img.slice(7, 19)}`;
  return img;
}

function renderOverview() {
  const v = $("#view-overview");
  if (!env) {
    v.innerHTML = `<h1>Welcome</h1><p class="sub">Point the app at your Devilbox Boost folder to get started.</p><button class="btn primary" id="go-settings">Open settings</button>`;
    $("#go-settings").addEventListener("click", () => showView("settings"));
    return;
  }
  const running = status.filter((s) => s.state === "running").length;
  const cards = status.map((s) => {
    const url = portUrl(s);
    const ports = s.ports.map((p) => `${p.published}→${p.target}`).join(", ");
    return `<div class="card">
      <h3>${esc(s.service)} <span class="badge ${stateClass(s)}">${esc(s.health || s.state)}</span></h3>
      <p>${esc(imageLabel(s))}</p>
      <p>${esc(s.status)}${ports ? " · " + esc(ports) : ""}</p>
      <div class="actions">
        ${url ? `<button class="btn small" data-tool="${esc(s.service)}" data-url="${esc(url)}">Open</button>` : ""}
        <button class="btn small" data-action="restart" data-service="${esc(s.service)}">Restart</button>
        <button class="btn small" data-logs="${esc(s.service)}">Logs</button>
      </div></div>`;
  }).join("");
  v.innerHTML = `<h1>Overview ${helpButton("overview")}</h1>
    <p class="sub">${esc(env.root)} · PHP ${esc(env.php)} · ${esc(env.httpd)} · ${esc(env.mysql)} · projects at <code>*.${esc(env.tld)}</code></p>
    <div class="row">
      <button class="btn primary" data-action="start">Start stack</button>
      <button class="btn" data-action="stop">Stop stack</button>
      <button class="btn" data-action="restart">Restart stack</button>
      <button class="btn" data-action="recreate" title="docker compose up -d --force-recreate (needed after .env changes)">Recreate</button>
      <span class="badge ${status.length && running === status.length ? "ok" : running ? "warn" : "bad"}">${running} / ${status.length} running</span>
    </div>
    <h2>Services</h2>
    ${status.length ? `<div class="grid">${cards}</div>` : `<div class="card empty">The stack is not running. Start it above.</div>`}
    <h2 id="logs-title" hidden>Logs</h2><pre class="log" id="logs" hidden></pre>`;
  v.querySelectorAll<HTMLButtonElement>("[data-action]").forEach((b) => b.addEventListener("click", async () => {
    const action = b.dataset.action!, service = b.dataset.service;
    b.disabled = true; toast(`${action} ${service ?? "stack"}…`);
    const out = await call<string>("stack_action", { action, service: service ?? null });
    b.disabled = false;
    if (out !== undefined) { toast(`${action} done`); refreshStatus(); }
  }));
  v.querySelectorAll<HTMLButtonElement>("[data-logs]").forEach((b) => b.addEventListener("click", async () => {
    const text = await call<string>("stack_logs", { service: b.dataset.logs, lines: 200 });
    if (text === undefined) return;
    $("#logs-title").hidden = false;
    const pre = $("#logs"); pre.hidden = false; pre.textContent = text || "(no output)"; pre.scrollTop = pre.scrollHeight;
  }));
  v.querySelectorAll<HTMLButtonElement>("[data-tool]").forEach((b) => b.addEventListener("click", () => call("open_tool", { label: `svc-${b.dataset.tool}`, title: b.dataset.tool, url: b.dataset.url })));
  wireHelp(v);
}

// ----- shared action wiring ---------------------------------------------------------------------
function wireCommonActions(root: HTMLElement) {
  wireHelp(root);
  root.querySelectorAll<HTMLElement>("[data-open]").forEach((b) => b.addEventListener("click", () => call("open_url", { url: b.dataset.open })));
  root.querySelectorAll<HTMLElement>("[data-db]").forEach((b) => b.addEventListener("click", () => call("open_db_client", { db: b.dataset.db })));
  root.querySelectorAll<HTMLElement>("[data-adminer]").forEach((b) => b.addEventListener("click", () => openAdminer(b.dataset.adminer!)));
  root.querySelectorAll<HTMLElement>("[data-pma]").forEach((b) => b.addEventListener("click", () => openPma(b.dataset.pma!)));
  root.querySelectorAll<HTMLElement>("[data-project]").forEach((b) => b.addEventListener("click", () => openProject(b.dataset.project!)));
  root.querySelectorAll<HTMLElement>("[data-import-db]").forEach((b) => b.addEventListener("click", () =>
    startImport(root.querySelector<HTMLElement>("#import-panel")!, b.dataset.importProject || null, b.dataset.importDb || "")));
  root.querySelectorAll<HTMLElement>("[data-with]").forEach((b) => b.addEventListener("click", () => {
    const which = b.dataset.with!;
    const app = which === "editor" ? settings.editor : which === "terminal" ? settings.terminal : which;
    call("open_with", { appName: app, target: b.dataset.path });
  }));
}
function openAdminer(db: string) {
  if (!env) return;
  call("open_tool", { label: "adminer", title: `Adminer${db ? " · " + db : ""}`, url: `${env.ui_url}/vendor/adminer-6.1.2-devilbox.php?username=root${db ? `&db=${encodeURIComponent(db)}` : ""}` });
}
function openPma(db: string) {
  if (!env) return;
  call("open_tool", { label: "phpmyadmin", title: `phpMyAdmin${db ? " · " + db : ""}`, url: `${env.ui_url}/vendor/phpmyadmin-5.2.3/index.php${db ? `?route=/database/structure&db=${encodeURIComponent(db)}` : ""}` });
}
const dbButtons = (db: string, small = true) => `
  ${hasDbClient() ? `<button class="btn ${small ? "small" : ""}" data-db="${esc(db)}">${esc(settings.db_client)}</button>` : ""}
  <button class="btn ${small ? "small" : ""}" data-adminer="${esc(db)}">Adminer</button>
  <button class="btn ${small ? "small" : ""}" data-pma="${esc(db)}">phpMyAdmin</button>`;

// ----- projects ------------------------------------------------------------------------------------
async function loadProjects() {
  const v = $("#view-projects");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  await loadProjectList();
  renderProjects();
}

function renderProjects() {
  const v = $("#view-projects");
  const f = projectFilter.toLowerCase();
  const rows = projectList.filter((p) => !f || p.name.toLowerCase().includes(f) || p.kind.includes(f) || p.db.toLowerCase().includes(f));
  v.innerHTML = `<h1>Projects ${helpButton("projects")}</h1><p class="sub">${projectList.length} folders in data/www. Click a name for details, database, workers, logs and a terminal.</p>
    <div class="row" style="margin-bottom:12px">
      <input class="search" id="project-search" placeholder="Filter by name, type or database" value="${esc(projectFilter)}">
      <button class="btn" id="projects-reload">Reload</button>
      <button class="btn" id="projects-folder">Open data/www</button>
      <button class="btn" id="projects-new">New project</button>
    </div>
    <table><thead><tr><th>Project</th><th>Type</th><th>Database</th><th></th></tr></thead><tbody>
    ${rows.map((p) => `<tr>
      <td><a class="link" data-project="${esc(p.name)}">${esc(p.name)}</a><div class="kind">${esc(p.url)}</div></td>
      <td><span class="kind">${esc(p.kind)}</span></td>
      <td>${p.db ? esc(p.db) : `<span class="kind">–</span>`}</td>
      <td class="actions">
        <button class="btn small" data-open="${esc(p.url)}">Site</button>
        <button class="btn small" data-with="editor" data-path="${esc(p.path)}">Editor</button>
        <button class="btn small" data-project="${esc(p.name)}">Details</button>
      </td></tr>`).join("")}
    </tbody></table>
    ${rows.length ? "" : `<div class="empty">No projects match.</div>`}
    <p class="sub" style="margin-top:14px">Tip: <kbd>${esc(env?.shortcut ?? "Cmd+Shift+D")}</kbd> opens this list from anywhere; type to filter, Enter opens the first match.</p>`;
  const search = $<HTMLInputElement>("#project-search");
  search.addEventListener("input", () => { projectFilter = search.value; const pos = search.selectionStart; renderProjects(); const s2 = $<HTMLInputElement>("#project-search"); s2.focus(); s2.setSelectionRange(pos, pos); });
  search.addEventListener("keydown", (ev) => {
    if (ev.key === "Enter" && rows.length) { ev.preventDefault(); call("open_url", { url: rows[0].url }); }
    if (ev.key === "Escape") { projectFilter = ""; renderProjects(); }
  });
  $("#projects-reload").addEventListener("click", loadProjects);
  $("#projects-folder").addEventListener("click", () => call("open_with", { appName: "Finder", target: `${env!.root}/data/www` }));
  $("#projects-new").addEventListener("click", () => showView("create"));
  wireCommonActions(v);
}

// ----- project detail --------------------------------------------------------------------------------
let detailTab = "overview";

async function openProject(name: string, tab?: string) {
  const p = projectList.find((x) => x.name === name);
  if (!p) return toast(`project ${name} not found`, true);
  currentProject = name;
  if (tab) detailTab = tab;
  activate("view-project", document.querySelector<HTMLElement>('.nav-item[data-view="projects"]'));
  const v = $("#view-project");
  const tabs = ["overview", "database", "workers", "media", "logs", "terminal"];
  const labels: Record<string, string> = { overview: "Overview", database: "Database", workers: "Workers & dev server", media: "Media fallback", logs: "Logs", terminal: "Terminal" };
  v.innerHTML = `<button class="back" id="detail-back">← Projects</button>
    <h1 style="margin-top:8px">${esc(p.name)} <span class="badge muted">${esc(p.kind)}</span> ${helpButton("project")}</h1>
    <p class="sub"><a class="link" data-open="${esc(p.url)}">${esc(p.url)}</a> · ${esc(p.path)}${p.db ? ` · db <code>${esc(p.db)}</code>` : ""}</p>
    <div class="row">
      <button class="btn primary" data-open="${esc(p.url)}">Open site</button>
      <button class="btn" data-with="editor" data-path="${esc(p.path)}">Editor</button>
      <button class="btn" data-with="terminal" data-path="${esc(p.path)}">${esc(settings.terminal)}</button>
      <button class="btn" data-with="Finder" data-path="${esc(p.path)}">${esc(fileManager)}</button>
      <button class="btn" id="detail-files">Files</button>
      ${UPDATABLE.includes(p.kind) ? `<button class="btn" id="detail-update">Update</button>` : ""}
      <button class="btn" id="detail-doctor">Doctor</button>
    </div>
    <div class="tabs">${tabs.map((t) => `<button class="tab ${t === detailTab ? "active" : ""}" data-tab="${t}">${labels[t]}</button>`).join("")}</div>
    <div id="detail-body"></div>
    <div id="detail-output"></div>`;
  wireCommonActions(v);
  $("#detail-back").addEventListener("click", () => showView("projects"));
  $("#detail-files").addEventListener("click", () => { showView("files"); renderFiles(p.path); });
  $("#detail-doctor").addEventListener("click", async () => {
    $("#detail-output").innerHTML = jobPane("doctor", "Doctor");
    const findings = await call<Finding[]>("doctor", { project: p.name });
    if (findings) $("#detail-output").innerHTML = renderFindings(findings);
  });
  $("#detail-update")?.addEventListener("click", async () => {
    const job = `update-${p.name}`; jobBuffers.set(job, []);
    $("#detail-output").innerHTML = jobPane(job, `Updating ${p.name}`);
    const r = await call<BulkResult>("update_site", { project: p.name });
    if (r) toast(r.ok ? `${p.name} updated` : `${p.name}: ${r.message}`, !r.ok);
  });
  v.querySelectorAll<HTMLButtonElement>("[data-tab]").forEach((b) => b.addEventListener("click", () => { detailTab = b.dataset.tab!; openProject(name); }));
  if (detailTab !== "terminal") closeTerminal();
  const body = $("#detail-body");
  switch (detailTab) {
    case "overview": await renderDetailOverview(body, p); break;
    case "database": await renderDetailDatabase(body, p); break;
    case "workers": await renderDetailWorkers(body, p); break;
    case "media": await renderDetailMedia(body, p); break;
    case "logs": await renderDetailLogs(body, p); break;
    case "terminal": await renderDetailTerminal(body, p); break;
  }
}

async function renderDetailOverview(body: HTMLElement, p: Project) {
  const info = updateInfo.get(p.name);
  const g = (await call<GitStatus>("git_status", { project: p.name })) ?? ({ is_repo: false } as GitStatus);
  const gitCard = g.is_repo
    ? `<div class="card"><h3>Git <span class="badge ${g.dirty + g.untracked ? "warn" : "ok"}">${g.dirty + g.untracked ? `${g.dirty} changed, ${g.untracked} untracked` : "clean"}</span></h3>
        <p>Branch <code>${esc(g.branch)}</code>${g.ahead || g.behind ? ` · ${g.ahead} ahead, ${g.behind} behind` : ""}</p>
        <p>${esc(g.last_commit)}</p>${g.remote ? `<p class="muted">${esc(g.remote)}</p>` : ""}
        <div class="actions">${gitApps.map((a) => `<button class="btn small" data-with="${esc(a)}" data-path="${esc(p.path)}">Open in ${esc(a)}</button>`).join("")}<button class="btn small" data-with="editor" data-path="${esc(p.path)}">Editor</button></div></div>`
    : `<div class="card"><h3>Git</h3><p class="muted">Not a git repository.</p></div>`;
  body.innerHTML = `<div class="two">
    <div class="card"><h3>Local</h3>
      <p>URL <a class="link" data-open="${esc(p.url)}">${esc(p.url)}</a></p>
      <p>Folder <code>${esc(p.path)}</code></p>
      <p>Type ${esc(p.kind)}${p.db ? ` · database <code>${esc(p.db)}</code> on 127.0.0.1:${esc(env?.mysql_host_port ?? "3306")} (root)` : ""}</p>
    </div>
    <div class="card"><h3>Updates</h3>
      <p>${info ? (info.error ? esc(info.error) : [...info.plugins, ...info.themes, ...(info.core ? ["core " + info.core] : []), ...info.packages].map(esc).join("<br>") || "Up to date") : "Not checked yet."}</p>
      <div class="actions">${UPDATABLE.includes(p.kind) ? `<button class="btn small" id="ov-check">Check</button>` : `<span class="muted">Not an updatable project type.</span>`}</div>
    </div>
    ${gitCard}
  </div>`;
  wireCommonActions(body);
  $("#ov-check", body)?.addEventListener("click", async () => {
    const i = await call<UpdateInfo>("check_updates", { project: p.name });
    if (i) { updateInfo.set(p.name, i); renderDetailOverview(body, p); }
  });
}

async function renderDetailDatabase(body: HTMLElement, p: Project) {
  const db = p.db || p.name.replace(/-/g, "_");
  body.innerHTML = `<div class="card"><h3>Database <code>${esc(db)}</code></h3>
    <div class="actions">${dbButtons(db, false)}<button class="btn" data-import-db="${esc(db)}" data-import-project="${esc(p.name)}">Import dump…</button></div>
    ${hasDbClient() ? "" : `<p class="muted" style="margin-top:8px">No SQL client detected in /Applications (Sequel Ace, TablePlus, DBeaver, Beekeeper Studio, DbGate, Querious). Install one and it appears here.</p>`}
  </div>
  <div id="import-panel"></div>
  <h2>Snapshots</h2>
  <div class="row" style="margin-bottom:10px"><input class="text" id="snap-label" placeholder="label (optional)"><button class="btn primary" id="snap-create">Take snapshot</button><button class="btn" id="snap-prune">Keep newest 5</button></div>
  <div id="snap-list">Loading…</div>`;
  wireCommonActions(body);
  const list = async () => {
    const snaps = (await call<Snapshot[]>("snapshots", { project: p.name })) ?? [];
    const holder = $("#snap-list", body); if (!holder) return;
    holder.innerHTML = snaps.length
      ? `<table><thead><tr><th>Snapshot</th><th>Size</th><th>Taken</th><th></th></tr></thead><tbody>${snaps.map((s) => `<tr>
          <td>${esc(s.name)}</td><td class="num">${fmtSize(s.size)}</td><td class="num">${fmtDate(s.modified)}</td>
          <td class="actions"><button class="btn small" data-restore="${esc(s.file)}">Restore</button><button class="btn small" data-delete="${esc(s.file)}">Delete</button></td></tr>`).join("")}</tbody></table>`
      : `<div class="card empty">No snapshots yet. A snapshot is a compressed dump in backups/snapshots/${esc(db)}/.</div>`;
    body.querySelectorAll<HTMLButtonElement>("[data-restore]").forEach((b) => b.addEventListener("click", async () => {
      if (b.dataset.confirm !== "1") { b.dataset.confirm = "1"; b.textContent = "Really restore? (takes a safety snapshot first)"; b.classList.add("primary"); return; }
      b.disabled = true; toast("Restoring…");
      const out = await call<string>("snapshot_action", { project: p.name, action: "restore", arg: b.dataset.restore });
      if (out !== undefined) toast("Database restored");
      list();
    }));
    body.querySelectorAll<HTMLButtonElement>("[data-delete]").forEach((b) => b.addEventListener("click", async () => {
      if (b.dataset.confirm !== "1") { b.dataset.confirm = "1"; b.textContent = "Really delete?"; return; }
      await call("snapshot_action", { project: p.name, action: "delete", arg: b.dataset.delete }); list();
    }));
  };
  $("#snap-create", body).addEventListener("click", async (ev) => {
    const b = ev.currentTarget as HTMLButtonElement; b.disabled = true;
    const out = await call<string>("snapshot_action", { project: p.name, action: "create", arg: $<HTMLInputElement>("#snap-label", body).value.trim() || null });
    b.disabled = false; if (out !== undefined) { toast("Snapshot taken"); list(); }
  });
  $("#snap-prune", body).addEventListener("click", async () => { await call("snapshot_action", { project: p.name, action: "prune", arg: "5" }); list(); });
  list();
}

async function renderDetailWorkers(body: HTMLElement, p: Project) {
  const isLaravel = p.kind === "laravel" || p.kind === "statamic";
  const workers = isLaravel ? (await call<Worker[]>("workers_status", { project: p.name })) ?? [] : [];
  const dev = (await call<DevStatus>("dev_server_status", { project: p.name })) ?? { running: false, url: "" };
  const httpUrl = p.url.replace(/^https:/, "http:");
  body.innerHTML = `<div class="two">
    <div class="card"><h3>Workers <span class="muted">(inside the PHP container)</span></h3>
      ${isLaravel ? `<table><tbody>${workers.map((w) => `<tr><td>${esc(w.name)}</td><td><span class="badge ${w.running ? "ok" : "muted"}">${w.running ? "running · pid " + esc(w.pid) : "stopped"}</span></td>
        <td class="actions"><button class="btn small" data-worker="${esc(w.name)}" data-act="${w.running ? "stop" : "start"}">${w.running ? "Stop" : "Start"}</button><button class="btn small" data-wlogs="${esc(w.name)}">Log</button></td></tr>`).join("")}</tbody></table>
        <p class="muted" style="margin-top:8px">queue: <code>artisan queue:work</code> · schedule: <code>artisan schedule:work</code> · horizon / reverb when installed. Output in storage/logs/worker-*.log.</p>`
        : `<p class="muted">Workers are available for Laravel and Statamic projects.</p>`}
    </div>
    <div class="card"><h3>Vite dev server <span class="muted">(npm run dev on this Mac)</span></h3>
      <p><span class="badge ${dev.running ? "ok" : "muted"}">${dev.running ? "running" : "stopped"}</span> ${dev.url ? `· hot file points at ${esc(dev.url)}` : ""}</p>
      <p class="muted">The dev server speaks plain http, so browse the site over <a class="link" data-open="${esc(httpUrl)}">${esc(httpUrl)}</a> while it runs to avoid mixed-content blocking. Stop it to serve the built assets again.</p>
      <div class="actions"><button class="btn ${dev.running ? "" : "primary"}" id="dev-toggle">${dev.running ? "Stop dev server" : "Start dev server"}</button>${dev.running ? `<button class="btn" data-open="${esc(httpUrl)}">Open http site</button>` : ""}</div>
    </div>
  </div>
  <div id="worker-output">${dev.running ? jobPane(`dev-${p.name}`, "Dev server output") : ""}</div>`;
  wireCommonActions(body);
  body.querySelectorAll<HTMLButtonElement>("[data-worker]").forEach((b) => b.addEventListener("click", async () => {
    b.disabled = true;
    const r = await call<Worker[]>("worker_action", { project: p.name, action: b.dataset.act, worker: b.dataset.worker });
    if (r) toast(`${b.dataset.worker} ${b.dataset.act === "start" ? "started" : "stopped"}`);
    renderDetailWorkers(body, p);
  }));
  body.querySelectorAll<HTMLButtonElement>("[data-wlogs]").forEach((b) => b.addEventListener("click", async () => {
    const text = await call<string>("worker_logs", { project: p.name, worker: b.dataset.wlogs });
    $("#worker-output", body).innerHTML = `<div class="job"><h2>${esc(b.dataset.wlogs!)} log</h2><pre class="log">${esc(text ?? "")}</pre></div>`;
  }));
  $("#dev-toggle", body).addEventListener("click", async () => {
    if (dev.running) { await call("dev_server_stop", { project: p.name }); toast("Dev server stopped"); }
    else { jobBuffers.set(`dev-${p.name}`, []); const r = await call<DevStatus>("dev_server_start", { project: p.name }); if (r) toast("Dev server starting…"); }
    renderDetailWorkers(body, p);
  });
}

async function renderDetailMedia(body: HTMLElement, p: Project) {
  const current = (await call<string>("media_status", { project: p.name })) ?? "off";
  const on = current !== "off";
  body.innerHTML = `<div class="card"><h3>Production media fallback <span class="badge ${on ? "ok" : "muted"}">${on ? "on" : "off"}</span></h3>
    <p class="muted">Uploads are usually not in git. With this on, any <code>/wp-content/uploads/…</code> file missing locally is redirected to the same path on production, so images keep working. It also lets the vhost answer for <code>*.${esc(p.name)}.${esc(env?.tld ?? "loc")}</code> (multisite). Writes <code>.devilbox/nginx.yml</code> and restarts the web server.</p>
    ${on ? `<p>Currently redirecting to <strong>${esc(current)}</strong></p>` : ""}
    <div class="field"><label>Production URL</label><input class="text" id="media-url" value="${esc(on ? current : "")}" placeholder="https://www.example.com" style="min-width:380px"></div>
    <label class="row"><input type="checkbox" id="media-subdir"> Subdirectory multisite (adds the /site/wp-admin rewrites)</label>
    <div class="actions" style="margin-top:12px"><button class="btn primary" id="media-on">${on ? "Update" : "Enable"}</button>${on ? `<button class="btn" id="media-off">Disable</button>` : ""}</div>
  </div><div id="media-output"></div>`;
  $("#media-on", body).addEventListener("click", async () => {
    const url = $<HTMLInputElement>("#media-url", body).value.trim();
    if (!/^https?:\/\//.test(url)) return toast("Enter the production URL including https://", true);
    $("#media-output", body).innerHTML = jobPane(`media-${p.name}`, "Applying");
    const r = await call<string>("media_set", { project: p.name, url, subdir: $<HTMLInputElement>("#media-subdir", body).checked });
    if (r !== undefined) { toast("Media fallback enabled"); renderDetailMedia(body, p); }
  });
  $("#media-off", body)?.addEventListener("click", async () => {
    const r = await call<string>("media_set", { project: p.name, url: "--off", subdir: false });
    if (r !== undefined) { toast("Media fallback disabled"); renderDetailMedia(body, p); }
  });
}

let logsTimer: number | undefined;
async function renderDetailLogs(body: HTMLElement, p: Project) {
  window.clearInterval(logsTimer);
  body.innerHTML = `<div class="row" style="margin-bottom:10px">
      <select class="text" id="logs-which"><option value="all">Everything</option><option value="error">Web server + PHP errors</option><option value="app">Application log</option><option value="access">Recent requests</option></select>
      <label class="row"><input type="checkbox" id="logs-live" checked> Live (refresh every 4s)</label>
      <button class="btn" id="logs-refresh">Refresh</button>
    </div><pre class="log" id="logs-pre" style="max-height:600px">Loading…</pre>`;
  const load = async () => {
    const which = $<HTMLSelectElement>("#logs-which", body)?.value ?? "all";
    const text = await call<string>("project_logs", { project: p.name, lines: 150, which });
    const pre = $("#logs-pre", body); if (!pre) return;
    const atBottom = pre.scrollTop + pre.clientHeight >= pre.scrollHeight - 20;
    pre.textContent = text?.trim() || "(nothing logged yet)";
    if (atBottom) pre.scrollTop = pre.scrollHeight;
  };
  $("#logs-refresh", body).addEventListener("click", load);
  $("#logs-which", body).addEventListener("change", load);
  const arm = () => { window.clearInterval(logsTimer); if ($<HTMLInputElement>("#logs-live", body).checked) logsTimer = window.setInterval(() => { if (document.contains(body) && detailTab === "logs") load(); else window.clearInterval(logsTimer); }, 4000); };
  $("#logs-live", body).addEventListener("change", arm);
  await load(); arm();
}

// ----- terminal ---------------------------------------------------------------------------------------
let term: Terminal | null = null;
let termId: number | null = null;
let termUnlisten: (() => void)[] = [];
let termObserver: ResizeObserver | null = null;

async function closeTerminal() {
  termUnlisten.forEach((u) => u()); termUnlisten = [];
  termObserver?.disconnect(); termObserver = null;
  if (termId !== null) { const id = termId; termId = null; await call("terminal_close", { id }); }
  term?.dispose(); term = null;
}

async function renderDetailTerminal(body: HTMLElement, p: Project) {
  await closeTerminal();
  body.innerHTML = `<p class="muted" style="margin:0 0 8px">bash inside the PHP container as <code>devilbox</code>, in <code>/shared/httpd/${esc(p.name)}</code>. composer, artisan, wp, npm, bun and friends are all here.</p>
    <div class="term" id="term"></div>
    <div class="row" style="margin-top:8px"><button class="btn small" id="term-restart">Restart shell</button></div>`;
  term = new Terminal({ cursorBlink: true, fontSize: 13, fontFamily: "SF Mono, Menlo, Monaco, monospace", theme: { background: "#0f1218", foreground: "#d7dce5", cursor: "#ff6a2b" } });
  const fit = new FitAddon();
  term.loadAddon(fit);
  term.open($("#term", body));
  fit.fit();
  const id = await call<number>("terminal_open", { project: p.name, cols: term.cols, rows: term.rows });
  if (id === undefined) return;
  termId = id;
  termUnlisten.push(await listen<{ id: number; data: string }>("term-data", (e) => { if (e.payload.id === id) term?.write(e.payload.data); }));
  termUnlisten.push(await listen<{ id: number }>("term-exit", (e) => { if (e.payload.id === id) term?.write("\r\n\x1b[2m[shell exited]\x1b[0m\r\n"); }));
  term.onData((data) => { if (termId !== null) invoke("terminal_write", { id: termId, data }).catch(() => {}); });
  term.onResize(({ cols, rows }) => { if (termId !== null) invoke("terminal_resize", { id: termId, cols, rows }).catch(() => {}); });
  termObserver = new ResizeObserver(() => fit.fit());
  termObserver.observe($("#term", body));
  term.focus();
  $("#term-restart", body).addEventListener("click", () => renderDetailTerminal(body, p));
}

// ----- dump import --------------------------------------------------------------------------------------
async function startImport(panel: HTMLElement, project: string | null, db: string) {
  const file = await call<string | null>("pick_dump");
  if (!file) return;
  const isWp = project ? projectList.find((p) => p.name === project)?.kind === "wordpress" : false;
  panel.innerHTML = `<div class="card" style="margin-top:14px">
    <h3>Import dump${project ? ` into ${esc(project)}` : ""}</h3>
    <p>${esc(file)}</p>
    ${isWp
      ? `<p>WordPress project: the import tool detects the table prefix and production URL, writes or updates wp-config.php and pins the site to its local https URL.</p>`
      : `<div class="field"><label>Target database</label><input class="text" id="import-db" value="${esc(db)}"></div>
         <label class="row"><input type="checkbox" id="import-fresh" checked> Drop and recreate the database first</label>`}
    <div class="actions" style="margin-top:12px"><button class="btn primary" id="import-run">Run import</button><button class="btn" id="import-cancel">Cancel</button></div>
    <pre class="log" id="import-log" hidden></pre></div>`;
  panel.scrollIntoView({ behavior: "smooth", block: "nearest" });
  $("#import-cancel", panel).addEventListener("click", () => (panel.innerHTML = ""));
  $<HTMLButtonElement>("#import-run", panel).addEventListener("click", async (ev) => {
    const btn = ev.currentTarget as HTMLButtonElement;
    const targetDb = isWp ? db : ($<HTMLInputElement>("#import-db", panel)?.value.trim() || db);
    const fresh = isWp ? true : !!$<HTMLInputElement>("#import-fresh", panel)?.checked;
    btn.disabled = true; btn.textContent = "Importing…";
    const log = $<HTMLPreElement>("#import-log", panel); log.hidden = false; log.textContent = "Running… large dumps can take a few minutes.";
    try {
      const out = await invoke<string>("import_dump", { project, db: targetDb, file, fresh });
      log.textContent = out || "(no output)"; toast("Import finished");
    } catch (e) { log.textContent = String(e); toast("Import failed, see log", true); }
    finally { btn.disabled = false; btn.textContent = "Run import"; }
  });
}

// ----- databases ---------------------------------------------------------------------------------------
async function loadDatabases() {
  const v = $("#view-databases");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  v.innerHTML = `<h1>Databases</h1><p class="sub">Loading…</p>`;
  const dbs = await call<string[]>("databases");
  if (!dbs) { v.innerHTML = `<h1>Databases</h1><div class="card empty">Could not list databases. Is the stack running?</div>`; return; }
  const owners = new Map(projectList.map((p) => [p.db, p.name]));
  v.innerHTML = `<h1>Databases ${helpButton("databases")}</h1>
    <p class="sub">MariaDB on 127.0.0.1:${esc(env.mysql_host_port)} as root.${hasDbClient() ? ` Click a name to open it in ${esc(settings.db_client)}.` : " No desktop SQL client detected; Adminer and phpMyAdmin are built in."}</p>
    <div class="row" style="margin-bottom:12px">${dbButtons("", false)}<button class="btn" data-import-db="">Import dump…</button><button class="btn" id="db-reload">Reload</button></div>
    <div id="import-panel"></div>
    <table><thead><tr><th>Database</th><th>Project</th><th></th></tr></thead><tbody>
    ${dbs.map((d) => `<tr>
      <td>${hasDbClient() ? `<a class="link" data-db="${esc(d)}">${esc(d)}</a>` : esc(d)}</td>
      <td>${owners.get(d) ? `<a class="link" data-project="${esc(owners.get(d)!)}">${esc(owners.get(d)!)}</a>` : ""}</td>
      <td class="actions">${dbButtons(d)}</td></tr>`).join("")}
    </tbody></table>`;
  $("#db-reload").addEventListener("click", loadDatabases);
  wireCommonActions(v);
}

// ----- tools ----------------------------------------------------------------------------------------------
function renderTools() {
  const v = $("#view-tools");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  const ui = env.ui_url;
  const svcUrl = (names: string[], target: number) => { for (const n of names) { const p = status.find((s) => s.service === n)?.ports.find((x) => x.target === target); if (p) return `http://localhost:${p.published}`; } return null; };
  const tools = [
    { label: "intranet", title: "Devilbox intranet", url: ui, note: "The classic dashboard, in its own window" },
    { label: "phpmyadmin", title: "phpMyAdmin", url: `${ui}/vendor/phpmyadmin-5.2.3/index.php`, note: "MariaDB / MySQL" },
    { label: "adminer", title: "Adminer", url: `${ui}/vendor/adminer-6.1.2-devilbox.php?username=root`, note: "MariaDB, PostgreSQL, SQLite" },
    { label: "phppgadmin", title: "phpPgAdmin", url: `${ui}/vendor/phppgadmin-7.13.0/`, note: "PostgreSQL" },
    { label: "phpcacheadmin", title: "phpCacheAdmin", url: `${ui}/vendor/phpcacheadmin-2.7.2/index.php`, note: "Redis, Memcached, OPcache" },
    { label: "opcache", title: "OPcache GUI", url: `${ui}/vendor/opcache-gui-3.6.1.php`, note: "PHP opcode cache" },
    { label: "mailpit", title: "Mailpit", url: svcUrl(["mailpit"], 8025), note: "Caught outgoing mail" },
    { label: "rabbitmq", title: "RabbitMQ", url: svcUrl(["rabbitmq", "rabbit"], 15672), note: "Management console" },
    { label: "meilisearch", title: "Meilisearch", url: svcUrl(["meilisearch"], 7700), note: "Search engine" },
    { label: "rustfs", title: "RustFS console", url: svcUrl(["rustfs", "minio"], 9001), note: "S3 storage" },
  ];
  v.innerHTML = `<h1>Tools ${helpButton("tools")}</h1><p class="sub">Each tool opens in its own window inside the app, so nothing lands in your browser tabs.</p>
    <div class="grid">${tools.map((t) => `<div class="card">
      <h3>${esc(t.title)} ${t.url ? "" : `<span class="badge muted">not running</span>`}</h3><p>${esc(t.note)}</p>
      <div class="actions">
        <button class="btn small" ${t.url ? "" : "disabled"} data-tool="${esc(t.label)}" data-title="${esc(t.title)}" data-url="${esc(t.url ?? "")}">Open window</button>
        <button class="btn small" ${t.url ? "" : "disabled"} data-open="${esc(t.url ?? "")}">Browser</button>
      </div></div>`).join("")}</div>`;
  v.querySelectorAll<HTMLButtonElement>("[data-tool]").forEach((b) => b.addEventListener("click", () => call("open_tool", { label: b.dataset.tool, title: b.dataset.title, url: b.dataset.url })));
  wireCommonActions(v);
}

// ----- doctor ---------------------------------------------------------------------------------------------
function renderFindings(findings: Finding[]) {
  const counts: Record<string, number> = { OK: 0, WARN: 0, FAIL: 0 };
  findings.forEach((f) => (counts[f.level] = (counts[f.level] ?? 0) + 1));
  const cls = (l: string) => (l === "OK" ? "ok" : l === "WARN" ? "warn" : "bad");
  const order = ["FAIL", "WARN", "OK"];
  const sorted = [...findings].sort((a, b) => order.indexOf(a.level) - order.indexOf(b.level));
  return `<div class="row" style="margin:10px 0"><span class="badge bad">${counts.FAIL} failing</span><span class="badge warn">${counts.WARN} warnings</span><span class="badge ok">${counts.OK} ok</span></div>
    <div class="card" style="padding:0">${sorted.map((f) => `<div class="finding"><span class="badge ${cls(f.level)}">${esc(f.level)}</span><span class="scope">${f.scope === "stack" ? "stack" : `<a class="link" data-project="${esc(f.scope)}">${esc(f.scope)}</a>`}</span><span>${esc(f.message)}</span></div>`).join("") || `<div class="empty">Nothing to report.</div>`}</div>`;
}

async function renderDoctor() {
  const v = $("#view-doctor");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  v.innerHTML = `<h1>Doctor ${helpButton("doctor")}</h1>
    <p class="sub">Checks Docker, the DNS resolver, CA trust, the intranet and check-config, then every site: reachability with the matching error-log line, databases, active plugins missing on disk, incomplete plugins, fatal errors, missing uploads and pending migrations. Same as <code>dbox doctor</code>.</p>
    <div class="row" style="margin-bottom:10px"><button class="btn primary" id="doc-run">Run full check</button><button class="btn" id="doc-stack">Stack only</button></div>
    <div id="doc-result"></div>`;
  const run = async (stackOnly: boolean) => {
    jobBuffers.set("doctor", []);
    $("#doc-result").innerHTML = jobPane("doctor", stackOnly ? "Checking the stack…" : "Checking the stack and all sites… (one probe per site)");
    v.querySelectorAll<HTMLButtonElement>("button").forEach((b) => (b.disabled = true));
    const findings = await call<Finding[]>("doctor", { project: stackOnly ? "--stack" : null });
    v.querySelectorAll<HTMLButtonElement>("button").forEach((b) => (b.disabled = false));
    if (findings) { $("#doc-result").innerHTML = renderFindings(findings); wireCommonActions($("#doc-result")); }
  };
  $("#doc-run").addEventListener("click", () => run(false));
  $("#doc-stack").addEventListener("click", () => run(true));
  wireHelp(v);
}

// ----- updates -------------------------------------------------------------------------------------------------
async function renderUpdates() {
  const v = $("#view-updates");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  if (!projectList.length) await loadProjectList();
  const sites = projectList.filter((p) => UPDATABLE.includes(p.kind));
  const summary = (p: Project) => {
    const i = updateInfo.get(p.name);
    if (!i) return `<span class="kind">not checked</span>`;
    if (i.error) return `<span class="badge bad" title="${esc(i.error)}">error</span>`;
    const parts: string[] = [];
    if (i.plugins.length) parts.push(`${i.plugins.length} plugin${i.plugins.length > 1 ? "s" : ""}`);
    if (i.themes.length) parts.push(`${i.themes.length} theme${i.themes.length > 1 ? "s" : ""}`);
    if (i.core) parts.push(`core ${i.core}`);
    if (i.packages.length) parts.push(`${i.packages.length} package${i.packages.length > 1 ? "s" : ""}`);
    const detail = [...i.plugins, ...i.themes, ...i.packages].join("\n");
    return parts.length ? `<span class="badge warn" title="${esc(detail)}">${esc(parts.join(", "))}</span>` : `<span class="badge ok">up to date</span>`;
  };
  v.innerHTML = `<h1>Updates ${helpButton("updates")}</h1>
    <p class="sub">WordPress: plugins, themes, core and database upgrade via wp-cli. Composer projects (Laravel, Statamic, Symfony, Drupal, Craft …): composer update, migrations and a frontend build. Same as <code>dbox update</code>.</p>
    <div class="row" style="margin-bottom:12px">
      <button class="btn" id="upd-check-all">Check all</button>
      <button class="btn primary" data-bulk="wordpress">Update all WordPress</button>
      <button class="btn primary" data-bulk="php">Update all Composer projects</button>
      <button class="btn" data-bulk="all">Update everything</button>
    </div>
    <div class="card" style="margin-bottom:14px">
      <h3>Schedule</h3>
      <p>Runs while the app is open (it lives in the menu bar). A desktop notification reports the result.${settings.schedule_last_run ? ` Last run: ${esc(settings.schedule_last_run)}.` : ""}</p>
      <div class="row" style="margin-top:8px">
        <label class="row"><input type="checkbox" id="sch-enabled" ${settings.schedule_enabled ? "checked" : ""}> Enabled</label>
        <label class="row">Daily at <input class="text" id="sch-time" type="time" value="${esc(settings.schedule_time)}" style="min-width:130px"></label>
        <select class="text" id="sch-scope" style="min-width:220px">
          <option value="wordpress" ${settings.schedule_scope === "wordpress" ? "selected" : ""}>WordPress sites</option>
          <option value="php" ${settings.schedule_scope === "php" ? "selected" : ""}>Composer projects</option>
          <option value="all" ${settings.schedule_scope === "all" ? "selected" : ""}>Everything</option>
        </select>
        <button class="btn" id="sch-save">Save schedule</button>
      </div>
    </div>
    <table><thead><tr><th>Site</th><th>Type</th><th>Available</th><th></th></tr></thead><tbody>
    ${sites.map((p) => `<tr>
      <td><a class="link" data-project="${esc(p.name)}">${esc(p.name)}</a></td>
      <td><span class="kind">${esc(p.kind)}</span></td>
      <td id="upd-${esc(p.name)}">${summary(p)}</td>
      <td class="actions"><button class="btn small" data-check="${esc(p.name)}">Check</button><button class="btn small" data-update="${esc(p.name)}">Update</button></td></tr>`).join("")}
    </tbody></table>
    <div id="upd-output"></div>`;
  wireCommonActions(v);
  const check = async (name: string) => {
    const cell = v.querySelector(`#upd-${CSS.escape(name)}`);
    if (cell) cell.innerHTML = `<span class="kind">checking…</span>`;
    const info = await call<UpdateInfo>("check_updates", { project: name });
    if (info) updateInfo.set(name, info);
    const p = projectList.find((x) => x.name === name);
    if (cell && p) cell.innerHTML = summary(p);
  };
  v.querySelectorAll<HTMLButtonElement>("[data-check]").forEach((b) => b.addEventListener("click", () => check(b.dataset.check!)));
  $("#upd-check-all").addEventListener("click", async () => { for (const p of sites) await check(p.name); });
  v.querySelectorAll<HTMLButtonElement>("[data-update]").forEach((b) => b.addEventListener("click", async () => {
    const name = b.dataset.update!; const job = `update-${name}`;
    jobBuffers.set(job, []);
    $("#upd-output").innerHTML = jobPane(job, `Updating ${name}`);
    b.disabled = true;
    const r = await call<BulkResult>("update_site", { project: name });
    b.disabled = false;
    if (r) { toast(r.ok ? `${name} updated` : `${name}: ${r.message}`, !r.ok); updateInfo.delete(name); check(name); }
  }));
  v.querySelectorAll<HTMLButtonElement>("[data-bulk]").forEach((b) => b.addEventListener("click", async () => {
    const scope = b.dataset.bulk!; const job = "update-all";
    jobBuffers.set(job, []);
    $("#upd-output").innerHTML = jobPane(job, `Updating all (${scope})`);
    v.querySelectorAll<HTMLButtonElement>("[data-bulk]").forEach((x) => (x.disabled = true));
    const results = await call<BulkResult[]>("update_all", { scope });
    v.querySelectorAll<HTMLButtonElement>("[data-bulk]").forEach((x) => (x.disabled = false));
    if (results) {
      const failed = results.filter((r) => !r.ok);
      toast(failed.length ? `${results.length - failed.length} updated, ${failed.length} failed: ${failed.map((f) => f.project).join(", ")}` : `${results.length} sites updated`, failed.length > 0);
      updateInfo.clear();
    }
  }));
  $("#sch-save").addEventListener("click", async () => {
    const next: Settings = { ...settings, schedule_enabled: $<HTMLInputElement>("#sch-enabled").checked, schedule_time: $<HTMLInputElement>("#sch-time").value || "03:00", schedule_scope: $<HTMLSelectElement>("#sch-scope").value };
    const saved = await call<Settings>("save_settings_cmd", { settings: next });
    if (saved) { settings = saved; toast("Schedule saved"); }
  });
}

// ----- new project -----------------------------------------------------------------------------------------------
async function renderCreate() {
  const v = $("#view-create");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  const types = (await call<ProjectType[]>("project_types")) ?? [];
  const presets = (await call<Preset[]>("presets")) ?? [];
  const needsAdmin = (k: string) => ["wordpress", "bedrock", "drupal", "craft"].includes(k);
  const hasTitle = (k: string) => needsAdmin(k) || k === "static";
  v.innerHTML = `<h1>New project ${helpButton("create")}</h1>
    <p class="sub">Created inside the PHP container with the stack's PHP, Composer, wp-cli and npm. Database, local config, keys, migrations and a frontend build are set up for you. Same as <code>dbox new</code>.</p>
    <div class="card">
      <div class="field"><label>Type</label>
        <select class="text" id="np-kind" style="min-width:520px">${types.map((t) => `<option value="${esc(t.id)}">${esc(t.id)} — ${esc(t.description)}</option>`).join("")}</select></div>
      <div class="field"><label>Name (becomes <code>https://&lt;name&gt;.${esc(env.tld)}</code> and database <code>&lt;name&gt;</code>)</label>
        <input class="text" id="np-name" placeholder="my-new-app" style="min-width:320px"></div>
      <div class="field" id="np-preset-field"><label>Preset (optional post-install steps, see tools/presets)</label>
        <select class="text" id="np-preset" style="min-width:520px"></select></div>
      <div id="np-title-field" class="field" hidden><label>Site title</label><input class="text" id="np-title" placeholder="My Site" style="min-width:320px"></div>
      <div id="np-admin" class="row" hidden>
        <div class="field"><label>Admin user</label><input class="text" id="np-user" value="admin"></div>
        <div class="field"><label>Admin password</label><input class="text" id="np-pass" value="admin"></div>
        <div class="field"><label>Admin email</label><input class="text" id="np-mail" placeholder="admin@site.${esc(env.tld)}" style="min-width:260px"></div>
      </div>
      <div class="row"><button class="btn primary" id="np-create">Create project</button></div>
    </div>
    <div id="np-output"></div>`;
  wireHelp(v);
  const kind = $<HTMLSelectElement>("#np-kind");
  const presetSel = $<HTMLSelectElement>("#np-preset");
  const sync = () => {
    const k = kind.value;
    $("#np-title-field").hidden = !hasTitle(k);
    $("#np-admin").hidden = !needsAdmin(k);
    const base = k.split("-")[0];
    const applicable = presets.filter((p) => p.kind === "any" || p.kind === k || p.kind === base);
    presetSel.innerHTML = `<option value="">None</option>` + applicable.map((p) => `<option value="${esc(p.name)}">${esc(p.name)} — ${esc(p.description)}</option>`).join("");
    $("#np-preset-field").hidden = applicable.length === 0;
  };
  kind.addEventListener("change", sync); sync();
  $("#np-create").addEventListener("click", async (ev) => {
    const btn = ev.currentTarget as HTMLButtonElement;
    const name = $<HTMLInputElement>("#np-name").value.trim().toLowerCase();
    if (!/^[a-z0-9][a-z0-9-]*$/.test(name)) return toast("Name must be lowercase letters, digits and dashes", true);
    const spec = { name, kind: kind.value, title: $<HTMLInputElement>("#np-title").value.trim(), admin_user: $<HTMLInputElement>("#np-user").value.trim(), admin_password: $<HTMLInputElement>("#np-pass").value, admin_email: $<HTMLInputElement>("#np-mail").value.trim(), preset: presetSel.value };
    const job = `create-${name}`; jobBuffers.set(job, []);
    $("#np-output").innerHTML = jobPane(job, `Creating ${name}`);
    btn.disabled = true; btn.textContent = "Creating…";
    const url = await call<string>("create_site", { spec });
    btn.disabled = false; btn.textContent = "Create project";
    if (url) {
      toast(`Created ${name}`);
      await loadProjectList();
      $("#np-output").insertAdjacentHTML("afterbegin", `<div class="card" style="margin-top:14px"><h3>Ready</h3><p><a class="link" data-open="${esc(url)}">${esc(url)}</a> · <a class="link" data-project="${esc(name)}">open project details</a></p></div>`);
      wireCommonActions($("#np-output"));
    }
  });
}

// ----- files -----------------------------------------------------------------------------------------------------
let filesPath = "";
let filesExtract = true;
let dropHooked = false;

async function renderFiles(path?: string) {
  const v = $("#view-files");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  if (path !== undefined) filesPath = path;
  if (!filesPath) filesPath = `${env.root}/data/www`;
  const listing = await call<Listing>("list_dir", { path: filesPath });
  if (!listing) { if (filesPath !== `${env.root}/data/www`) { filesPath = `${env.root}/data/www`; return renderFiles(); } return; }
  filesPath = listing.path;
  const base = `${env.root}/data/www`;
  const parts = listing.relative ? listing.relative.split("/") : [];
  const crumbs = [`<a data-go="${esc(base)}">data/www</a>`].concat(parts.map((seg, i) => `<span class="sep">/</span><a data-go="${esc(base + "/" + parts.slice(0, i + 1).join("/"))}">${esc(seg)}</a>`)).join(" ");
  v.innerHTML = `<h1>Files ${helpButton("files")}</h1>
    <p class="sub">Browse project folders and drop files or folders onto the zone below to copy them in. Handy for <code>wp-content/uploads</code>.</p>
    <div class="row">
      <select class="text" id="files-project"><option value="">Jump to project…</option>${projectList.map((p) => `<option value="${esc(p.path)}" ${filesPath.startsWith(p.path) ? "selected" : ""}>${esc(p.name)}</option>`).join("")}</select>
      <button class="btn" id="files-up" ${listing.parent ? "" : "disabled"}>Up</button>
      <button class="btn" id="files-finder">Open in ${esc(fileManager)}</button>
      <button class="btn" id="files-editor">Open in editor</button>
      <button class="btn" id="files-newdir">New folder…</button>
      <button class="btn" id="files-add">Add files…</button>
      <button class="btn" id="files-reload">Reload</button>
    </div>
    <div class="crumbs">${crumbs}</div>
    <div class="dropzone" id="dropzone">Drop files or folders here to copy them into <strong>${esc(listing.relative || "data/www")}</strong><br>
      <label class="row" style="justify-content:center;margin-top:8px"><input type="checkbox" id="files-extract" ${filesExtract ? "checked" : ""}> Extract .zip / .tar.gz archives instead of copying them</label></div>
    <div id="files-newdir-form"></div>
    <table><thead><tr><th>Name</th><th>Size</th><th>Modified</th></tr></thead><tbody>
    ${listing.entries.map((e) => `<tr>
      <td>${e.is_dir ? `<a class="link" data-go="${esc(e.path)}">${esc(e.name)}/</a>` : esc(e.name)}${e.is_symlink ? ` <span class="kind">link</span>` : ""}</td>
      <td class="num">${e.is_dir ? "" : fmtSize(e.size)}</td><td class="num">${fmtDate(e.modified)}</td></tr>`).join("")}
    </tbody></table>
    ${listing.entries.length ? "" : `<div class="empty">Empty folder.</div>`}
    <div id="files-report"></div>`;
  wireHelp(v);
  v.querySelectorAll<HTMLElement>("[data-go]").forEach((a) => a.addEventListener("click", () => renderFiles(a.dataset.go!)));
  $<HTMLSelectElement>("#files-project").addEventListener("change", (ev) => { const p = (ev.target as HTMLSelectElement).value; if (p) renderFiles(p); });
  $("#files-up").addEventListener("click", () => listing.parent && renderFiles(listing.parent));
  $("#files-finder").addEventListener("click", () => call("open_with", { appName: "Finder", target: filesPath }));
  $("#files-editor").addEventListener("click", () => call("open_with", { appName: settings.editor, target: filesPath }));
  $("#files-reload").addEventListener("click", () => renderFiles());
  $<HTMLInputElement>("#files-extract").addEventListener("change", (ev) => (filesExtract = (ev.target as HTMLInputElement).checked));
  $("#files-newdir").addEventListener("click", () => {
    const f = $("#files-newdir-form");
    f.innerHTML = `<div class="row" style="margin:8px 0"><input class="text" id="newdir-name" placeholder="folder name"><button class="btn primary" id="newdir-ok">Create</button><button class="btn" id="newdir-cancel">Cancel</button></div>`;
    $("#newdir-cancel").addEventListener("click", () => (f.innerHTML = ""));
    $("#newdir-ok").addEventListener("click", async () => { const r = await call<string>("make_dir", { path: filesPath, name: $<HTMLInputElement>("#newdir-name").value }); if (r) renderFiles(); });
    $("#newdir-name").focus();
  });
  $("#files-add").addEventListener("click", async () => { const picked = await call<string[]>("pick_files"); if (picked?.length) uploadInto(picked); });
  if (!dropHooked) {
    dropHooked = true;
    getCurrentWebview().onDragDropEvent((event) => {
      const zone = document.querySelector("#dropzone");
      if (!$("#view-files").classList.contains("active") || !zone) return;
      if (event.payload.type === "enter" || event.payload.type === "over") zone.classList.add("over");
      else if (event.payload.type === "leave") zone.classList.remove("over");
      else if (event.payload.type === "drop") { zone.classList.remove("over"); uploadInto(event.payload.paths); }
    });
  }
}

async function uploadInto(sources: string[]) {
  const report = $("#files-report");
  const plan = (await call<PlanItem[]>("upload_plan", { dest: filesPath, sources, extractArchives: filesExtract })) ?? [];
  if (!plan.length) return;
  const verb: Record<string, string> = { copy: "copy", merge: "merge into", extract: "extract into", "extract-merge": "unpack into", replace: "replace", skip: "skip" };
  report.innerHTML = `<div class="card" style="margin-top:12px"><h3>Copy ${plan.length} item(s) into <code>${esc(filesPath.replace(env!.root + "/data/www", "data/www"))}</code></h3>
    <table><tbody>${plan.map((i) => `<tr><td>${esc(i.name)}</td><td><span class="badge ${i.action === "skip" ? "bad" : i.action === "copy" ? "ok" : "warn"}">${esc(verb[i.action] ?? i.action)}</span></td><td class="muted">${esc(i.target.replace(env!.root + "/data/www/", ""))}${i.note ? ` — ${esc(i.note)}` : ""}</td></tr>`).join("")}</tbody></table>
    <div class="actions" style="margin-top:10px"><button class="btn primary" id="upload-go">Copy now</button><button class="btn" id="upload-cancel">Cancel</button></div></div>`;
  report.scrollIntoView({ behavior: "smooth", block: "nearest" });
  $("#upload-cancel", report).addEventListener("click", () => (report.innerHTML = ""));
  $("#upload-go", report).addEventListener("click", () => performUpload(sources));
}

async function performUpload(sources: string[]) {
  const report = $("#files-report");
  report.innerHTML = `<div class="card" style="margin-top:12px"><p>Copying ${sources.length} item(s)…</p></div>`;
  const r = await call<UploadReport>("upload_files", { dest: filesPath, sources, extractArchives: filesExtract });
  if (!r) { report.innerHTML = ""; return; }
  toast(`${r.copied} file(s) copied${r.extracted.length ? `, ${r.extracted.length} archive(s) extracted` : ""}${r.errors.length ? `, ${r.errors.length} error(s)` : ""}`, r.errors.length > 0);
  await renderFiles();
  if (r.errors.length) $("#files-report").innerHTML = `<div class="card" style="margin-top:12px"><h3>Problems</h3><pre class="log">${esc(r.errors.join("\n"))}</pre></div>`;
}

// ----- settings ----------------------------------------------------------------------------------------------------
function renderSettings() {
  const v = $("#view-settings");
  const clientOptions = dbClients.length
    ? `<option value="">None (use Adminer / phpMyAdmin)</option>` + dbClients.map((n) => `<option ${settings.db_client === n ? "selected" : ""}>${esc(n)}</option>`).join("")
    : `<option value="">None detected in /Applications</option>`;
  v.innerHTML = `<h1>Settings ${helpButton("settings")}</h1><p class="sub">Stored in your user Application Support folder. The Devilbox folder is also picked up from <code>DEVILBOX_ROOT</code>.</p>
    <div class="card">
      <div class="field"><label>Devilbox Boost folder</label>
        <div class="row"><input class="text" id="s-root" value="${esc(settings.root)}" placeholder="/path/to/Devilbox-Boost" style="min-width:460px"><button class="btn" id="s-pick">Choose…</button></div></div>
      <div class="field"><label>Desktop SQL client (detected: ${dbClients.length ? esc(dbClients.join(", ")) : "none"})</label>
        <select class="text" id="s-dbclient" style="min-width:320px">${clientOptions}</select></div>
      <div class="field"><label>Editor (detected: ${editors.length ? esc(editors.join(", ")) : "none"})</label>
        <div class="row"><select class="text" id="s-editor-sel" style="min-width:260px">${editors.map((n) => `<option ${settings.editor === n ? "selected" : ""}>${esc(n)}</option>`).join("")}<option value="__custom" ${editors.includes(settings.editor) ? "" : "selected"}>Custom app name…</option></select>
        <input class="text" id="s-editor" value="${esc(settings.editor)}" placeholder="exact app name, e.g. PhpStorm" style="min-width:260px" ${editors.includes(settings.editor) ? "hidden" : ""}></div></div>
      <div class="field"><label>Terminal (detected: ${terminals.length ? esc(terminals.join(", ")) : "none"})</label>
        <div class="row"><select class="text" id="s-terminal-sel" style="min-width:260px">${terminals.map((n) => `<option ${settings.terminal === n ? "selected" : ""}>${esc(n)}</option>`).join("")}<option value="__custom" ${terminals.includes(settings.terminal) ? "" : "selected"}>Custom app name…</option></select>
        <input class="text" id="s-terminal" value="${esc(settings.terminal)}" placeholder="exact app name, e.g. Warp" style="min-width:260px" ${terminals.includes(settings.terminal) ? "hidden" : ""}></div></div>
      <div class="field"><label class="row"><input type="checkbox" id="s-notify" ${settings.notifications ? "checked" : ""}> Desktop notifications when a container stops on its own or PHP-FPM workers crash</label></div>
      <div class="row"><button class="btn primary" id="s-save">Save</button><button class="btn" id="s-test-notify">Send test notification</button></div>
      <p class="sub" style="margin:12px 0 0">Global shortcut: <kbd>${esc(env?.shortcut ?? "Cmd+Shift+D")}</kbd> opens the project launcher from anywhere. The same operations are available in the terminal as <code>bin/dbox</code>.</p>
    </div>
    ${env ? `<h2>Environment</h2><div class="card"><p>Intranet: ${esc(env.ui_url)}<br>TLD: .${esc(env.tld)} · PHP ${esc(env.php)} · ${esc(env.httpd)} · ${esc(env.mysql)} · PostgreSQL ${esc(env.pgsql)} · Redis ${esc(env.redis)} · Mongo ${esc(env.mongo)}</p></div>` : ""}`;
  $("#s-pick").addEventListener("click", async () => { const picked = await call<string | null>("pick_root"); if (picked) $<HTMLInputElement>("#s-root").value = picked; });
  for (const [sel, inp] of [["#s-editor-sel", "#s-editor"], ["#s-terminal-sel", "#s-terminal"]] as const) {
    $<HTMLSelectElement>(sel).addEventListener("change", (ev) => { const val = (ev.target as HTMLSelectElement).value; const input = $<HTMLInputElement>(inp); if (val === "__custom") { input.hidden = false; input.focus(); } else { input.hidden = true; input.value = val; } });
  }
  wireHelp(v);
  $("#s-test-notify").addEventListener("click", () => call("test_notification"));
  $("#s-save").addEventListener("click", async () => {
    const next: Settings = { ...settings,
      root: $<HTMLInputElement>("#s-root").value.trim(),
      db_client: $<HTMLSelectElement>("#s-dbclient").value,
      editor: $<HTMLInputElement>("#s-editor").value.trim() || editors[0] || "",
      terminal: $<HTMLInputElement>("#s-terminal").value.trim() || terminals[0] || "Terminal",
      notifications: $<HTMLInputElement>("#s-notify").checked };
    const saved = await call<Settings>("save_settings_cmd", { settings: next });
    if (!saved) return;
    settings = saved; toast("Settings saved");
    await boot(); showView("overview");
  });
}

// ----- boot -----------------------------------------------------------------------------------------------------------
async function boot() {
  settings = (await call<Settings>("get_settings")) ?? settings;
  dbClients = (await call<string[]>("db_clients")) ?? [];
  editors = (await call<string[]>("detect_apps", { kind: "editor" })) ?? [];
  terminals = (await call<string[]>("detect_apps", { kind: "terminal" })) ?? [];
  gitApps = (await call<string[]>("detect_apps", { kind: "git" })) ?? [];
  fileManager = (await call<string>("file_manager_name")) ?? "Finder";
  if ((settings.editor === "Visual Studio Code" || !settings.editor) && editors.length && !editors.includes(settings.editor)) {
    settings = { ...settings, editor: editors[0] };
    await call("save_settings_cmd", { settings });
  }
  if (settings.db_client && !dbClients.includes(settings.db_client)) {
    settings = { ...settings, db_client: dbClients[0] ?? "" };
    await call("save_settings_cmd", { settings });
  } else if (!settings.db_client && dbClients.length) {
    settings = { ...settings, db_client: dbClients[0] };
    await call("save_settings_cmd", { settings });
  }
  try { env = await invoke<Environment>("get_environment"); } catch { env = null; }
  await refreshStatus();
  renderOverview();
  await loadProjectList();
}

void currentProject;
boot();
window.setInterval(refreshStatus, 15000);
