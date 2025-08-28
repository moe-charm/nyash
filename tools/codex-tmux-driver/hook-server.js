#!/usr/bin/env node
// hook-server.js
// Codexフックからのイベントを受信してClaude連携するサーバー

const WebSocket = require('ws');
const fs = require('fs').promises;

const PORT = process.env.HOOK_SERVER_PORT || 8770;
const STRIP_ANSI = process.env.HOOK_STRIP_ANSI !== 'false';
const AUTO_BRIDGE = process.env.AUTO_BRIDGE === 'true';
const AUTO_EXIT = process.env.HOOK_SERVER_AUTO_EXIT === 'true';
const IDLE_EXIT_MS = Number(process.env.HOOK_IDLE_EXIT_MS || 2000);

// WebSocketサーバー
const wss = new WebSocket.Server({ port: PORT });

// 状態管理
const state = {
  lastInput: '',
  lastOutput: '',
  waitingCount: 0,
  questionQueue: [],
  // 接続クライアント: Map<WebSocket, 'hook' | 'control'>
  clients: new Map()
};

console.log(`🪝 Codex Hook Server listening on ws://localhost:${PORT}`);

wss.on('connection', (ws, req) => {
  const clientType = req.url === '/control' ? 'control' : 'hook';
  
  console.log(`📌 New ${clientType} connection`);
  state.clients.set(ws, clientType);
  
  ws.on('message', async (data) => {
    try {
      const msg = JSON.parse(data.toString());
      
      if (clientType === 'hook') {
        // Codexフックからのメッセージ
        await handleHookMessage(msg, ws);
      } else {
        // 制御クライアントからのメッセージ
        await handleControlMessage(ws, msg);
      }
    } catch (e) {
      console.error('Message error:', e);
    }
  });
  
  ws.on('close', () => {
    state.clients.delete(ws);
    maybeAutoExit();
  });
});

// ANSIエスケープ除去
function stripAnsi(s) {
  if (!STRIP_ANSI) return s;
  if (typeof s !== 'string') return s;
  // Robust ANSI/CSI/OSC sequences removal
  const ansiPattern = /[\u001B\u009B][[\]()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nq-uy=><]/g;
  return s.replace(ansiPattern, '');
}

