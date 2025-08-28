// tmux-codex-controller.js
// tmux経由でCodexを完全制御するコントローラー

const { spawn } = require('child_process');
const WebSocket = require('ws');

class TmuxCodexController {
  constructor(sessionName = 'codex-8770', port = 8770) {
    this.sessionName = sessionName;
    this.port = port;
    this.hookServerUrl = `ws://localhost:${port}`;
  }

  // tmuxセッションを作成してCodexを起動
  async start() {
    console.log('🚀 Starting Codex in tmux...');
    
    // 既存セッションを削除
    await this.exec('tmux', ['kill-session', '-t', this.sessionName]).catch(() => {});
    
    // 新しいセッションでCodexを起動（対話モード！）
    const cmd = [
      'new-session', '-d', '-s', this.sessionName,
      `export CODEX_REAL_BIN=/home/tomoaki/.volta/bin/codex && ` +
      `export CODEX_HOOK_SERVER=${this.hookServerUrl} && ` +
      `export CODEX_HOOK_BANNER=false && ` +
      `/home/tomoaki/.volta/bin/codex`  // 直接codexを起動（対話モード）
    ];
    
    await this.exec('tmux', cmd);
    console.log(`✅ Codex started in tmux session: ${this.sessionName}`);
    
    // 起動を待つ
    await this.sleep(2000);
  }

  // tmux経由でキーを送信（Enterも送れる！）
  async sendKeys(text, enter = true) {
    console.log(`📤 Sending: "${text}"${enter ? ' + Enter' : ''}`);
    
    const args = ['send-keys', '-t', this.sessionName, text];
    if (enter) {
      args.push('Enter');
    }
    
    await this.exec('tmux', args);
  }

  // 画面内容をキャプチャ
  async capture() {
    const result = await this.exec('tmux', ['capture-pane', '-t', this.sessionName, '-p']);
    return result.stdout;
  }

  // セッションにアタッチ（デバッグ用）
  attach() {
    console.log(`📺 Attaching to ${this.sessionName}...`);
    spawn('tmux', ['attach', '-t', this.sessionName], { stdio: 'inherit' });
  }

  // セッションを終了
  async stop() {
    await this.exec('tmux', ['kill-session', '-t', this.sessionName]);
    console.log('👋 Session stopped');
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
        if (code !== 0) {
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

// 使用例
async function demo() {
  const controller = new TmuxCodexController();
  
  try {
    // Codexを起動
    await controller.start();
    
    // メッセージを送信（自動でEnter！）
    await controller.sendKeys('こんにちは！Nyashプロジェクトから自動挨拶だにゃ🐱');
    await controller.sleep(1000);
    
    // もう一つメッセージ
    await controller.sendKeys('JIT開発の進捗はどう？');
    await controller.sleep(1000);
    
    // 画面内容を確認
    const screen = await controller.capture();
    console.log('\n📺 Current screen:');
    console.log(screen);
    
    // デバッグ用にアタッチもできる
    // controller.attach();
    
  } catch (err) {
    console.error('❌ Error:', err);
  }
}

// モジュールとして使えるようにエクスポート
module.exports = TmuxCodexController;

// 直接実行したらデモを実行
if (require.main === module) {
  demo();
}