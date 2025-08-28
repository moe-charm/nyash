#!/usr/bin/env node
// codex-hook-wrapper.js
// Codexバイナリにフックをかけて入出力を横取りするラッパー
// 使い方: このファイルを codex として PATH に配置

const { spawn } = require('child_process');
const path = require('path');
let WebSocket;
try {
  WebSocket = require('ws');
} catch (e) {
  console.error('FATAL: Cannot find module "ws"');
  console.error('Hint: run "npm install" inside tools/codex-tmux-driver, or ensure the wrapper is symlinked to that directory.');
  process.exit(1);
}
const fs = require('fs');

// 設定
// 実バイナリは環境変数で上書き可能。未設定かつ存在しない場合はエラーにする。
const REAL_CODEX = process.env.CODEX_REAL_BIN || '/home/tomoaki/.volta/tools/image/packages/@openai/codex/lib/node_modules/@openai/codex/bin/codex-x86_64-unknown-linux-musl';
const HOOK_SERVER = process.env.CODEX_HOOK_SERVER || 'ws://localhost:8770';
const LOG_FILE = process.env.CODEX_LOG_FILE || '/tmp/codex-hook.log';
const ENTER_MODE = (process.env.CODEX_HOOK_ENTER || 'crlf').toLowerCase(); // lf|cr|crlf
const ENABLE_HOOK = process.env.CODEX_HOOK_ENABLE !== 'false';
const USE_SCRIPT_PTY = process.env.CODEX_USE_SCRIPT_PTY === 'true'; // default false
const SHOW_BANNER = process.env.CODEX_HOOK_BANNER !== 'false';
const ECHO_INJECT = process.env.CODEX_HOOK_ECHO_INJECT === 'true';
const PRE_NEWLINE = process.env.CODEX_HOOK_PRENEWLINE === 'true';
const INJECT_PREFIX = process.env.CODEX_HOOK_INJECT_PREFIX || '';
const INJECT_SUFFIX = process.env.CODEX_HOOK_INJECT_SUFFIX || '';

// WebSocket接続（オプショナル）
let ws = null;
if (ENABLE_HOOK) {
  console.error(`[codex-hook] Attempting to connect to ${HOOK_SERVER}...`);
  try {
    ws = new WebSocket(HOOK_SERVER);
    ws.on('open', () => {
      // 目印ログ（接続先確認）
      log('hook-connect', { url: HOOK_SERVER });
      console.error(`[codex-hook] ✅ Successfully connected to ${HOOK_SERVER}`);
    });
    ws.on('error', (e) => {
      // 接続エラーは無視（フォールバック動作）
      console.error(`[codex-hook] ❌ Connection error: ${e?.message || e}`);
    });
    ws.on('close', () => {
      console.error(`[codex-hook] 🔌 Connection closed`);
    });
  } catch (e) {
    // WebSocketサーバーが起動していない場合は通常動作
    console.error(`[codex-hook] ❌ Failed to create WebSocket: ${e}`);
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

// Codexプロセス起動
if (!fs.existsSync(REAL_CODEX)) {
  console.error(`FATAL: REAL_CODEX not found: ${REAL_CODEX}`);
  console.error('Set CODEX_REAL_BIN to the real Codex binary path.');
  process.exit(1);
}

// 引数決定（無指定時は既定コマンドを許可）
let userArgs = process.argv.slice(2);
const DEFAULT_CMD = process.env.CODEX_WRAPPER_DEFAULT_CMD; // 例: "exec --ask-for-approval never"
if (userArgs.length === 0 && DEFAULT_CMD) {
  try {
    userArgs = DEFAULT_CMD.split(' ').filter(Boolean);
    if (SHOW_BANNER) {
      console.error(`[codex-hook] using default cmd: ${DEFAULT_CMD}`);
    }
  } catch {}
}

// script(1) を使って擬似TTY経由で起動（インジェクションを確実に通すため）
function shEscape(s) { return `'${String(s).replace(/'/g, `'\''`)}'`; }
let codexProcess;
let usingPty = false;
try {
  if (USE_SCRIPT_PTY) {
    const cmdStr = [REAL_CODEX, ...userArgs].map(shEscape).join(' ');
    // -q: quiet, -f: flush, -e: return child exit code, -c: command
    codexProcess = spawn('script', ['-qfec', cmdStr, '/dev/null'], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: process.env
    });
    usingPty = true;
    log('start-info', { mode: 'pty(script)', cmd: cmdStr });
  }
} catch (e) {
  // フォールバック
}

if (!codexProcess) {
  codexProcess = spawn(REAL_CODEX, userArgs, {
    stdio: ['pipe', 'pipe', 'pipe'],
    env: process.env
  });
  log('start-info', { mode: 'pipe', bin: REAL_CODEX, args: process.argv.slice(2) });
}

// 入力フック（標準入力 → Codex）
let inputBuffer = '';
process.stdin.on('data', (chunk) => {
  const data = chunk.toString();
  inputBuffer += data;
  
  // 改行で区切って入力を記録
  if (data.includes('\n')) {
    const lines = inputBuffer.split('\n');
    inputBuffer = lines.pop() || '';
    
    lines.forEach(line => {
      if (line.trim()) {
        log('input', line);
        
        // 入力パターン検出
        if (ENABLE_HOOK) {
          detectInputPattern(line);
        }
      }
    });
  }
  
  // そのままCodexに転送
  codexProcess.stdin.write(chunk);
});

// 出力フック（Codex → 標準出力）
let outputBuffer = '';
codexProcess.stdout.on('data', (chunk) => {
  const data = chunk.toString();
  outputBuffer += data;
  
  // バッファリングして意味のある単位で記録
  if (data.includes('\n') || data.includes('▌')) {
    log('output', outputBuffer);
    
    // 出力パターン検出
    if (ENABLE_HOOK) {
      detectOutputPattern(outputBuffer);
    }
    
    outputBuffer = '';
  }
  
  // そのまま標準出力へ
  process.stdout.write(chunk);
});

// エラー出力
codexProcess.stderr.on('data', (chunk) => {
  log('error', chunk.toString());
  process.stderr.write(chunk);
});

// プロセス終了
codexProcess.on('exit', (code, signal) => {
  log('exit', { code, signal });
  
  if (ws) {
    try {
      ws.send(JSON.stringify({
        type: 'hook-event',
        event: 'codex-exit',
        data: { code, signal }
      }));
    } catch {}
    ws.close();
  }
  
  process.exit(typeof code === 'number' ? code : 0);
});

// 入力パターン検出
function detectInputPattern(input) {
  const patterns = {
    question: /\?$|どうしますか|どう思いますか/,
    command: /^(status|help|exit|clear)/,
    code: /^(function|box|if|for|while|return)/
  };
  
  for (const [type, pattern] of Object.entries(patterns)) {
    if (pattern.test(input)) {
      log('input-pattern', { type, input });
      
      // 特定パターンでの自動介入
      if (type === 'question' && ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
          type: 'hook-event',
          event: 'question-detected',
          data: input
        }));
      }
    }
  }
}

