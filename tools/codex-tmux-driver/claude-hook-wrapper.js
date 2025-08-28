#!/usr/bin/env node
// claude-hook-wrapper.js
// Claudeバイナリにフックをかけて入出力を横取りするラッパー

const { spawn } = require('child_process');
const path = require('path');
let WebSocket;
try {
  WebSocket = require('ws');
} catch (e) {
  console.error('FATAL: Cannot find module "ws"');
  console.error('Hint: run "npm install" inside tools/codex-tmux-driver');
  process.exit(1);
}
const fs = require('fs');

// 設定
const REAL_CLAUDE = process.env.CLAUDE_REAL_BIN || '/home/tomoaki/.volta/tools/image/node/22.16.0/bin/claude';
const HOOK_SERVER = process.env.CLAUDE_HOOK_SERVER || 'ws://localhost:8770';
const LOG_FILE = process.env.CLAUDE_LOG_FILE || '/tmp/claude-hook.log';
const ENABLE_HOOK = process.env.CLAUDE_HOOK_ENABLE !== 'false';
const USE_SCRIPT_PTY = process.env.CLAUDE_USE_SCRIPT_PTY !== 'false'; // デフォルトでPTY有効

// WebSocket接続
let ws = null;
if (ENABLE_HOOK) {
  console.error(`[claude-hook] Attempting to connect to ${HOOK_SERVER}...`);
  try {
    ws = new WebSocket(HOOK_SERVER);
    ws.on('open', () => {
      log('hook-connect', { url: HOOK_SERVER });
      console.error(`[claude-hook] ✅ Successfully connected to ${HOOK_SERVER}`);
    });
    ws.on('error', (e) => {
      console.error(`[claude-hook] ❌ Connection error: ${e?.message || e}`);
    });
    ws.on('close', () => {
      console.error(`[claude-hook] 🔌 Connection closed`);
    });
  } catch (e) {
    console.error(`[claude-hook] ❌ Failed to create WebSocket: ${e}`);
  }
}

// ログ関数
function log(type, data) {
  const timestamp = new Date().toISOString();
  const logEntry = { timestamp, type, data };
  
  // ファイルログ
  fs.appendFileSync(LOG_FILE, JSON.stringify(logEntry) + '\n');
  
  // WebSocket送信
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(logEntry));
  }
}

// Claudeプロセス起動
if (!fs.existsSync(REAL_CLAUDE)) {
  console.error(`FATAL: REAL_CLAUDE not found: ${REAL_CLAUDE}`);
  console.error('Set CLAUDE_REAL_BIN to the real Claude binary path.');
  process.exit(1);
}

// 引数を渡してClaude起動
const userArgs = process.argv.slice(2);

// script(1) を使って擬似TTY経由で起動（Claudeはインタラクティブモードに必要）
function shEscape(s) { return `'${String(s).replace(/'/g, `'\\''`)}'`; }
let claudeProcess;
let usingPty = false;

try {
  if (USE_SCRIPT_PTY) {
    const cmdStr = [REAL_CLAUDE, ...userArgs].map(shEscape).join(' ');
    // -q: quiet, -f: flush, -e: return child exit code, -c: command
    claudeProcess = spawn('script', ['-qfec', cmdStr, '/dev/null'], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: process.env
    });
    usingPty = true;
    log('start-info', { mode: 'pty(script)', cmd: cmdStr });
  }
} catch (e) {
  // フォールバック
  console.error(`[claude-hook] PTY spawn failed: ${e.message}`);
}

if (!claudeProcess) {
  claudeProcess = spawn(REAL_CLAUDE, userArgs, {
    stdio: ['pipe', 'pipe', 'pipe'],
    env: process.env
  });
  log('start-info', { mode: 'pipe', bin: REAL_CLAUDE, args: userArgs });
}

// 標準入力をClaudeへ
process.stdin.on('data', (chunk) => {
  const input = chunk.toString();
  log('input', input);
  claudeProcess.stdin.write(chunk);
});

// 標準出力
let outputBuffer = '';
claudeProcess.stdout.on('data', (chunk) => {
  const data = chunk.toString();
  outputBuffer += data;
  
  // 改行で区切って出力をログ
  if (data.includes('\n')) {
    log('output', outputBuffer);
    outputBuffer = '';
  }
  
  process.stdout.write(chunk);
});

// エラー出力
claudeProcess.stderr.on('data', (chunk) => {
  log('error', chunk.toString());
  process.stderr.write(chunk);
});

// プロセス終了
claudeProcess.on('exit', (code, signal) => {
  log('exit', { code, signal });
  
  if (ws) {
    try {
      ws.send(JSON.stringify({
        type: 'hook-event',
        event: 'claude-exit',
        data: { code, signal }
      }));
    } catch {}
    ws.close();
  }
  
  process.exit(typeof code === 'number' ? code : 0);
});

// シグナルハンドリング
process.on('SIGINT', () => {
  claudeProcess.kill('SIGINT');
});

process.on('SIGTERM', () => {
  claudeProcess.kill('SIGTERM');
});

// WebSocketからのメッセージ受信
if (ws) {
  ws.on('message', (data) => {
    try {
      const cmd = JSON.parse(data.toString());
      
      if (cmd.type === 'inject-input') {
        // Claudeに入力を注入
        log('inject', cmd.data);
        claudeProcess.stdin.write(cmd.data + '\n');
      }
    } catch (e) {
      console.error(`[claude-hook] ❌ Error parsing message: ${e}`);
    }
  });
}

// 起動ログ
log('start', {
  args: userArgs,
  pid: process.pid,
  hookEnabled: ENABLE_HOOK,
  usingPty
});

console.error(`[claude-hook] active (pty=${usingPty ? 'on' : 'off'}) REAL_CLAUDE=${REAL_CLAUDE}`);