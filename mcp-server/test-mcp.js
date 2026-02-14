#!/usr/bin/env node

/**
 * Quick MCP server tester
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const serverPath = join(__dirname, 'index.js');

console.log('Testing Devilbox MCP Server...\n');

const server = spawn('node', [serverPath]);

let output = '';

server.stdout.on('data', (data) => {
  output += data.toString();
});

server.stderr.on('data', (data) => {
  console.log('Server started:', data.toString().trim());
});

// Test 1: Initialize
setTimeout(() => {
  console.log('→ Test 1: Initializing server...');
  server.stdin.write(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'initialize',
    params: {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'test', version: '1.0.0' }
    }
  }) + '\n');
}, 100);

// Test 2: List tools
setTimeout(() => {
  console.log('→ Test 2: Listing tools...');
  server.stdin.write(JSON.stringify({
    jsonrpc: '2.0',
    id: 2,
    method: 'tools/list'
  }) + '\n');
}, 200);

// Test 3: Call devilbox_status
setTimeout(() => {
  console.log('→ Test 3: Calling devilbox_status...');
  server.stdin.write(JSON.stringify({
    jsonrpc: '2.0',
    id: 3,
    method: 'tools/call',
    params: {
      name: 'devilbox_status',
      arguments: {}
    }
  }) + '\n');
}, 300);

// Test 4: Call devilbox_vhosts
setTimeout(() => {
  console.log('→ Test 4: Calling devilbox_vhosts...');
  server.stdin.write(JSON.stringify({
    jsonrpc: '2.0',
    id: 4,
    method: 'tools/call',
    params: {
      name: 'devilbox_vhosts',
      arguments: {}
    }
  }) + '\n');
}, 400);

// Analyze results
setTimeout(() => {
  server.stdin.end();

  setTimeout(() => {
    console.log('\n' + '='.repeat(60));
    console.log('Test Results:');
    console.log('='.repeat(60));

    const lines = output.split('\n');
    let initOk = false;
    let toolCount = 0;
    let statusOk = false;
    let vhostsOk = false;

    lines.forEach(line => {
      try {
        const json = JSON.parse(line);

        if (json.id === 1 && json.result) {
          initOk = true;
          console.log('✓ Initialization: OK');
        }

        if (json.id === 2 && json.result && json.result.tools) {
          toolCount = json.result.tools.length;
          console.log(`✓ Tools list: ${toolCount} tools found`);
          console.log('  Tools:', json.result.tools.map(t => t.name).join(', '));
        }

        if (json.id === 3 && json.result) {
          statusOk = true;
          console.log('✓ devilbox_status: Working');
          const text = json.result.content[0].text;
          const containerCount = (text.match(/devilbox-/g) || []).length;
          console.log(`  Containers detected: ${containerCount}`);
        }

        if (json.id === 4 && json.result) {
          vhostsOk = true;
          console.log('✓ devilbox_vhosts: Working');
          const text = json.result.content[0].text;
          const match = text.match(/Found (\d+) projects/);
          if (match) {
            console.log(`  Projects detected: ${match[1]}`);
          }
        }
      } catch (e) {
        // Ignore non-JSON lines
      }
    });

    console.log('\n' + '='.repeat(60));
    console.log('Summary:');
    console.log('='.repeat(60));

    const allOk = initOk && toolCount === 10 && statusOk && vhostsOk;

    if (allOk) {
      console.log('✅ All tests passed!');
      console.log('✅ MCP server is working correctly');
      process.exit(0);
    } else {
      console.log('❌ Some tests failed');
      if (!initOk) console.log('  - Initialization failed');
      if (toolCount !== 10) console.log(`  - Expected 10 tools, got ${toolCount}`);
      if (!statusOk) console.log('  - devilbox_status failed');
      if (!vhostsOk) console.log('  - devilbox_vhosts failed');
      process.exit(1);
    }
  }, 500);
}, 1000);