// 出力パターン検出
function detectOutputPattern(output) {
  const patterns = {
    thinking: /考え中|Processing|Thinking|分析中/,
    complete: /完了|Complete|Done|終了/,
    error: /エラー|Error|失敗|Failed/,
    waiting: /waiting|待機中|入力待ち|▌/
  };
  
  for (const [type, pattern] of Object.entries(patterns)) {
    if (pattern.test(output)) {
      log('output-pattern', { type, output: output.substring(0, 100) });
      
      // 待機状態での介入ポイント
      if (type === 'waiting' && ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
          type: 'hook-event',
          event: 'waiting-detected',
          data: output
        }));
      }
    }
  }
}

// シグナルハンドリング
process.on('SIGINT', () => {
  codexProcess.kill('SIGINT');
});

process.on('SIGTERM', () => {
  codexProcess.kill('SIGTERM');
});

// WebSocketからの介入コマンド受信
if (ws) {
  ws.on('message', (data) => {
    try {
      const cmd = JSON.parse(data.toString());
      
      if (cmd.type === 'inject-input') {
        // Codexに入力を注入
        log('inject', cmd.data);
        // Enterの扱いは環境依存のため、モードで切替（デフォルト: crlf）
        let eol = '\r\n';
        if (ENTER_MODE === 'lf') eol = '\n';
        else if (ENTER_MODE === 'cr') eol = '\r';
        try {
          const payload = `${INJECT_PREFIX}${cmd.data}${INJECT_SUFFIX}`;
          if (PRE_NEWLINE) {
            codexProcess.stdin.write('\n');
          }
          const written = codexProcess.stdin.write(payload + eol);
          if (ECHO_INJECT) {
            // メッセージだけをシンプルに表示
            process.stdout.write(`\n\n${payload}\n`);
          }
        } catch (e) {
          console.error(`[codex-hook] ❌ Error writing to stdin: ${e}`);
          log('inject-error', e?.message || String(e));
        }
      }
    } catch (e) {
      console.error(`[codex-hook] ❌ Error parsing message: ${e}`);
    }
  });
}

// 起動ログ
log('start', {
  args: process.argv.slice(2),
  pid: process.pid,
  hookEnabled: ENABLE_HOOK,
  usingPty
});

if (SHOW_BANNER) {
  console.error(`[codex-hook] active (pty=${usingPty ? 'on' : 'off'} enter=${ENTER_MODE}) REAL_CODEX=${REAL_CODEX}`);
}
