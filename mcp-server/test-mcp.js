#!/usr/bin/env node

/**
 * Quick MCP server tester
 *
 * Exercises the server over stdio: initialize, tools/list and a handful of
 * tool calls, including a check that the service allowlist rejects unknown
 * (potentially malicious) service names.
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const serverPath = join(__dirname, 'index.js');

const EXPECTED_TOOLS = [
  'devilbox_status',
  'devilbox_start',
  'devilbox_stop',
  'devilbox_restart',
  'devilbox_logs',
  'devilbox_exec',
  'devilbox_artisan',
  'devilbox_composer',
  'devilbox_sql',
  'devilbox_databases',
  'devilbox_php_errors',
  'devilbox_xdebug',
  'devilbox_projects',
  'devilbox_vhosts',
  'devilbox_config',
  'devilbox_health',
];

console.log('Testing Devilbox MCP Server...\n');

const server = spawn('node', [serverPath]);

let output = '';

server.stdout.on('data', (data) => {
  output += data.toString();
});

server.stderr.on('data', (data) => {
  console.log('Server started:', data.toString().trim());
});

function send(msg, delay) {
  setTimeout(() => {
    server.stdin.write(JSON.stringify(msg) + '\n');
  }, delay);
}

// Test 1: Initialize
console.log('→ Test 1: Initializing server...');
send({
  jsonrpc: '2.0',
  id: 1,
  method: 'initialize',
  params: {
    protocolVersion: '2024-11-05',
    capabilities: {},
    clientInfo: { name: 'test', version: '1.0.0' }
  }
}, 100);

// Test 2: List tools
console.log('→ Test 2: Listing tools...');
send({ jsonrpc: '2.0', id: 2, method: 'tools/list' }, 250);

// Test 3: devilbox_vhosts (filesystem only, no Docker required)
console.log('→ Test 3: Calling devilbox_vhosts...');
send({
  jsonrpc: '2.0',
  id: 3,
  method: 'tools/call',
  params: { name: 'devilbox_vhosts', arguments: {} }
}, 400);

// Test 4: devilbox_projects (filesystem + .env, no Docker required)
console.log('→ Test 4: Calling devilbox_projects...');
send({
  jsonrpc: '2.0',
  id: 4,
  method: 'tools/call',
  params: { name: 'devilbox_projects', arguments: {} }
}, 550);

// Test 5: devilbox_config get (reads .env)
console.log('→ Test 5: Calling devilbox_config (get PHP_SERVER)...');
send({
  jsonrpc: '2.0',
  id: 5,
  method: 'tools/call',
  params: { name: 'devilbox_config', arguments: { action: 'get', key: 'PHP_SERVER' } }
}, 700);

// Test 6: allowlist rejection - a hostile service name must be refused
console.log('→ Test 6: Verifying service allowlist rejects bad input...');
send({
  jsonrpc: '2.0',
  id: 6,
  method: 'tools/call',
  params: {
    name: 'devilbox_exec',
    arguments: { service: 'php; rm -rf /', command: 'id' }
  }
}, 850);

// Analyze results
setTimeout(() => {
  server.stdin.end();

  setTimeout(() => {
    console.log('\n' + '='.repeat(60));
    console.log('Test Results:');
    console.log('='.repeat(60));

    const lines = output.split('\n');
    let initOk = false;
    let toolsOk = false;
    let vhostsOk = false;
    let projectsOk = false;
    let configOk = false;
    let allowlistOk = false;

    lines.forEach(line => {
      try {
        const json = JSON.parse(line);

        if (json.id === 1 && json.result) {
          initOk = true;
          console.log('✓ Initialization: OK');
        }

        if (json.id === 2 && json.result && json.result.tools) {
          const names = json.result.tools.map(t => t.name);
          const missing = EXPECTED_TOOLS.filter(t => !names.includes(t));
          toolsOk = missing.length === 0;
          console.log(`${toolsOk ? '✓' : '✗'} Tools list: ${names.length} tools found`);
          console.log('  Tools:', names.join(', '));
          if (missing.length > 0) {
            console.log('  Missing:', missing.join(', '));
          }
        }

        if (json.id === 3 && json.result && !json.result.isError) {
          vhostsOk = true;
          console.log('✓ devilbox_vhosts: Working');
          const match = json.result.content[0].text.match(/Found (\d+) projects/);
          if (match) console.log(`  Projects detected: ${match[1]}`);
        }

        if (json.id === 4 && json.result && !json.result.isError) {
          projectsOk = true;
          console.log('✓ devilbox_projects: Working');
          const match = json.result.content[0].text.match(/Found (\d+) projects/);
          if (match) console.log(`  Projects detected: ${match[1]}`);
        }

        if (json.id === 5 && json.result && !json.result.isError) {
          configOk = /PHP_SERVER=/.test(json.result.content[0].text);
          console.log(`${configOk ? '✓' : '✗'} devilbox_config: ${json.result.content[0].text.trim()}`);
        }

        if (json.id === 6 && json.result) {
          const text = json.result.content[0].text;
          allowlistOk = json.result.isError === true && /not known/.test(text);
          console.log(`${allowlistOk ? '✓' : '✗'} Service allowlist: hostile service name rejected`);
          if (!allowlistOk) console.log('  Response was:', text.slice(0, 200));
        }
      } catch (e) {
        // Ignore non-JSON lines
      }
    });

    console.log('\n' + '='.repeat(60));
    console.log('Summary:');
    console.log('='.repeat(60));

    const allOk = initOk && toolsOk && vhostsOk && projectsOk && configOk && allowlistOk;

    if (allOk) {
      console.log('✅ All tests passed!');
      console.log('✅ MCP server is working correctly');
      process.exit(0);
    } else {
      console.log('❌ Some tests failed');
      if (!initOk) console.log('  - Initialization failed');
      if (!toolsOk) console.log('  - Tools list incomplete');
      if (!vhostsOk) console.log('  - devilbox_vhosts failed');
      if (!projectsOk) console.log('  - devilbox_projects failed');
      if (!configOk) console.log('  - devilbox_config failed');
      if (!allowlistOk) console.log('  - Service allowlist did not reject bad input');
      process.exit(1);
    }
  }, 500);
}, 1400);
