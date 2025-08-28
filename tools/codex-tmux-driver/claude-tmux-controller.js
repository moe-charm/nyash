// claude-tmux-controller.js
// Claudeもtmuxで制御！完璧な双方向通信！

const { spawn } = require('child_process');

class ClaudeTmuxController {
  constructor(sessionName = 'claude-8771') {
    this.sessionName = sessionName;
  }

  // tmuxセッションでClaudeを起動
  async start() {
    console.log('🤖 Starting Claude in tmux...');
    
    // 既存セッションを削除
    await this.exec('tmux', ['kill-session', '-t', this.sessionName]).catch(() => {});
    
    // 新しいセッションでclaude cliを起動
    const cmd = [
      'new-session', '-d', '-s', this.sessionName,
      'claude'  // claude CLI（仮定）
    ];
    
    await this.exec('tmux', cmd);
    console.log(`✅ Claude started in tmux session: ${this.sessionName}`);
    
    await this.sleep(2000);
  }

  // tmux経由でテキストとEnterを送信！
  async sendMessage(text) {
    console.log(`📤 Sending to Claude: "${text}"`);
    await this.exec('tmux', ['send-keys', '-t', this.sessionName, text, 'Enter']);
  }

  // 画面をキャプチャ
  async capture() {
    const result = await this.exec('tmux', ['capture-pane', '-t', this.sessionName, '-p']);
    return result.stdout;
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

module.exports = ClaudeTmuxController;