// フックメッセージ処理
async function handleHookMessage(msg, senderWs) {
  const preview = typeof msg.data === 'string' ? stripAnsi(msg.data).substring(0, 80) : JSON.stringify(msg.data);
  console.log(`[${msg.type}] ${preview}`);
  
  // 全制御クライアントに転送
  broadcast('control', msg);
  
  switch (msg.type) {
    case 'input':
      state.lastInput = msg.data;
      break;
      
    case 'output':
      state.lastOutput = typeof msg.data === 'string' ? stripAnsi(msg.data) : msg.data;
      break;
      
    case 'hook-event':
      await handleHookEvent(msg);
      break;
      
    case 'inject-input':
      // フッククライアントからの入力注入リクエスト
      console.log('🔄 Relaying inject-input from hook client');

      // 明示的なターゲットがあればそれを最優先（tmuxセッション名を想定）
      if (msg.target && typeof msg.target === 'string') {
        const { spawn } = require('child_process');
        const text = String(msg.data ?? '');
        const targetSession = msg.target;
        console.log(`📤 Sending to explicit target via tmux: ${targetSession}`);

        // 文字列を通常の方法で送信
        const { exec } = require('child_process');
        const messageEscaped = text.replace(/'/g, "'\\''");
        await new Promise((resolve) => {
          exec(`tmux send-keys -t ${targetSession} '${messageEscaped}' Enter`, (error) => {
            if (error) console.error(`❌ tmux error: ${error.message}`);
            resolve();
          });
        });

        if (process.env.HOOK_SEND_CTRL_J === 'true') {
          await new Promise((resolve) => {
            const p = spawn('tmux', ['send-keys', '-t', targetSession, 'C-j']);
            p.on('close', () => resolve());
          });
        }

        console.log(`✅ Message + Enter sent to ${targetSession}`);
        break;
      }

      // 互換ルーティング（source から推測）
      let targetSession = 'claude';
      if (msg.source === 'codex') {
        targetSession = 'claude';
      } else if (msg.source === 'claude') {
        targetSession = 'codex';
      }

      console.log(`📤 Forwarding to ${targetSession}`);

      if (targetSession === 'claude') {
        // Claude想定：WebSocket経由でstdinに直接送信（注意: 全hookに送られる）
        console.log('🎯 Sending to Claude via WebSocket (stdin)');
        broadcast('hook', {
          type: 'inject-input',
          data: msg.data,
          target: 'claude'
        });
      } else {
        // Codex想定：tmux send-keys
        const { exec } = require('child_process');
        const text = String(msg.data ?? '');
        const messageEscaped = text.replace(/'/g, "'\\''");
        console.log(`📤 Sending to ${targetSession} via tmux`);
        await new Promise((resolve) => {
          exec(`tmux send-keys -t ${targetSession} '${messageEscaped}' Enter`, (error) => {
            if (error) console.error(`❌ tmux error: ${error.message}`);
            resolve();
          });
        });
        if (process.env.HOOK_SEND_CTRL_J === 'true') {
          await new Promise((resolve) => {
            const p = spawn('tmux', ['send-keys', '-t', targetSession, 'C-j']);
            p.on('close', () => resolve());
          });
        }
        console.log(`✅ Message + Enter sent to ${targetSession}`);
      }
      break;
  }
}

// フックイベント処理
async function handleHookEvent(msg) {
  switch (msg.event) {
    case 'question-detected':
      console.log('❓ Question detected:', msg.data);
      state.questionQueue.push({
        question: msg.data,
        timestamp: Date.now()
      });
      
      if (AUTO_BRIDGE) {
        // 自動ブリッジが有効なら応答を生成
        setTimeout(() => {
          injectResponse('考えさせてください...');
        }, 1000);
      }
      break;

    case 'waiting-detected':
      state.waitingCount++;
      console.log(`⏳ Waiting detected (count: ${state.waitingCount})`);
      
      // 3回連続で待機状態なら介入
      if (state.waitingCount >= 3 && AUTO_BRIDGE) {
        console.log('🚨 Auto-intervention triggered');
        injectResponse('続けてください');
        state.waitingCount = 0;
      }
      break;

    case 'codex-exit':
      console.log('🛑 Codex process exited');
      maybeAutoExit();
      break;
  }
}

// 制御メッセージ処理
async function handleControlMessage(ws, msg) {
  switch (msg.op) {
    case 'inject':
      injectResponse(msg.data);
      ws.send(JSON.stringify({ type: 'injected', data: msg.data }));
      break;
      
    case 'status':
      ws.send(JSON.stringify({
        type: 'status',
        state: {
          lastInput: state.lastInput,
          lastOutput: state.lastOutput.substring(0, 100),
          waitingCount: state.waitingCount,
          questionCount: state.questionQueue.length,
          clients: state.clients.size
        }
      }));
      break;
      
    case 'questions':
      ws.send(JSON.stringify({
        type: 'questions',
        data: state.questionQueue
      }));
      break;
  }
}

// Codexに応答を注入
function injectResponse(response) {
  console.log('💉 Injecting response:', response);
  
  // フッククライアントに注入コマンドを送信
  broadcast('hook', {
    type: 'inject-input',
    data: response
  });
}

// ブロードキャスト
function broadcast(clientType, message, excludeWs = null) {
  const data = JSON.stringify(message);
  let sentCount = 0;
  for (const [clientWs, type] of state.clients.entries()) {
    if (type === clientType && clientWs.readyState === WebSocket.OPEN) {
      if (excludeWs && clientWs === excludeWs) continue; // 送信元を除外
      clientWs.send(data);
      sentCount++;
    }
  }
  console.log(`📡 Broadcast to ${sentCount} ${clientType} clients`);
}

// フッククライアントがいなければ自動終了
let exitTimer = null;
function maybeAutoExit() {
  if (!AUTO_EXIT) return;
  const hasHook = Array.from(state.clients.values()).some(t => t === 'hook');
  if (hasHook) return;
  if (exitTimer) clearTimeout(exitTimer);
  exitTimer = setTimeout(() => {
    const hasHookNow = Array.from(state.clients.values()).some(t => t === 'hook');
    if (!hasHookNow) {
      console.log(`\n👋 No hook clients. Auto-exiting hook server (port ${PORT}).`);
      wss.close();
      process.exit(0);
    }
  }, IDLE_EXIT_MS);
}

// 統計情報の定期出力
setInterval(() => {
  console.log(`📊 Stats: Questions: ${state.questionQueue.length}, Waiting: ${state.waitingCount}, Clients: ${state.clients.size}`);
}, 60000);

// グレースフルシャットダウン
process.on('SIGINT', () => {
  console.log('\n👋 Shutting down hook server...');
  wss.close();
  process.exit(0);
});
