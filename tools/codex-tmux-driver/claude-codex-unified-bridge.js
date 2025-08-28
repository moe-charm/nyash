// claude-codex-unified-bridge.js
// 同一hook-serverを使った完璧な双方向ブリッジ！

const { spawn } = require('child_process');
const WebSocket = require('ws');
const EventEmitter = require('events');

class ClaudeCodexUnifiedBridge extends EventEmitter {
  constructor(config = {}) {
    super();
    this.config = {
      hookServer: config.hookServer || 'ws://localhost:8770',
      claudeSession: config.claudeSession || 'claude-8771',
      codexSession: config.codexSession || 'codex-safe',
      watchInterval: config.watchInterval || 500,
      ...config
    };
    
    this.ws = null;
    this.isRunning = false;
    this.lastClaudeOutput = '';
    this.lastCodexOutput = '';
  }

  // ブリッジ開始
  async start() {
    console.log('🌉 Starting Claude-Codex Unified Bridge...');
    console.log('📡 Hook Server:', this.config.hookServer);
    
    // WebSocket接続
    await this.connectToHookServer();
    
    // 監視開始
    this.isRunning = true;
    this.startWatching();
    
    console.log('✅ Bridge is running!');
  }

  // hook-serverに接続
  connectToHookServer() {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.config.hookServer);
      
      this.ws.on('open', () => {
        console.log('✅ Connected to hook-server');
        
        // ブリッジとして登録
        this.ws.send(JSON.stringify({
          source: 'bridge',
          type: 'register',
          data: 'claude-codex-bridge'
        }));
        
        resolve();
      });
      
      this.ws.on('error', (err) => {
        console.error('❌ WebSocket error:', err);
        reject(err);
      });
      
      this.ws.on('close', () => {
        console.log('🔌 Disconnected from hook-server');
        this.isRunning = false;
      });
    });
  }

  // 監視ループ
  startWatching() {
    const watchLoop = setInterval(async () => {
      if (!this.isRunning) {
        clearInterval(watchLoop);
        return;
      }
      
      try {
        // Codexの出力をチェック
        await this.checkCodexOutput();
        
        // Claudeの出力もチェック（必要に応じて）
        // await this.checkClaudeOutput();
        
      } catch (err) {
        console.error('❌ Watch error:', err);
      }
    }, this.config.watchInterval);
  }

  // Codexの出力をチェック
  async checkCodexOutput() {
    const output = await this.capturePane(this.config.codexSession);
    
    // 新しい内容があるかチェック
    if (output !== this.lastCodexOutput) {
      const newContent = this.extractNewContent(output, this.lastCodexOutput);
      
      if (newContent && this.isCodexResponse(newContent)) {
        console.log('📨 Codex response detected!');
        
        // Claudeに転送
        this.sendToClaude(newContent);
        
        this.lastCodexOutput = output;
      }
    }
  }

  // Claudeにメッセージを送信（hook-server経由）
  sendToClaude(message) {
    console.log('📤 Sending to Claude via hook-server...');
    
    const payload = {
      source: 'codex',
      type: 'inject-input',
      data: `[Codex Response]\n${message}`
    };
    
    this.ws.send(JSON.stringify(payload));
    
    this.emit('codex-to-claude', message);
  }

  // tmuxペインをキャプチャ
  capturePane(sessionName) {
    return new Promise((resolve, reject) => {
      const proc = spawn('tmux', ['capture-pane', '-t', sessionName, '-p']);
      let output = '';
      
      proc.stdout.on('data', (data) => output += data);
      proc.on('close', (code) => {
        if (code === 0) {
          resolve(output);
        } else {
          reject(new Error(`tmux capture failed with code ${code}`));
        }
      });
    });
  }

  // 新しいコンテンツを抽出
  extractNewContent(current, previous) {
    if (current.length > previous.length) {
      return current.substring(previous.length).trim();
    }
    return null;
  }

  // Codexの応答かどうか判定
  isCodexResponse(text) {
    // Working状態でない、プロンプトでない、十分な長さ
    return !text.includes('Working') && 
           !text.includes('▌') && 
           text.length > 20 &&
           !text.includes('⏎ send');
  }

  // 停止
  stop() {
    this.isRunning = false;
    if (this.ws) {
      this.ws.close();
    }
    console.log('🛑 Bridge stopped');
  }
}

// メイン実行
if (require.main === module) {
  const bridge = new ClaudeCodexUnifiedBridge();
  
  // イベントリスナー
  bridge.on('codex-to-claude', (content) => {
    console.log('📊 Transferred to Claude:', content.substring(0, 50) + '...');
  });
  
  // 開始
  bridge.start().catch(console.error);
  
  // 終了処理
  process.on('SIGINT', () => {
    console.log('\n👋 Shutting down...');
    bridge.stop();
    process.exit(0);
  });
}

module.exports = ClaudeCodexUnifiedBridge;