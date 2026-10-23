import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ----- types mirrored from src-tauri/src/lib.rs -----------------------------
interface Settings { root: string; db_client: string; editor: string; terminal: string }
interface Environment {
  root: string; tld: string; scheme: string; vhost_port: string; ui_url: string;
  php: string; httpd: string; mysql: string; pgsql: string; redis: string; mongo: string;
  mysql_root_password: string; mysql_host_port: string;
}
interface PortMap { published: number; target: number; url: string }
interface ServiceStatus { service: string; name: string; state: string; health: string; status: string; image: string; ports: PortMap[] }
interface Project { name: string; kind: string; url: string; path: string; db: string }

// ----- state ------------------------------------------------------------------
let settings: Settings = { root: "", db_client: "Sequel Ace", editor: "Visual Studio Code", terminal: "Terminal" };
let env: Environment | null = null;
let status: ServiceStatus[] = [];
let projectList: Project[] = [];
let projectFilter = "";

const $ = <T extends HTMLElement>(sel: string) => document.querySelector(sel) as T;
const esc = (s: string) => s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c] as string));

function toast(msg: string, error = false) {
  const t = $("#toast");
  t.textContent = msg;
  t.className = "toast" + (error ? " error" : "");
  t.hidden = false;
  window.setTimeout(() => (t.hidden = true), error ? 6000 : 2500);
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    toast(String(e), true);
    return undefined;
  }
}

// ----- navigation ---------------------------------------------------------------
function showView(name: string) {
  document.querySelectorAll<HTMLElement>(".view").forEach((v) => v.classList.toggle("active", v.id === `view-${name}`));
  document.querySelectorAll<HTMLElement>(".nav-item").forEach((b) => b.classList.toggle("active", b.dataset.view === name));
  if (name === "projects") loadProjects();
  if (name === "databases") loadDatabases();
  if (name === "tools") renderTools();
  if (name === "settings") renderSettings();
  if (name === "overview") renderOverview();
}

function showFrame(path: string, button: HTMLElement) {
  if (!env) return toast("Devilbox folder not set", true);
  document.querySelectorAll<HTMLElement>(".view").forEach((v) => v.classList.toggle("active", v.id === "view-frame"));
  document.querySelectorAll<HTMLElement>(".nav-item").forEach((b) => b.classList.toggle("active", b === button));
  const frame = $<HTMLIFrameElement>("#intranet-frame");
  const target = env.ui_url + path;
  if (frame.src !== target) frame.src = target;
}

document.querySelectorAll<HTMLElement>(".nav-item[data-view]").forEach((b) => b.addEventListener("click", () => showView(b.dataset.view!)));
document.querySelectorAll<HTMLElement>(".nav-item[data-frame]").forEach((b) => b.addEventListener("click", () => showFrame(b.dataset.frame!, b)));

// ----- status --------------------------------------------------------------------
function stateClass(s: ServiceStatus) {
  if (s.state !== "running") return "bad";
  if (s.health && s.health !== "healthy" && s.health !== "") return "warn";
  return "ok";
}

