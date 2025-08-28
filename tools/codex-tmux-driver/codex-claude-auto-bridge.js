// codex-claude-auto-bridge.js
// CodexとClaudeを自動で橋渡しするシステム

const fs = require('fs').promises;
const path = require('path');
const TmuxCodexController = require('./tmux-codex-controller');
const CodexOutputWatcher = require('./codex-output-watcher');

class CodexClaudeAutoBridge {
  constructor(config = {}) {
    this.config = {
      sessionName: config.sessionName || 'codex-safe',
      outputFile: config.outputFile || './codex-response.txt',
      logFile: config.logFile || './bridge.log',
      watchInterval: config.watchInterval || 500,
      ...config
    };
    
    this.controller = new TmuxCodexController(this.config.sessionName);
    this.watcher = new CodexOutputWatcher(this.config.sessionName);
    this.isRunning = false;
  }

  // ブリッジを開始
  async start() {
    console.log('🌉 Starting Codex-Claude Auto Bridge...');
    this.isRunning = true;
    
    // 出力ウォッチャーのイベント設定
    this.watcher.on('response', async (response) => {
      await this.handleCodexResponse(response);
    });
    
    this.watcher.on('ready', () => {
      console.log('💚 Codex is ready for next input');
    });
    
    // 監視開始
    this.watcher.start(this.config.watchInterval);
    
    await this.log('Bridge started');
  }

  // Codexの応答を処理
  async handleCodexResponse(response) {
    console.log('\n📝 Got Codex response!');
    
    // 応答をファイルに保存（Claudeが読めるように）
    await this.saveResponse(response);
    
    // ログに記録
    await this.log(`Codex response: ${response.substring(0, 100)}...`);
    
    // 通知
    console.log('✅ Response saved to:', this.config.outputFile);
    console.log('📢 Please read the response file and send next message to Codex!');
    
    // 自動応答モードの場合（オプション）
    if (this.config.autoReply) {
      await this.sendAutoReply();
    }
  }

  // 応答をファイルに保存
  async saveResponse(response) {
    const timestamp = new Date().toISOString();
    const content = `=== Codex Response at ${timestamp} ===\n\n${response}\n\n`;
    
    await fs.writeFile(this.config.outputFile, content);
  }

  // Codexにメッセージを送信
  async sendToCodex(message) {
    console.log(`📤 Sending to Codex: "${message}"`);
    await this.controller.sendKeys(message, true); // Enterも送る
    await this.log(`Sent to Codex: ${message}`);
  }

  // 自動応答（実験的）
  async sendAutoReply() {
    // 簡単な自動応答ロジック
    const replies = [
      "なるほど！それについてもう少し詳しく教えて",
      "いい感じだにゃ！次はどうする？",
      "了解！他に何か提案はある？"
    ];
    
    const reply = replies[Math.floor(Math.random() * replies.length)];
    
    console.log(`🤖 Auto-replying in 3 seconds: "${reply}"`);
    setTimeout(async () => {
      await this.sendToCodex(reply);
    }, 3000);
  }

  // ログ記録
  async log(message) {
    const timestamp = new Date().toISOString();
    const logEntry = `[${timestamp}] ${message}\n`;
    
    await fs.appendFile(this.config.logFile, logEntry);
  }

  // 停止
  stop() {
    this.watcher.stop();
    this.isRunning = false;
    console.log('🛑 Bridge stopped');
  }
}

// CLIとして使う場合
if (require.main === module) {
  const bridge = new CodexClaudeAutoBridge({
    outputFile: './codex-response.txt',
    autoReply: false // 自動応答は無効
  });
  
  // 引数からメッセージを取得
  const initialMessage = process.argv.slice(2).join(' ');
  
  async function run() {
    // ブリッジ開始
    await bridge.start();
    
    // 初期メッセージがあれば送信
    if (initialMessage) {
      console.log('📨 Sending initial message...');
      await bridge.sendToCodex(initialMessage);
    } else {
      console.log('💡 Send a message to Codex using:');
      console.log('   tmux send-keys -t codex-safe "your message" Enter');
    }
    
    // Ctrl+Cで終了
    process.on('SIGINT', () => {
      console.log('\n👋 Shutting down...');
      bridge.stop();
      process.exit(0);
    });
  }
  
  run().catch(console.error);
}

module.exports = CodexClaudeAutoBridge;