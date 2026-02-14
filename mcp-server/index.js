#!/usr/bin/env node

/**
 * Devilbox MCP Server
 *
 * AI-powered assistant for Devilbox development environment
 * Provides tools for service management, vhost configuration, logs, and more
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import { exec } from 'child_process';
import { promisify } from 'util';
import { readFile, writeFile, access } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const execAsync = promisify(exec);
const __dirname = dirname(fileURLToPath(import.meta.url));
const DEVILBOX_PATH = join(__dirname, '..');

/**
 * Execute docker-compose command
 */
async function dockerCompose(args, options = {}) {
  const cwd = options.cwd || DEVILBOX_PATH;
  const { stdout, stderr } = await execAsync(`docker-compose ${args}`, { cwd });
  return { stdout, stderr };
}

/**
 * Execute docker command
 */
async function docker(args, options = {}) {
  const { stdout, stderr } = await execAsync(`docker ${args}`, options);
  return { stdout, stderr };
}

/**
 * Read .env file
 */
async function readEnv() {
  const envPath = join(DEVILBOX_PATH, '.env');
  const content = await readFile(envPath, 'utf-8');
  const env = {};

  content.split('\n').forEach(line => {
    line = line.trim();
    if (line && !line.startsWith('#')) {
      const [key, ...valueParts] = line.split('=');
      if (key) {
        env[key.trim()] = valueParts.join('=').trim();
      }
    }
  });

  return env;
}

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
          description: 'Optional: Specific services to start (php, httpd, mysql, pgsql, redis, memcd, mongo). If omitted, starts all.',
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
    description: 'View logs from Devilbox containers. Can follow logs in real-time or view recent logs.',
    inputSchema: {
      type: 'object',
      properties: {
        service: {
          type: 'string',
          description: 'Service name (php, httpd, mysql, pgsql, redis, memcd, mongo, bind)',
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
    description: 'Execute a command inside a Devilbox container',
    inputSchema: {
      type: 'object',
      properties: {
        service: {
          type: 'string',
          description: 'Service name (php, httpd, mysql, pgsql, etc.)',
        },
        command: {
          type: 'string',
          description: 'Command to execute',
        },
      },
      required: ['service', 'command'],
    },
  },
  {
    name: 'devilbox_vhosts',
    description: 'List all virtual hosts (projects) in Devilbox',
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
    name: 'devilbox_databases',
    description: 'List databases in MySQL or PostgreSQL',
    inputSchema: {
      type: 'object',
      properties: {
        type: {
          type: 'string',
          enum: ['mysql', 'pgsql'],
          description: 'Database type',
          default: 'mysql',
        },
      },
    },
  },
  {
    name: 'devilbox_health',
    description: 'Check Devilbox health - service connectivity, disk space, permissions',
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
    const { stdout } = await dockerCompose('ps');
    return {
      content: [{ type: 'text', text: stdout }],
    };
  },

  async devilbox_start({ services }) {
    const serviceArg = services && services.length > 0 ? services.join(' ') : '';
    const { stdout, stderr } = await dockerCompose(`up -d ${serviceArg}`);
    return {
      content: [{ type: 'text', text: stdout || stderr }],
    };
  },

  async devilbox_stop({ services }) {
    const serviceArg = services && services.length > 0 ? services.join(' ') : '';
    const { stdout, stderr } = await dockerCompose(`stop ${serviceArg}`);
    return {
      content: [{ type: 'text', text: stdout || stderr }],
    };
  },

  async devilbox_restart({ services }) {
    const serviceArg = services && services.length > 0 ? services.join(' ') : '';
    const { stdout, stderr } = await dockerCompose(`restart ${serviceArg}`);
    return {
      content: [{ type: 'text', text: stdout || stderr }],
    };
  },

  async devilbox_logs({ service, lines = 100 }) {
    const { stdout } = await dockerCompose(`logs --tail=${lines} ${service}`);
    return {
      content: [{ type: 'text', text: stdout }],
    };
  },

  async devilbox_exec({ service, command }) {
    const containerName = `devilbox-${service}-1`;
    const { stdout, stderr } = await docker(`exec ${containerName} ${command}`);
    return {
      content: [{ type: 'text', text: stdout || stderr }],
    };
  },

  async devilbox_vhosts() {
    const dataPath = join(DEVILBOX_PATH, 'data/www');
    try {
      const { stdout } = await execAsync(`ls -1 "${dataPath}"`);
      const projects = stdout.trim().split('\n').filter(p => p);

      let output = `Found ${projects.length} projects:\n\n`;
      for (const project of projects) {
        const projectPath = join(dataPath, project);
        const devilboxConfigPath = join(projectPath, '.devilbox');

        try {
          await access(devilboxConfigPath);
          output += `✅ ${project} (configured)\n`;
        } catch {
          output += `⚠️  ${project} (no vhost config)\n`;
        }
      }

      return {
        content: [{ type: 'text', text: output }],
      };
    } catch (error) {
      return {
        content: [{ type: 'text', text: `Error: ${error.message}` }],
      };
    }
  },

  async devilbox_config({ action, key, value }) {
    const envPath = join(DEVILBOX_PATH, '.env');

    if (action === 'get') {
      const env = await readEnv();
      if (key) {
        const val = env[key];
        return {
          content: [{ type: 'text', text: val ? `${key}=${val}` : `Key "${key}" not found` }],
        };
      } else {
        const formatted = Object.entries(env)
          .map(([k, v]) => `${k}=${v}`)
          .join('\n');
        return {
          content: [{ type: 'text', text: formatted }],
        };
      }
    } else if (action === 'set') {
      if (!key || !value) {
        throw new Error('Both key and value are required for action=set');
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

      return {
        content: [{ type: 'text', text: `Updated ${key}=${value}` }],
      };
    }
  },

  async devilbox_databases({ type = 'mysql' }) {
    if (type === 'mysql') {
      const { stdout } = await docker('exec devilbox-php-1 mysql -h 127.0.0.1 -u root -proot --skip-ssl -e "SHOW DATABASES;"');
      return {
        content: [{ type: 'text', text: stdout }],
      };
    } else if (type === 'pgsql') {
      const { stdout } = await docker('exec devilbox-php-1 psql -h 127.0.0.1 -U postgres -l');
      return {
        content: [{ type: 'text', text: stdout }],
      };
    }
  },

  async devilbox_health() {
    const results = [];

    // Check services
    const { stdout: psOut } = await dockerCompose('ps');
    results.push('=== Service Status ===');
    results.push(psOut);

    // Check disk space
    const { stdout: dfOut } = await execAsync('df -h');
    results.push('\n=== Disk Space ===');
    results.push(dfOut);

    // Check Docker
    const { stdout: dockerInfo } = await docker('info --format "{{.OperatingSystem}} | {{.ServerVersion}}"');
    results.push('\n=== Docker Info ===');
    results.push(dockerInfo);

    // Check connectivity
    try {
      await docker('exec devilbox-php-1 ping -c 1 mysql');
      results.push('\n✅ PHP → MySQL connectivity: OK');
    } catch {
      results.push('\n❌ PHP → MySQL connectivity: FAILED');
    }

    return {
      content: [{ type: 'text', text: results.join('\n') }],
    };
  },
};

/**
 * Initialize MCP server
 */
const server = new Server(
  {
    name: 'devilbox',
    version: '1.0.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

/**
 * List tools handler
 */
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools,
}));

/**
 * Call tool handler
 */
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  const handler = toolHandlers[name];
  if (!handler) {
    throw new Error(`Unknown tool: ${name}`);
  }

  try {
    return await handler(args || {});
  } catch (error) {
    return {
      content: [{ type: 'text', text: `Error: ${error.message}` }],
      isError: true,
    };
  }
});

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