function applyStatus(list: ServiceStatus[]) {
  status = list;
  const running = list.filter((s) => s.state === "running").length;
  const dot = $("#brand-dot");
  const sub = $("#brand-sub");
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

// ----- overview ------------------------------------------------------------------
function portUrl(s: ServiceStatus): string | null {
  const web: Record<string, number> = { mailpit: 8025, rabbitmq: 15672, meilisearch: 7700, rustfs: 9001, minio: 9001, httpd: 443 };
  const target = web[s.service];
  if (!target) return null;
  const p = s.ports.find((x) => x.target === target);
  if (!p) return null;
  return `${s.service === "httpd" ? "https" : "http"}://localhost:${p.published}`;
}

function renderOverview() {
  const v = $("#view-overview");
  if (!env) {
    v.innerHTML = `<h1>Welcome</h1><p class="sub">Point the app at your Devilbox Boost folder to get started.</p>
      <button class="btn primary" id="go-settings">Open settings</button>`;
    $("#go-settings").addEventListener("click", () => showView("settings"));
    return;
  }
  const running = status.filter((s) => s.state === "running").length;
  const cards = status.map((s) => {
    const url = portUrl(s);
    const ports = s.ports.map((p) => `${p.published}→${p.target}`).join(", ");
    return `<div class="card">
      <h3>${esc(s.service)} <span class="badge ${stateClass(s)}">${esc(s.health || s.state)}</span></h3>
      <p>${esc(s.image.split("@")[0])}</p>
      <p>${esc(s.status)}${ports ? " · " + esc(ports) : ""}</p>
      <div class="actions">
        ${url ? `<button class="btn small" data-tool="${esc(s.service)}" data-url="${esc(url)}">Open</button>` : ""}
        <button class="btn small" data-action="restart" data-service="${esc(s.service)}">Restart</button>
        <button class="btn small" data-logs="${esc(s.service)}">Logs</button>
      </div></div>`;
  }).join("");
  v.innerHTML = `<h1>Overview</h1>
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
    b.disabled = true;
    toast(`${action} ${service ?? "stack"}…`);
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
  v.querySelectorAll<HTMLButtonElement>("[data-tool]").forEach((b) => b.addEventListener("click", () =>
    call("open_tool", { label: `svc-${b.dataset.tool}`, title: b.dataset.tool, url: b.dataset.url })));
}

// ----- projects --------------------------------------------------------------------
async function loadProjects() {
  const v = $("#view-projects");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  const list = await call<Project[]>("projects");
  if (!list) return;
  projectList = list;
  renderProjects();
}

function renderProjects() {
  const v = $("#view-projects");
  const f = projectFilter.toLowerCase();
  const rows = projectList.filter((p) => !f || p.name.toLowerCase().includes(f) || p.kind.includes(f) || p.db.toLowerCase().includes(f));
  v.innerHTML = `<h1>Projects</h1><p class="sub">${projectList.length} folders in data/www</p>
    <div class="row" style="margin-bottom:12px">
      <input class="search" id="project-search" placeholder="Filter by name, type or database" value="${esc(projectFilter)}">
      <button class="btn" id="projects-reload">Reload</button>
      <button class="btn" id="projects-folder">Open data/www</button>
    </div>
    <table><thead><tr><th>Project</th><th>Type</th><th>Database</th><th></th></tr></thead><tbody>
    ${rows.map((p) => `<tr>
      <td><a class="link" data-open="${esc(p.url)}">${esc(p.name)}</a><div class="kind">${esc(p.url)}</div></td>
      <td><span class="kind">${esc(p.kind)}</span></td>
      <td>${p.db ? `<a class="link" data-db="${esc(p.db)}" title="Open in ${esc(settings.db_client)}">${esc(p.db)}</a>` : `<span class="kind">–</span>`}</td>
      <td class="actions">
        <button class="btn small" data-open="${esc(p.url)}">Site</button>
        <button class="btn small" data-with="editor" data-path="${esc(p.path)}">Editor</button>
        <button class="btn small" data-with="terminal" data-path="${esc(p.path)}">Terminal</button>
        <button class="btn small" data-with="Finder" data-path="${esc(p.path)}">Finder</button>
        ${p.db ? `<button class="btn small" data-adminer="${esc(p.db)}">Adminer</button>` : ""}
      </td></tr>`).join("")}
    </tbody></table>
    ${rows.length ? "" : `<div class="empty">No projects match.</div>`}`;
  const search = $<HTMLInputElement>("#project-search");
  search.addEventListener("input", () => { projectFilter = search.value; const pos = search.selectionStart; renderProjects(); const s2 = $<HTMLInputElement>("#project-search"); s2.focus(); s2.setSelectionRange(pos, pos); });
  $("#projects-reload").addEventListener("click", loadProjects);
  $("#projects-folder").addEventListener("click", () => call("open_with", { appName: "Finder", target: `${env!.root}/data/www` }));
  wireCommonActions(v);
}

function wireCommonActions(root: HTMLElement) {
  root.querySelectorAll<HTMLElement>("[data-open]").forEach((b) => b.addEventListener("click", () => call("open_url", { url: b.dataset.open })));
  root.querySelectorAll<HTMLElement>("[data-db]").forEach((b) => b.addEventListener("click", () => call("open_db_client", { db: b.dataset.db })));
  root.querySelectorAll<HTMLElement>("[data-adminer]").forEach((b) => b.addEventListener("click", () => openAdminer(b.dataset.adminer!)));
  root.querySelectorAll<HTMLElement>("[data-pma]").forEach((b) => b.addEventListener("click", () => openPma(b.dataset.pma!)));
  root.querySelectorAll<HTMLElement>("[data-with]").forEach((b) => b.addEventListener("click", () => {
    const which = b.dataset.with!;
    const app = which === "editor" ? settings.editor : which === "terminal" ? settings.terminal : which;
    call("open_with", { appName: app, target: b.dataset.path });
  }));
}

// ----- databases -------------------------------------------------------------------
function openAdminer(db: string) {
  if (!env) return;
  const url = `${env.ui_url}/vendor/adminer-6.1.2-devilbox.php?username=root${db ? `&db=${encodeURIComponent(db)}` : ""}`;
  call("open_tool", { label: "adminer", title: `Adminer${db ? " · " + db : ""}`, url });
}
function openPma(db: string) {
  if (!env) return;
  const url = `${env.ui_url}/vendor/phpmyadmin-5.2.3/index.php${db ? `?route=/database/structure&db=${encodeURIComponent(db)}` : ""}`;
  call("open_tool", { label: "phpmyadmin", title: `phpMyAdmin${db ? " · " + db : ""}`, url });
}

async function loadDatabases() {
  const v = $("#view-databases");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  v.innerHTML = `<h1>Databases</h1><p class="sub">Loading…</p>`;
  const dbs = await call<string[]>("databases");
  if (!dbs) { v.innerHTML = `<h1>Databases</h1><div class="card empty">Could not list databases. Is the stack running?</div>`; return; }
  const owners = new Map(projectList.map((p) => [p.db, p.name]));
  v.innerHTML = `<h1>Databases</h1>
    <p class="sub">MariaDB on 127.0.0.1:${esc(env.mysql_host_port)} as root. Click a name to open it in ${esc(settings.db_client)}.</p>
    <div class="row" style="margin-bottom:12px">
      <button class="btn" data-db="">Open ${esc(settings.db_client)}</button>
      <button class="btn" data-adminer="">Adminer</button>
      <button class="btn" data-pma="">phpMyAdmin</button>
      <button class="btn" id="db-reload">Reload</button>
    </div>
    <table><thead><tr><th>Database</th><th>Project</th><th></th></tr></thead><tbody>
    ${dbs.map((d) => `<tr>
      <td><a class="link" data-db="${esc(d)}">${esc(d)}</a></td>
      <td><span class="kind">${esc(owners.get(d) ?? "")}</span></td>
      <td class="actions">
        <button class="btn small" data-db="${esc(d)}">${esc(settings.db_client)}</button>
        <button class="btn small" data-adminer="${esc(d)}">Adminer</button>
        <button class="btn small" data-pma="${esc(d)}">phpMyAdmin</button>
      </td></tr>`).join("")}
    </tbody></table>`;
  $("#db-reload").addEventListener("click", loadDatabases);
  wireCommonActions(v);
}

// ----- tools --------------------------------------------------------------------------
function renderTools() {
  const v = $("#view-tools");
  if (!env) { v.innerHTML = `<div class="card empty">Set the Devilbox folder in Settings first.</div>`; return; }
  const ui = env.ui_url;
  const svc = (name: string) => status.find((s) => s.service === name);
  const svcUrl = (name: string, target: number, scheme = "http") => {
    const p = svc(name)?.ports.find((x) => x.target === target);
    return p ? `${scheme}://localhost:${p.published}` : null;
  };
  const tools: { label: string; title: string; url: string | null; note: string; browser?: boolean }[] = [
    { label: "intranet", title: "Devilbox intranet", url: ui, note: "The classic dashboard, in its own window" },
    { label: "phpmyadmin", title: "phpMyAdmin", url: `${ui}/vendor/phpmyadmin-5.2.3/index.php`, note: "MariaDB / MySQL" },
    { label: "adminer", title: "Adminer", url: `${ui}/vendor/adminer-6.1.2-devilbox.php?username=root`, note: "MariaDB, PostgreSQL, SQLite" },
    { label: "phppgadmin", title: "phpPgAdmin", url: `${ui}/vendor/phppgadmin-7.13.0/`, note: "PostgreSQL" },
    { label: "phpcacheadmin", title: "phpCacheAdmin", url: `${ui}/vendor/phpcacheadmin-2.7.2/index.php`, note: "Redis, Memcached, OPcache" },
    { label: "opcache", title: "OPcache GUI", url: `${ui}/vendor/opcache-gui-3.6.1.php`, note: "PHP opcode cache" },
    { label: "mailpit", title: "Mailpit", url: svcUrl("mailpit", 8025), note: "Caught outgoing mail" },
    { label: "rabbitmq", title: "RabbitMQ", url: svcUrl("rabbitmq", 15672), note: "Management console" },
    { label: "meilisearch", title: "Meilisearch", url: svcUrl("meilisearch", 7700), note: "Search engine" },
    { label: "rustfs", title: "RustFS console", url: svcUrl("rustfs", 9001) ?? svcUrl("minio", 9001), note: "S3 storage" },
  ];
  v.innerHTML = `<h1>Tools</h1><p class="sub">Each tool opens in its own window inside the app, so nothing lands in your browser tabs.</p>
    <div class="grid">${tools.map((t) => `<div class="card">
      <h3>${esc(t.title)} ${t.url ? "" : `<span class="badge muted">not running</span>`}</h3>
      <p>${esc(t.note)}</p>
      <div class="actions">
        <button class="btn small" ${t.url ? "" : "disabled"} data-tool="${esc(t.label)}" data-title="${esc(t.title)}" data-url="${esc(t.url ?? "")}">Open window</button>
        <button class="btn small" ${t.url ? "" : "disabled"} data-open="${esc(t.url ?? "")}">Browser</button>
      </div></div>`).join("")}</div>`;
  v.querySelectorAll<HTMLButtonElement>("[data-tool]").forEach((b) => b.addEventListener("click", () =>
    call("open_tool", { label: b.dataset.tool, title: b.dataset.title, url: b.dataset.url })));
  wireCommonActions(v);
}

