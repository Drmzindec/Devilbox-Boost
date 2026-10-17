#!/usr/bin/env node

/**
 * Devilbox MCP Server
 *
 * AI-powered assistant for the Devilbox Boost development environment.
 * Provides tools for service management, project/vhost inspection, logs,
 * databases, artisan/composer, Xdebug toggling and more.
 *
 * Security model:
 * - No host shell is ever invoked. Every process is spawned via argument
 *   arrays (child_process.spawn), so tool arguments can never inject into
 *   the host shell.
 * - Commands that intentionally run a shell do so INSIDE a container via
 *   `docker compose exec -T <service> sh -c <command>` where <command> is a
 *   single argv element.
 * - Service names are allowlisted against the services actually defined in
 *   docker-compose.yml / docker-compose.override.yml.
 * - Credentials are read from the repo's .env (never hardcoded).
 */

import { McpServer, fromJsonSchema } from '@modelcontextprotocol/server';
import { StdioServerTransport } from '@modelcontextprotocol/server/stdio';
import { spawn } from 'child_process';
import { readFile, writeFile, access, readdir, unlink, open, stat } from 'fs/promises';
import { join, dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DEVILBOX_PATH = resolve(join(__dirname, '..'));

/** Fallback allowlist, used only if compose files cannot be parsed */
const FALLBACK_SERVICES = [
  'bind', 'php', 'httpd', 'mysql', 'pgsql', 'redis', 'memcd', 'mongo',
  'meilisearch', 'mailpit', 'rabbit', 'rustfs',
];

const MAX_OUTPUT_CHARS = 60000;
const DEFAULT_TIMEOUT_MS = 120000;

/**
 * Spawn a process with an argument array (no shell), collect output.
 * Supports cwd, stdin input, extra env and a timeout.
 */
function run(cmd, args, options = {}) {
  const {
    cwd = DEVILBOX_PATH,
    input = null,
    env = null,
    timeout = DEFAULT_TIMEOUT_MS,
  } = options;

  return new Promise((resolvePromise, reject) => {
    const child = spawn(cmd, args, {
      cwd,
      env: env ? { ...process.env, ...env } : process.env,
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';
    let timedOut = false;

    const timer = setTimeout(() => {
      timedOut = true;
      child.kill('SIGKILL');
    }, timeout);

    child.stdout.on('data', (d) => { stdout += d.toString(); });
    child.stderr.on('data', (d) => { stderr += d.toString(); });

    child.on('error', (err) => {
      clearTimeout(timer);
      reject(new Error(`Failed to run "${cmd}": ${err.message}`));
    });

    child.on('close', (code) => {
      clearTimeout(timer);
      if (timedOut) {
        reject(new Error(`Command timed out after ${timeout / 1000}s: ${cmd} ${args.join(' ')}`));
      } else if (code !== 0) {
        const detail = (stderr.trim() || stdout.trim() || `exit code ${code}`);
        const err = new Error(detail);
        err.code = code;
        err.stdout = stdout;
        err.stderr = stderr;
        reject(err);
      } else {
        resolvePromise({ stdout, stderr });
      }
    });

    if (input !== null) {
      child.stdin.write(input);
    }
    child.stdin.end();
  });
}

/**
 * Run `docker compose <args>` in the Devilbox root (argument array, no shell).
 * cwd is always the repo root so docker-compose.override.yml merging works.
 */
async function dockerCompose(args, options = {}) {
  return run('docker', ['compose', ...args], { ...options, cwd: DEVILBOX_PATH });
}

/**
 * Run an argv inside a container: docker compose exec -T [flags] <service> <argv...>
 */
async function composeExec(service, argv, options = {}) {
  const flags = ['exec', '-T'];
  if (options.user) flags.push('--user', options.user);
  if (options.workdir) flags.push('--workdir', options.workdir);
  for (const [key, value] of Object.entries(options.envVars || {})) {
    flags.push('--env', `${key}=${value}`);
  }
  return dockerCompose([...flags, service, ...argv], { input: options.input ?? null, timeout: options.timeout });
}

/**
 * Run a shell command INSIDE a container. The command string is passed as a
 * single argv element to `sh -c` — it never touches the host shell.
 */
async function execInContainer(service, command, options = {}) {
  return composeExec(service, ['sh', '-c', command], options);
}

/**
 * Parse service names from a compose file's top-level `services:` block.
 */
function parseComposeServices(content) {
  const services = [];
  let inServices = false;
  for (const line of content.split('\n')) {
    if (/^services:\s*(#.*)?$/.test(line)) {
      inServices = true;
      continue;
    }
    if (inServices) {
      if (/^[A-Za-z#]/.test(line)) {
        inServices = false;
        continue;
      }
      const m = line.match(/^ {2}([A-Za-z0-9._-]+):\s*(#.*)?$/);
      if (m) services.push(m[1]);
    }
  }
  return services;
}

let cachedServices = null;

/**
 * Get the allowlist of compose service names (docker-compose.yml merged with
 * docker-compose.override.yml). Cached after first read.
 */
async function getServices() {
  if (cachedServices) return cachedServices;
  const names = new Set();
  for (const file of ['docker-compose.yml', 'docker-compose.override.yml']) {
    try {
      const content = await readFile(join(DEVILBOX_PATH, file), 'utf-8');
      parseComposeServices(content).forEach(s => names.add(s));
    } catch {
      // file missing — ignore
    }
  }
  cachedServices = names.size > 0 ? [...names].sort() : [...FALLBACK_SERVICES].sort();
  return cachedServices;
}

/**
 * Validate a service name against the compose allowlist.
 * Throws with an actionable message on failure.
 */
async function validateService(service) {
  const services = await getServices();
  if (typeof service !== 'string' || !services.includes(service)) {
    throw new Error(
      `Service '${service}' not known. Valid services: ${services.join(', ')}`
    );
  }
  return service;
}

async function validateServices(list) {
  for (const s of list) await validateService(s);
  return list;
}

/**
 * Read .env file. Everything after the first '=' is the value.
 */
async function readEnv() {
  const envPath = join(DEVILBOX_PATH, '.env');
  const content = await readFile(envPath, 'utf-8');
  const env = {};

  content.split('\n').forEach(line => {
    line = line.trim();
    if (line && !line.startsWith('#')) {
      const idx = line.indexOf('=');
      if (idx > 0) {
        const key = line.slice(0, idx).trim();
        let value = line.slice(idx + 1).trim();
        // Strip one layer of matching surrounding quotes
        if (value.length >= 2 &&
            ((value.startsWith('"') && value.endsWith('"')) ||
             (value.startsWith("'") && value.endsWith("'")))) {
          value = value.slice(1, -1);
        }
        env[key] = value;
      }
    }
  });

  return env;
}

/**
 * Validate a project name (directory under data/www) and confirm it exists.
 */
async function validateProject(project) {
  if (typeof project !== 'string' || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(project)) {
    throw new Error(
      `Invalid project name '${project}'. Use letters, digits, dots, dashes and underscores.`
    );
  }
  const projectPath = join(DEVILBOX_PATH, 'data', 'www', project);
  try {
    await access(projectPath);
  } catch {
    const projects = await listProjectDirs();
    throw new Error(
      `Project '${project}' not found in data/www. Available projects: ${projects.join(', ') || '(none)'}`
    );
  }
  return project;
}

async function listProjectDirs() {
  const dataPath = join(DEVILBOX_PATH, 'data', 'www');
  const entries = await readdir(dataPath, { withFileTypes: true });
  return entries
    .filter(e => e.isDirectory() && !e.name.startsWith('.'))
    .map(e => e.name)
    .sort();
}

/**
 * Validate a PHP version string like "8.4" and default to PHP_SERVER from .env.
 */
async function resolvePhpVersion(version) {
  if (version === undefined || version === null || version === '') {
    const env = await readEnv();
    version = env.PHP_SERVER;
    if (!version) throw new Error('PHP_SERVER is not set in .env and no version was provided');
  }
  if (!/^\d+\.\d+$/.test(String(version))) {
    throw new Error(`Invalid PHP version '${version}'. Expected e.g. "8.4"`);
  }
  return String(version);
}

/**
 * Coerce a line count into a sane integer range.
 */
function coerceLines(lines, fallback) {
  const n = Number.parseInt(lines, 10);
  if (Number.isNaN(n)) return fallback;
  return Math.min(Math.max(n, 1), 10000);
}

/**
 * Truncate long output for MCP responses, keeping head and tail.
 */
function truncate(text) {
  if (text.length <= MAX_OUTPUT_CHARS) return text;
  const half = Math.floor(MAX_OUTPUT_CHARS / 2);
  return text.slice(0, half) +
    `\n\n... [output truncated: ${text.length} chars total] ...\n\n` +
    text.slice(-half);
}

function textResult(text) {
  return { content: [{ type: 'text', text: truncate(text || '(no output)') }] };
}

/**
 * Read the last N lines of a file without loading huge logs into memory.
 */
async function tailFile(path, lines) {
  const info = await stat(path);
  const wanted = Math.min(info.size, Math.max(64 * 1024, lines * 512));
  const fh = await open(path, 'r');
  try {
    const buf = Buffer.alloc(wanted);
    await fh.read(buf, 0, wanted, info.size - wanted);
    const content = buf.toString('utf-8');
    const allLines = content.split('\n');
    // Drop a possibly-partial first line if we started mid-file
    if (info.size > wanted) allLines.shift();
    return allLines.slice(-lines - 1).join('\n').trimEnd();
  } finally {
    await fh.close();
  }
}

const XDEBUG_INI_NAME = 'zzz-mcp-xdebug.ini';

/**
 * List all available tools
 */
const tools = [
  {
    name: 'devilbox_status',
    description: 'Get status of all Devilbox services (running/stopped)',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'devilbox_start',
    description: 'Start Devilbox services. Can start all services or specific ones.',
    inputSchema: {
      type: 'object',
      properties: {
        services: {
          type: 'array',
          items: { type: 'string' },
          description: 'Optional: Specific services to start (php, httpd, mysql, pgsql, redis, memcd, mongo, bind, mailpit, meilisearch, rabbit, rustfs). If omitted, starts all.',
        },
      },
    },
  },
  {
    name: 'devilbox_stop',
    description: 'Stop Devilbox services. Can stop all services or specific ones.',
    inputSchema: {
      type: 'object',
      properties: {
        services: {
          type: 'array',
          items: { type: 'string' },
          description: 'Optional: Specific services to stop. If omitted, stops all.',
        },
      },
    },
  },
  {
    name: 'devilbox_restart',
    description: 'Restart Devilbox services. Useful after configuration changes.',
    inputSchema: {
      type: 'object',
      properties: {
        services: {
          type: 'array',
          items: { type: 'string' },
          description: 'Optional: Specific services to restart. If omitted, restarts all.',
        },
      },
    },
  },
  {
    name: 'devilbox_logs',
    description: 'View recent logs from a Devilbox container.',
    inputSchema: {
      type: 'object',
      properties: {
        service: {
          type: 'string',
          description: 'Service name (php, httpd, mysql, pgsql, redis, memcd, mongo, bind, mailpit, meilisearch, rabbit, rustfs)',
        },
        lines: {
          type: 'number',
          description: 'Number of recent log lines to show (default: 100)',
          default: 100,
        },
      },
      required: ['service'],
    },
  },
  {
    name: 'devilbox_exec',
    description: 'Execute a shell command inside a Devilbox container (runs via sh -c inside the container).',
    inputSchema: {
      type: 'object',
      properties: {
        service: {
          type: 'string',
          description: 'Service name (php, httpd, mysql, pgsql, etc.)',
        },
        command: {
          type: 'string',
          description: 'Command to execute inside the container',
        },
        user: {
          type: 'string',
          description: 'Optional: user to run as inside the container (e.g. "devilbox" for file-permission-safe operations in /shared/httpd)',
        },
        workdir: {
          type: 'string',
          description: 'Optional: working directory inside the container (e.g. /shared/httpd/myproject)',
        },
      },
      required: ['service', 'command'],
    },
  },
  {
    name: 'devilbox_artisan',
    description: 'Run `php artisan <args>` in a Laravel project under /shared/httpd/<project>, as user devilbox.',
    inputSchema: {
      type: 'object',
      properties: {
        project: {
          type: 'string',
          description: 'Project directory name under data/www (e.g. "tapthetable")',
        },
        args: {
          type: 'string',
          description: 'Arguments for artisan (e.g. "migrate --seed", "route:list", "queue:work --once"). Default: "list"',
          default: 'list',
        },
      },
      required: ['project'],
    },
  },
  {
    name: 'devilbox_composer',
    description: 'Run composer in a project directory under /shared/httpd/<project>, as user devilbox.',
    inputSchema: {
      type: 'object',
      properties: {
        project: {
          type: 'string',
          description: 'Project directory name under data/www',
        },
        args: {
          type: 'string',
          description: 'Arguments for composer (e.g. "install", "require laravel/pint --dev", "dump-autoload")',
        },
      },
      required: ['project', 'args'],
    },
  },
  {
    name: 'devilbox_sql',
    description: 'Run a SQL query against MySQL/MariaDB or PostgreSQL and return the results. Credentials come from .env.',
    inputSchema: {
      type: 'object',
      properties: {
        engine: {
          type: 'string',
          enum: ['mysql', 'pgsql'],
          description: 'Database engine (default: mysql)',
          default: 'mysql',
        },
        query: {
          type: 'string',
          description: 'SQL query to execute (e.g. "SELECT COUNT(*) FROM users;")',
        },
        database: {
          type: 'string',
          description: 'Optional: database name to run the query against',
        },
      },
      required: ['query'],
    },
  },
  {
    name: 'devilbox_databases',
    description: 'List databases in MySQL and/or PostgreSQL',
    inputSchema: {
      type: 'object',
      properties: {
        type: {
          type: 'string',
          enum: ['mysql', 'pgsql', 'all'],
          description: 'Database type (default: all)',
          default: 'all',
        },
      },
    },
  },
  {
    name: 'devilbox_php_errors',
    description: 'Tail the PHP-FPM error log (log/php-fpm-<version>/php-errors.log). Reads the file directly from the host.',
    inputSchema: {
      type: 'object',
      properties: {
        lines: {
          type: 'number',
          description: 'Number of recent log lines to show (default: 50)',
          default: 50,
        },
        php_version: {
          type: 'string',
          description: 'Optional: PHP version (e.g. "8.4"). Defaults to PHP_SERVER from .env',
        },
      },
    },
  },
  {
    name: 'devilbox_xdebug',
    description: 'Check, enable or disable Xdebug via a drop-in ini in cfg/php-ini-<version>/ and reload PHP-FPM.',
    inputSchema: {
      type: 'object',
      properties: {
        action: {
          type: 'string',
          enum: ['status', 'on', 'off'],
          description: 'status: report current state; on: enable Xdebug; off: disable Xdebug',
        },
        mode: {
          type: 'string',
          description: 'Xdebug mode when enabling (default: "debug,develop")',
          default: 'debug,develop',
        },
        port: {
          type: 'number',
          description: 'Xdebug client port when enabling (default: 9003)',
          default: 9003,
        },
        trigger: {
          type: 'boolean',
          description: 'If true, only start debugging when a trigger (XDEBUG_TRIGGER) is present. Default: false (start with every request)',
          default: false,
        },
      },
      required: ['action'],
    },
  },
  {
    name: 'devilbox_projects',
    description: 'List projects in data/www with framework detection and vhost URLs (uses TLD_SUFFIX from .env).',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'devilbox_vhosts',
    description: 'List all virtual hosts (projects) in Devilbox and whether they have a vhost template configured',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
  {
    name: 'devilbox_config',
    description: 'Get or update Devilbox configuration (.env file)',
    inputSchema: {
      type: 'object',
      properties: {
        action: {
          type: 'string',
          enum: ['get', 'set'],
          description: 'Action: get (read config) or set (update config)',
        },
        key: {
          type: 'string',
          description: 'Configuration key (e.g., PHP_SERVER, MYSQL_SERVER)',
        },
        value: {
          type: 'string',
          description: 'Value to set (only for action=set)',
        },
      },
      required: ['action'],
    },
  },
  {
    name: 'devilbox_health',
    description: 'Check Devilbox health - service status, container connectivity, disk space, Docker info',
    inputSchema: {
      type: 'object',
      properties: {},
    },
  },
];

/**
 * Tool implementations
 */
const toolHandlers = {
  async devilbox_status() {
    const { stdout } = await dockerCompose(['ps']);
    return textResult(stdout);
  },

  async devilbox_start({ services }) {
    const args = ['up', '-d'];
    if (services && services.length > 0) {
      args.push(...await validateServices(services));
    }
    const { stdout, stderr } = await dockerCompose(args);
    return textResult(stdout || stderr);
  },

  async devilbox_stop({ services }) {
    const args = ['stop'];
    if (services && services.length > 0) {
      args.push(...await validateServices(services));
    }
    const { stdout, stderr } = await dockerCompose(args);
    return textResult(stdout || stderr);
  },

  async devilbox_restart({ services }) {
    const args = ['restart'];
    if (services && services.length > 0) {
      args.push(...await validateServices(services));
    }
    const { stdout, stderr } = await dockerCompose(args);
    return textResult(stdout || stderr);
  },

  async devilbox_logs({ service, lines = 100 }) {
    await validateService(service);
    const tail = coerceLines(lines, 100);
    const { stdout, stderr } = await dockerCompose(['logs', '--tail', String(tail), service]);
    // Some services log to stderr
    return textResult([stdout, stderr].filter(Boolean).join('\n'));
  },

  async devilbox_exec({ service, command, user, workdir }) {
    await validateService(service);
    if (typeof command !== 'string' || command.trim() === '') {
      throw new Error('command must be a non-empty string');
    }
    if (user !== undefined && !/^[A-Za-z0-9._-]+$/.test(user)) {
      throw new Error(`Invalid user '${user}'`);
    }
    if (workdir !== undefined && !workdir.startsWith('/')) {
      throw new Error(`workdir must be an absolute path inside the container (got '${workdir}')`);
    }
    const { stdout, stderr } = await execInContainer(service, command, { user, workdir });
    return textResult(stdout || stderr);
  },

  async devilbox_artisan({ project, args = 'list' }) {
    await validateProject(project);
    const artisanPath = join(DEVILBOX_PATH, 'data', 'www', project, 'artisan');
    try {
      await access(artisanPath);
    } catch {
      throw new Error(
        `Project '${project}' has no artisan file - is it a Laravel project? Use devilbox_projects to see detected frameworks.`
      );
    }
    const { stdout, stderr } = await execInContainer('php', `php artisan ${args}`, {
      user: 'devilbox',
      workdir: `/shared/httpd/${project}`,
    });
    return textResult(stdout || stderr);
  },

  async devilbox_composer({ project, args }) {
    await validateProject(project);
    if (typeof args !== 'string' || args.trim() === '') {
      throw new Error('args must be a non-empty string (e.g. "install")');
    }
    const { stdout, stderr } = await execInContainer('php', `composer ${args}`, {
      user: 'devilbox',
      workdir: `/shared/httpd/${project}`,
    });
    // Composer writes progress info to stderr
    return textResult([stdout, stderr].filter(Boolean).join('\n'));
  },

  async devilbox_sql({ engine = 'mysql', query, database }) {
    if (typeof query !== 'string' || query.trim() === '') {
      throw new Error('query must be a non-empty string');
    }
    if (database !== undefined && !/^[A-Za-z0-9_-]+$/.test(database)) {
      throw new Error(`Invalid database name '${database}'`);
    }
    const env = await readEnv();

    if (engine === 'mysql') {
      const password = env.MYSQL_ROOT_PASSWORD ?? '';
      // MariaDB 11+ images ship `mariadb` (no `mysql` symlink); MySQL images
      // ship `mysql`. --skip-ssl only exists on the mariadb client.
      const pickClient =
        'CLIENT="$(command -v mariadb || command -v mysql)"; ' +
        'case "$CLIENT" in *mariadb) exec "$CLIENT" --skip-ssl "$@";; *) exec "$CLIENT" "$@";; esac';
      const argv = ['sh', '-c', pickClient, 'sql-client', '-uroot', '--table'];
      if (database) argv.push('-D', database);
      const { stdout, stderr } = await composeExec('mysql', argv, {
        envVars: { MYSQL_PWD: password },
        input: query,
      });
      return textResult(stdout || stderr || 'Query OK (no output)');
    }

    if (engine === 'pgsql') {
      const pgUser = env.PGSQL_ROOT_USER || 'postgres';
      const password = env.PGSQL_ROOT_PASSWORD ?? '';
      const argv = ['psql', '-U', pgUser, '-d', database || 'postgres'];
      const { stdout, stderr } = await composeExec('pgsql', argv, {
        envVars: { PGPASSWORD: password },
        input: query,
      });
      return textResult(stdout || stderr || 'Query OK (no output)');
    }

    throw new Error(`Unknown engine '${engine}'. Valid engines: mysql, pgsql`);
  },

  async devilbox_databases({ type = 'all' } = {}) {
    const sections = [];

    if (type === 'mysql' || type === 'all') {
      try {
        const result = await toolHandlers.devilbox_sql({ engine: 'mysql', query: 'SHOW DATABASES;' });
        sections.push('=== MySQL/MariaDB ===\n' + result.content[0].text);
      } catch (error) {
        sections.push(`=== MySQL/MariaDB ===\nError: ${error.message}`);
      }
    }

    if (type === 'pgsql' || type === 'all') {
      try {
        const env = await readEnv();
        const pgUser = env.PGSQL_ROOT_USER || 'postgres';
        const { stdout } = await composeExec('pgsql', ['psql', '-U', pgUser, '-l'], {
          envVars: { PGPASSWORD: env.PGSQL_ROOT_PASSWORD ?? '' },
        });
        sections.push('=== PostgreSQL ===\n' + stdout);
      } catch (error) {
        sections.push(`=== PostgreSQL ===\nError: ${error.message}`);
      }
    }

    if (sections.length === 0) {
      throw new Error(`Unknown type '${type}'. Valid types: mysql, pgsql, all`);
    }
    return textResult(sections.join('\n\n'));
  },

  async devilbox_php_errors({ lines = 50, php_version } = {}) {
    const version = await resolvePhpVersion(php_version);
    const count = coerceLines(lines, 50);
    const logPath = join(DEVILBOX_PATH, 'log', `php-fpm-${version}`, 'php-errors.log');

    try {
      await access(logPath);
    } catch {
      throw new Error(
        `PHP error log not found at log/php-fpm-${version}/php-errors.log. ` +
        `Check PHP_SERVER in .env or pass php_version explicitly.`
      );
    }

    const tail = await tailFile(logPath, count);
    if (!tail.trim()) {
      return textResult(`PHP ${version} error log is empty. No errors - nice!`);
    }
    return textResult(`Last ${count} lines of log/php-fpm-${version}/php-errors.log:\n\n${tail}`);
  },

  async devilbox_xdebug({ action, mode = 'debug,develop', port = 9003, trigger = false }) {
    const env = await readEnv();
    const version = await resolvePhpVersion(env.PHP_SERVER);
    const iniDir = join(DEVILBOX_PATH, 'cfg', `php-ini-${version}`);
    const iniPath = join(iniDir, XDEBUG_INI_NAME);

    try {
      await access(iniDir);
    } catch {
      throw new Error(`Config directory cfg/php-ini-${version} does not exist`);
    }

    const dropInExists = await access(iniPath).then(() => true, () => false);

    if (action === 'status') {
      let cliState = 'unknown';
      try {
        const { stdout } = await composeExec('php', [
          'php', '-r',
          "echo extension_loaded('xdebug') ? 'loaded, mode=' . ini_get('xdebug.mode') : 'not loaded';",
        ]);
        cliState = stdout.trim();
      } catch (error) {
        cliState = `could not query php container (${error.message})`;
      }
      return textResult(
        `Xdebug status (PHP ${version}):\n` +
        `- MCP drop-in (cfg/php-ini-${version}/${XDEBUG_INI_NAME}): ${dropInExists ? 'present (enabled)' : 'absent (disabled)'}\n` +
        `- PHP CLI reports: ${cliState}\n` +
        `\nNote: PHP-FPM may report differently from CLI. Use "on"/"off" to toggle.`
      );
    }

    if (action === 'on') {
      if (!/^[a-z,]+$/.test(mode)) {
        throw new Error(`Invalid Xdebug mode '${mode}'. Example: "debug,develop"`);
      }
      const clientPort = Number.parseInt(port, 10);
      if (Number.isNaN(clientPort) || clientPort < 1 || clientPort > 65535) {
        throw new Error(`Invalid port '${port}'`);
      }
      const ini = [
        `; Managed by the Devilbox MCP server (devilbox_xdebug tool).`,
        `; Delete this file (or run devilbox_xdebug action=off) to disable Xdebug.`,
        `[PHP]`,
        `xdebug.mode               = ${mode}`,
        `xdebug.start_with_request = ${trigger ? 'trigger' : 'yes'}`,
        `xdebug.client_host        = host.docker.internal`,
        `xdebug.client_port        = ${clientPort}`,
        `xdebug.log_level          = 0`,
        '',
      ].join('\n');
      await writeFile(iniPath, ini);
      const reload = await reloadPhpFpm();
      return textResult(
        `Xdebug enabled (mode=${mode}, port=${clientPort}, ` +
        `start_with_request=${trigger ? 'trigger' : 'yes'}).\n` +
        `Wrote cfg/php-ini-${version}/${XDEBUG_INI_NAME}.\n${reload}`
      );
    }

    if (action === 'off') {
      if (!dropInExists) {
        return textResult(
          `Xdebug drop-in already absent (cfg/php-ini-${version}/${XDEBUG_INI_NAME} not found). ` +
          `If Xdebug is still active it is configured elsewhere (e.g. custom.ini).`
        );
      }
      await unlink(iniPath);
      const reload = await reloadPhpFpm();
      return textResult(`Xdebug disabled. Removed cfg/php-ini-${version}/${XDEBUG_INI_NAME}.\n${reload}`);
    }

    throw new Error(`Unknown action '${action}'. Valid actions: status, on, off`);
  },

  async devilbox_projects() {
    const env = await readEnv();
    const tld = env.TLD_SUFFIX || 'local';
    const templateDir = env.HTTPD_TEMPLATE_DIR || '.devilbox';
    const dataPath = join(DEVILBOX_PATH, 'data', 'www');
    const projects = await listProjectDirs();

    if (projects.length === 0) {
      return textResult(`No projects found in data/www. Create a directory there to add one.`);
    }

    const lines = [`Found ${projects.length} projects in data/www (TLD: .${tld}):\n`];
    for (const project of projects) {
      const projectPath = join(dataPath, project);
      const has = async (...parts) => access(join(projectPath, ...parts)).then(() => true, () => false);

      let framework = 'unknown';
      if (await has('artisan')) framework = 'Laravel';
      else if (await has('wp-config.php') || await has('wp-content')) framework = 'WordPress';
      else if (await has('bin', 'console')) framework = 'Symfony';
      else if (await has('composer.json')) framework = 'PHP (composer)';
      else if (await has('index.php')) framework = 'PHP';
      else if (await has('index.html')) framework = 'static';

      const configured = await has(templateDir);
      lines.push(
        `- ${project}\n` +
        `    URL:       http://${project}.${tld}\n` +
        `    Framework: ${framework}\n` +
        `    Vhost:     ${configured ? `custom template (${templateDir}/)` : 'auto-detected'}`
      );
    }

    return textResult(lines.join('\n'));
  },

  async devilbox_vhosts() {
    const env = await readEnv();
    const templateDir = env.HTTPD_TEMPLATE_DIR || '.devilbox';
    const projects = await listProjectDirs();

    let output = `Found ${projects.length} projects:\n\n`;
    for (const project of projects) {
      const configPath = join(DEVILBOX_PATH, 'data', 'www', project, templateDir);
      const configured = await access(configPath).then(() => true, () => false);
      output += configured
        ? `[configured] ${project}\n`
        : `[auto]       ${project}\n`;
    }

    return textResult(output);
  },

  async devilbox_config({ action, key, value }) {
    const envPath = join(DEVILBOX_PATH, '.env');

    if (action === 'get') {
      const env = await readEnv();
      if (key) {
        const val = env[key];
        return textResult(val !== undefined ? `${key}=${val}` : `Key "${key}" not found`);
      }
      // Redact credentials in the full listing; ask for a key explicitly to see it
      const formatted = Object.entries(env)
        .map(([k, v]) => /PASSWORD|SECRET|TOKEN/i.test(k) && v !== '' ? `${k}=********` : `${k}=${v}`)
        .join('\n');
      return textResult(formatted);
    }

    if (action === 'set') {
      if (!key || value === undefined) {
        throw new Error('Both key and value are required for action=set');
      }
      if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)) {
        throw new Error(`Invalid key '${key}'. Keys must look like PHP_SERVER, MYSQL_SERVER, ...`);
      }
      if (/[\r\n]/.test(value)) {
        throw new Error('Value must not contain newlines');
      }

      const content = await readFile(envPath, 'utf-8');
      const lines = content.split('\n');
      let found = false;

      const updatedLines = lines.map(line => {
        if (line.trim().startsWith(key + '=')) {
          found = true;
          return `${key}=${value}`;
        }
        return line;
      });

      if (!found) {
        updatedLines.push(`${key}=${value}`);
      }

      await writeFile(envPath, updatedLines.join('\n'));

      return textResult(
        `Updated ${key}=${value}\n` +
        `Restart affected services for the change to take effect (devilbox_restart).`
      );
    }

    throw new Error(`Unknown action '${action}'. Valid actions: get, set`);
  },

  async devilbox_health() {
    const results = [];

    // Check services
    try {
      const { stdout: psOut } = await dockerCompose(['ps']);
      results.push('=== Service Status ===');
      results.push(psOut.trim());
    } catch (error) {
      results.push(`=== Service Status ===\nError: ${error.message}`);
    }

    // Check Docker
    try {
      const { stdout: dockerInfo } = await run('docker', [
        'info', '--format', '{{.OperatingSystem}} | Docker {{.ServerVersion}}',
      ]);
      results.push('\n=== Docker Info ===');
      results.push(dockerInfo.trim());
    } catch (error) {
      results.push(`\n=== Docker Info ===\nError: ${error.message}`);
    }

    // Check disk space of the Devilbox volume
    try {
      const { stdout: dfOut } = await run('df', ['-h', DEVILBOX_PATH]);
      results.push('\n=== Disk Space ===');
      results.push(dfOut.trim());
    } catch (error) {
      results.push(`\n=== Disk Space ===\nError: ${error.message}`);
    }

    // Check connectivity from PHP container to backing services
    results.push('\n=== Connectivity (from php container) ===');
    const targets = [
      ['mysql', 3306],
      ['pgsql', 5432],
      ['redis', 6379],
      ['memcd', 11211],
      ['mongo', 27017],
    ];
    for (const [host, port] of targets) {
      try {
        await composeExec('php', [
          'php', '-r',
          `exit(@fsockopen('${host}', ${port}, $e, $s, 2) ? 0 : 1);`,
        ], { timeout: 15000 });
        results.push(`[OK]   php -> ${host}:${port}`);
      } catch {
        results.push(`[FAIL] php -> ${host}:${port}`);
      }
    }

    return textResult(results.join('\n'));
  },
};

/**
 * Reload PHP-FPM inside the php container (graceful, USR2 to PID 1).
 */
async function reloadPhpFpm() {
  try {
    await composeExec('php', ['kill', '-USR2', '1'], { timeout: 15000 });
    return 'PHP-FPM reloaded (kill -USR2 1). If the change does not apply, restart the php container: devilbox_restart ["php"].';
  } catch (error) {
    return `Could not reload PHP-FPM (${error.message}). Restart the php container to apply: devilbox_restart ["php"].`;
  }
}

/**
 * Initialize MCP server (MCP TypeScript SDK v2, @modelcontextprotocol/server).
 *
 * Tools keep their plain JSON Schema definitions above; fromJsonSchema() turns
 * each one into a validated input schema, so malformed arguments are rejected
 * by the SDK before a handler ever runs.
 */
const server = new McpServer({
  name: 'devilbox',
  version: '2.1.0',
});

for (const tool of tools) {
  const handler = toolHandlers[tool.name];
  if (!handler) {
    throw new Error(`No handler registered for tool: ${tool.name}`);
  }

  server.registerTool(
    tool.name,
    {
      description: tool.description,
      inputSchema: fromJsonSchema(tool.inputSchema),
    },
    async (args) => {
      try {
        return await handler(args || {});
      } catch (error) {
        return {
          content: [{ type: 'text', text: `Error: ${error.message}` }],
          isError: true,
        };
      }
    }
  );
}

/**
 * Start server
 */
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('Devilbox MCP server running on stdio');
}

main().catch((error) => {
  console.error('Server error:', error);
  process.exit(1);
});
