// codex-tmux-driver.js
// tmux経由でCodexを管理し、イベントをWebSocket配信するドライバ
// 使い方:
//   1) npm install
//   2) node codex-tmux-driver.js [options]
//      --session=codex-session  (tmuxセッション名)
//      --port=8766             (WebSocketポート)
//      --log=/tmp/codex.log    (ログファイルパス)
//   3) WebSocketで接続して操作
//      {"op":"send","data":"質問内容"}
//      {"op":"capture"}
//      {"op":"status"}

const { spawn } = require('child_process');
const { WebSocketServer } = require('ws');
const fs = require('fs');
const path = require('path');

// --- 設定 ---
const argv = process.argv.slice(2).reduce((a, kv) => {
  const [k, ...rest] = kv.split('=');
  const v = rest.join('=');
  a[k.replace(/^--/, '')] = v ?? true;
  return a;
}, {});

const SESSION_NAME = argv.session || 'codex-session';
const PORT = Number(argv.port || 8766);
const LOG_FILE = argv.log || `/tmp/codex-${Date.now()}.log`;
const CODEX_CMD = argv.cmd || 'codex exec';

// --- 状態管理 ---
let clients = new Set();
let tailProcess = null;
let sessionActive = false;
let codexEventBuffer = [];
const MAX_BUFFER_SIZE = 100;

// --- ユーティリティ関数 ---
function broadcast(msg) {
  const data = JSON.stringify(msg);
  for (const ws of clients) {
    try { ws.send(data); } catch {}
  }
}

function executeCommand(cmd, args = []) {
  return new Promise((resolve, reject) => {
    const proc = spawn(cmd, args);
    let stdout = '';
    let stderr = '';
    
    proc.stdout.on('data', (data) => { stdout += data; });
    proc.stderr.on('data', (data) => { stderr += data; });
    
    proc.on('close', (code) => {
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(new Error(`Command failed: ${stderr}`));
      }
    });
  });
}

// --- Codex出力パターン認識 ---
function parseCodexOutput(line) {
  const patterns = {
    // Codexの応答パターン
    response: /^(Codex:|回答:|Answer:|Response:)/i,
    thinking: /(考え中|Processing|Thinking|分析中)/i,
    error: /(エラー|Error|失敗|Failed)/i,
    complete: /(完了|Complete|Done|終了)/i,
    question: /(質問:|Question:|相談:|Help:)/i,
  };

  for (const [event, pattern] of Object.entries(patterns)) {
    if (pattern.test(line)) {
      return {
        type: 'codex-event',
        event: event,
        timestamp: new Date().toISOString(),
        data: line.trim()
      };
    }
  }

  // パターンに一致しない場合は生データとして返す
  return {
    type: 'codex-output',
    timestamp: new Date().toISOString(),
    data: line
  };
}

// --- tmuxセッション管理 ---
async function createTmuxSession() {
  try {
    // 既存セッションをチェック
    try {
      await executeCommand('tmux', ['has-session', '-t', SESSION_NAME]);
      console.log(`[INFO] Session ${SESSION_NAME} already exists`);
      sessionActive = true;
      // 既存セッションでもパイプと監視を確実に有効化する
      try {
        await executeCommand('tmux', [
          'pipe-pane', '-t', SESSION_NAME,
          '-o', `cat >> ${LOG_FILE}`
        ]);
      } catch (e) {
        console.warn('[WARN] Failed to ensure pipe-pane on existing session:', e.message || e);
      }
      if (!tailProcess) {
        startLogMonitoring();
      }
      return;
    } catch {
      // セッションが存在しない場合、作成
    }

    // 新規セッション作成
    await executeCommand('tmux', [
      'new-session', '-d', '-s', SESSION_NAME,
      CODEX_CMD
    ]);
    
    // pipe-paneでログ出力を設定
    await executeCommand('tmux', [
      'pipe-pane', '-t', SESSION_NAME,
      '-o', `cat >> ${LOG_FILE}`
    ]);
    
    sessionActive = true;
    console.log(`[INFO] Created tmux session: ${SESSION_NAME}`);
    
    // ログファイル監視開始
    startLogMonitoring();
    
  } catch (err) {
    console.error('[ERROR] Failed to create tmux session:', err);
    throw err;
  }
}

// --- ログ監視 ---
function startLogMonitoring() {
  // ログファイルが存在しない場合は作成
  if (!fs.existsSync(LOG_FILE)) {
    fs.writeFileSync(LOG_FILE, '');
  }

  // tail -fで監視
  tailProcess = spawn('tail', ['-f', '-n', '0', LOG_FILE]);
  
  tailProcess.stdout.on('data', (data) => {
    const lines = data.toString('utf8').split('\n').filter(Boolean);
    
    for (const line of lines) {
      const event = parseCodexOutput(line);
      
      // イベントバッファに追加
      codexEventBuffer.push(event);
      if (codexEventBuffer.length > MAX_BUFFER_SIZE) {
        codexEventBuffer.shift();
      }
      
      // クライアントに配信
      broadcast(event);
    }
  });
  
  tailProcess.on('error', (err) => {
    console.error('[ERROR] Tail process error:', err);
  });
}

