// tmux-perfect-bridge.js
// tmux × tmux = 完璧な双方向自動ブリッジ！

const { spawn } = require('child_process');
const EventEmitter = require('events');

class TmuxPerfectBridge extends EventEmitter {
  constructor() {
    super();
    this.codexSession = 'codex-bridge';
    this.claudeSession = 'claude-bridge';
    this.isRunning = false;
    this.lastCodexOutput = '';
    this.lastClaudeOutput = '';
  }

  // 両方のAIをtmuxで起動
  async start() {
    console.log('🚀 Starting Perfect Bridge...');
    
    // Codexを起動
    await this.startSession(this.codexSession, '/home/tomoaki/.volta/bin/codex');
    
    // Claudeを起動（仮のコマンド）
    // await this.startSession(this.claudeSession, 'claude');
    
    console.log('✅ Both AIs are ready in tmux!');
    this.isRunning = true;
  }

  // tmuxセッションを起動
  async startSession(sessionName, command) {
    // 既存セッションを削除
    await this.exec('tmux', ['kill-session', '-t', sessionName]).catch(() => {});
    
    // 新規セッション作成
    await this.exec('tmux', ['new-session', '-d', '-s', sessionName, command]);
    console.log(`📺 Started ${sessionName}`);
    
    // 起動待ち
    await this.sleep(2000);
  }

  // Codex → Claude 転送
  async forwardCodexToClaude() {
    const codexOutput = await this.capturePane(this.codexSession);
    const newContent = this.extractNewContent(codexOutput, this.lastCodexOutput);
    
    if (newContent && this.isCodexResponse(newContent)) {
      console.log('📨 Codex → Claude:', newContent.substring(0, 50) + '...');
      
      // tmux send-keysで直接送信！Enterも完璧！
      await this.sendToSession(this.claudeSession, newContent);
      
      this.lastCodexOutput = codexOutput;
      this.emit('codex-to-claude', newContent);
    }
  }

  // Claude → Codex 転送
  async forwardClaudeToCodex() {
    const claudeOutput = await this.capturePane(this.claudeSession);
    const newContent = this.extractNewContent(claudeOutput, this.lastClaudeOutput);
    
    if (newContent && this.isClaudeResponse(newContent)) {
      console.log('📨 Claude → Codex:', newContent.substring(0, 50) + '...');
      
      // tmux send-keysで直接送信！Enterも完璧！
      await this.sendToSession(this.codexSession, newContent);
      
      this.lastClaudeOutput = claudeOutput;
      this.emit('claude-to-codex', newContent);
    }
  }

  // 双方向監視ループ
  async startWatching(intervalMs = 1000) {
    console.log('👁️ Starting bidirectional watch...');
    
    const watchLoop = setInterval(async () => {
      if (!this.isRunning) {
        clearInterval(watchLoop);
        return;
      }
      
      try {
        // 両方向をチェック
        await this.forwardCodexToClaude();
        await this.forwardClaudeToCodex();
      } catch (err) {
        console.error('❌ Watch error:', err);
      }
    }, intervalMs);
  }

  // tmuxペインをキャプチャ
  async capturePane(sessionName) {
    const result = await this.exec('tmux', ['capture-pane', '-t', sessionName, '-p']);
    return result.stdout;
  }

  // tmuxセッションに送信（Enterも！）
  async sendToSession(sessionName, text) {
    await this.exec('tmux', ['send-keys', '-t', sessionName, text, 'Enter']);
  }

  // 新しいコンテンツを抽出
  extractNewContent(current, previous) {
    // 簡単な差分検出（実際はもっと高度にする）
    if (current.length > previous.length) {
      return current.substring(previous.length).trim();
    }
    return null;
  }

  // Codexの応答かどうか判定
  isCodexResponse(text) {
    return !text.includes('Working') && 
           !text.includes('▌') && 
           text.length > 10;
  }

  // Claudeの応答かどうか判定
  isClaudeResponse(text) {
    // Claudeの出力パターンに応じて調整
    return text.length > 10;
  }

  // 初期メッセージを送信
  async sendInitialMessage(message) {
    console.log('🎯 Sending initial message to Codex...');
    await this.sendToSession(this.codexSession, message);
  }

  // 両セッションを表示（デバッグ用）
  showSessions() {
    console.log('\n📺 Showing both sessions side by side...');
    spawn('tmux', [
      'new-window', '-n', 'AI-Bridge',
      `tmux select-pane -t 0 \\; \
       attach-session -t ${this.codexSession} \\; \
       split-window -h \\; \
       attach-session -t ${this.claudeSession}`
    ], { stdio: 'inherit' });
  }

  // 停止
  async stop() {
    this.isRunning = false;
    await this.exec('tmux', ['kill-session', '-t', this.codexSession]).catch(() => {});
    await this.exec('tmux', ['kill-session', '-t', this.claudeSession]).catch(() => {});
    console.log('👋 Bridge stopped');
  }

  // ヘルパー関数
  exec(command, args) {
    return new Promise((resolve, reject) => {
      const proc = spawn(command, args);
      let stdout = '';
      let stderr = '';
      
      proc.stdout.on('data', (data) => stdout += data);
      proc.stderr.on('data', (data) => stderr += data);
      
      proc.on('close', (code) => {
        if (code !== 0 && !stderr.includes('no server running')) {
          reject(new Error(`${command} exited with code ${code}: ${stderr}`));
        } else {
          resolve({ stdout, stderr });
        }
      });
    });
  }

  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// デモ実行
async function demo() {
  const bridge = new TmuxPerfectBridge();
  
  // イベントリスナー
  bridge.on('codex-to-claude', (content) => {
    console.log('🔄 Transferred from Codex to Claude');
  });
  
  bridge.on('claude-to-codex', (content) => {
    console.log('🔄 Transferred from Claude to Codex');
  });
  
  try {
    // ブリッジ開始
    await bridge.start();
    
    // 初期メッセージ
    await bridge.sendInitialMessage('Nyashプロジェクトについて、お互いに意見を交換してください');
    
    // 監視開始
    await bridge.startWatching(500);
    
    // デバッグ用に画面表示
    // bridge.showSessions();
    
  } catch (err) {
    console.error('❌ Error:', err);
  }
}

// エクスポート
module.exports = TmuxPerfectBridge;

// 直接実行
if (require.main === module) {
  demo();
}