// ----- settings -----------------------------------------------------------------------
function renderSettings() {
  const v = $("#view-settings");
  v.innerHTML = `<h1>Settings</h1><p class="sub">Stored in your user Application Support folder. The Devilbox folder is also picked up from <code>DEVILBOX_ROOT</code>.</p>
    <div class="card">
      <div class="field"><label>Devilbox Boost folder</label>
        <div class="row"><input class="text" id="s-root" value="${esc(settings.root)}" placeholder="/path/to/Devilbox-Boost" style="min-width:420px">
        <button class="btn" id="s-pick">Choose…</button></div></div>
      <div class="field"><label>Database client (macOS app name used with mysql:// links)</label>
        <select class="text" id="s-dbclient">
          ${["Sequel Ace", "TablePlus", "DBeaver", "Beekeeper Studio"].map((n) => `<option ${settings.db_client === n ? "selected" : ""}>${n}</option>`).join("")}
        </select></div>
      <div class="field"><label>Editor app</label><input class="text" id="s-editor" value="${esc(settings.editor)}"></div>
      <div class="field"><label>Terminal app</label><input class="text" id="s-terminal" value="${esc(settings.terminal)}"></div>
      <div class="row"><button class="btn primary" id="s-save">Save</button></div>
    </div>
    ${env ? `<h2>Environment</h2><div class="card"><p>Intranet: ${esc(env.ui_url)}<br>TLD: .${esc(env.tld)} · PHP ${esc(env.php)} · ${esc(env.httpd)} · ${esc(env.mysql)} · PostgreSQL ${esc(env.pgsql)} · Redis ${esc(env.redis)} · Mongo ${esc(env.mongo)}</p></div>` : ""}`;
  $("#s-pick").addEventListener("click", async () => {
    const picked = await call<string | null>("pick_root");
    if (picked) $<HTMLInputElement>("#s-root").value = picked;
  });
  $("#s-save").addEventListener("click", async () => {
    const next: Settings = {
      root: $<HTMLInputElement>("#s-root").value.trim(),
      db_client: $<HTMLSelectElement>("#s-dbclient").value,
      editor: $<HTMLInputElement>("#s-editor").value.trim() || "Visual Studio Code",
      terminal: $<HTMLInputElement>("#s-terminal").value.trim() || "Terminal",
    };
    const saved = await call<Settings>("save_settings_cmd", { settings: next });
    if (!saved) return;
    settings = saved;
    toast("Settings saved");
    await boot();
    showView("overview");
  });
}

// ----- boot ---------------------------------------------------------------------------
async function boot() {
  settings = (await call<Settings>("get_settings")) ?? settings;
  try { env = await invoke<Environment>("get_environment"); } catch { env = null; }
  await refreshStatus();
  renderOverview();
}

boot();
window.setInterval(refreshStatus, 15000);