// --- WebSocketサーバ ---
const wss = new WebSocketServer({ port: PORT });

wss.on('connection', (ws) => {
  clients.add(ws);
  
  // 接続時にステータスと最近のイベントを送信
  ws.send(JSON.stringify({
    type: 'status',
    data: {
      session: SESSION_NAME,
      active: sessionActive,
      logFile: LOG_FILE,
      recentEvents: codexEventBuffer.slice(-10)
    }
  }));
  
  ws.on('message', async (raw) => {
    let msg;
    try {
      msg = JSON.parse(raw.toString());
    } catch {
      ws.send(JSON.stringify({ type: 'error', data: 'Invalid JSON' }));
      return;
    }
    
    switch (msg.op) {
      case 'send': {
        // Codexに入力を送信
        if (!sessionActive) {
          ws.send(JSON.stringify({ type: 'error', data: 'Session not active' }));
          break;
        }
        
        const input = String(msg.data || '').trim();
        if (!input) break;
        
        try {
          await executeCommand('tmux', [
            'send-keys', '-t', SESSION_NAME,
            input, 'Enter'
          ]);
          
          broadcast({
            type: 'input-sent',
            timestamp: new Date().toISOString(),
            data: input
          });
        } catch (err) {
          ws.send(JSON.stringify({ 
            type: 'error', 
            data: 'Failed to send input' 
          }));
        }
        break;
      }
      
      case 'capture': {
        // 現在の画面をキャプチャ
        if (!sessionActive) {
          ws.send(JSON.stringify({ type: 'error', data: 'Session not active' }));
          break;
        }
        
        try {
          const output = await executeCommand('tmux', [
            'capture-pane', '-t', SESSION_NAME, '-p'
          ]);
          
          ws.send(JSON.stringify({
            type: 'screen-capture',
            timestamp: new Date().toISOString(),
            data: output
          }));
        } catch (err) {
          ws.send(JSON.stringify({ 
            type: 'error', 
            data: 'Failed to capture screen' 
          }));
        }
        break;
      }
      
      case 'status': {
        // ステータス確認
        ws.send(JSON.stringify({
          type: 'status',
          data: {
            session: SESSION_NAME,
            active: sessionActive,
            logFile: LOG_FILE,
            clientCount: clients.size,
            bufferSize: codexEventBuffer.length
          }
        }));
        break;
      }
      
      case 'history': {
        // イベント履歴取得
        const count = Number(msg.count || 20);
        const history = codexEventBuffer.slice(-count);
        
        ws.send(JSON.stringify({
          type: 'history',
          data: history
        }));
        break;
      }
      
      case 'filter': {
        // 特定のイベントタイプのみフィルタ
        const eventType = msg.event || 'all';
        const filtered = eventType === 'all' 
          ? codexEventBuffer
          : codexEventBuffer.filter(e => e.event === eventType);
        
        ws.send(JSON.stringify({
          type: 'filtered-events',
          filter: eventType,
          data: filtered
        }));
        break;
      }
      
      case 'kill': {
        // セッション終了
        if (!sessionActive) break;
        
        try {
          await executeCommand('tmux', ['kill-session', '-t', SESSION_NAME]);
          sessionActive = false;
          if (tailProcess) {
            try { tailProcess.kill(); } catch {}
            tailProcess = null;
          }
          broadcast({ type: 'session-killed' });
        } catch (err) {
          ws.send(JSON.stringify({ 
            type: 'error', 
            data: 'Failed to kill session' 
          }));
        }
        break;
      }
      
      default: {
        ws.send(JSON.stringify({ 
          type: 'error', 
          data: `Unknown operation: ${msg.op}` 
        }));
      }
    }
  });
  
  ws.on('close', () => {
    clients.delete(ws);
  });
});

// --- 起動処理 ---
async function start() {
  console.log('=== Codex tmux Driver ===');
  console.log(`WebSocket: ws://localhost:${PORT}`);
  console.log(`Session: ${SESSION_NAME}`);
  console.log(`Log file: ${LOG_FILE}`);
  
  try {
    await createTmuxSession();
    
    wss.on('listening', () => {
      console.log(`[INFO] WebSocket server listening on port ${PORT}`);
    });
  } catch (err) {
    console.error('[FATAL] Failed to start:', err);
    process.exit(1);
  }
}

// --- クリーンアップ ---
process.on('SIGINT', () => {
  console.log('\n[INFO] Shutting down...');
  if (tailProcess) {
    tailProcess.kill();
  }
  process.exit(0);
});

// 起動
start();
