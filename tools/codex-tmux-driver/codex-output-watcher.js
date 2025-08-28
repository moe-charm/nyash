// codex-output-watcher.js
// Codexの出力を監視してClaudeに転送するウォッチャー

const { spawn } = require('child_process');
const EventEmitter = require('events');

class CodexOutputWatcher extends EventEmitter {
  constructor(sessionName = 'codex-safe') {
    super();
    this.sessionName = sessionName;
    this.lastOutput = '';
    this.isWorking = false;
    this.watchInterval = null;
  }

  // 監視開始
  start(intervalMs = 1000) {
    console.log(`👁️ Starting to watch Codex output in ${this.sessionName}...`);
    
    this.watchInterval = setInterval(() => {
      this.checkOutput();
    }, intervalMs);
  }

  // 監視停止
  stop() {
    if (this.watchInterval) {
      clearInterval(this.watchInterval);
      this.watchInterval = null;
      console.log('👁️ Stopped watching');
    }
  }

  // 画面をキャプチャして状態を確認
  async checkOutput() {
    try {
      const output = await this.capturePane();
      
      // 状態を解析
      const wasWorking = this.isWorking;
      this.isWorking = this.detectWorking(output);
      
      // Working → 完了に変化した場合
      if (wasWorking && !this.isWorking) {
        console.log('✅ Codex finished working!');
        const response = this.extractCodexResponse(output);
        if (response) {
          this.emit('response', response);
        }
      }
      
      // プロンプトが表示されている = 入力待ち
      if (this.detectPrompt(output) && !this.isWorking) {
        this.emit('ready');
      }
      
      this.lastOutput = output;
    } catch (err) {
      console.error('❌ Watch error:', err);
    }
  }

  // tmuxペインをキャプチャ
  capturePane() {
    return new Promise((resolve, reject) => {
      const proc = spawn('tmux', ['capture-pane', '-t', this.sessionName, '-p']);
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

  // "Working" 状態を検出
  detectWorking(output) {
    return output.includes('Working (') || output.includes('⏳');
  }

  // プロンプト（入力待ち）を検出
  detectPrompt(output) {
    return output.includes('▌') && output.includes('⏎ send');
  }

  // Codexの応答を抽出
  extractCodexResponse(output) {
    const lines = output.split('\n');
    let inCodexResponse = false;
    let response = [];
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      // "codex" ラベルを見つけたら応答開始
      if (line.trim() === 'codex') {
        inCodexResponse = true;
        continue;
      }
      
      // 次のプロンプトや"user"が来たら終了
      if (inCodexResponse && (line.includes('▌') || line.trim() === 'user')) {
        break;
      }
      
      // 応答を収集
      if (inCodexResponse && line.trim()) {
        // Working行やメタ情報を除外
        if (!line.includes('Working') && !line.includes('⏎ send')) {
          response.push(line);
        }
      }
    }
    
    return response.join('\n').trim();
  }
}

// 使用例とテスト
if (require.main === module) {
  const watcher = new CodexOutputWatcher();
  
  watcher.on('response', (response) => {
    console.log('\n📝 Codex Response:');
    console.log('-------------------');
    console.log(response);
    console.log('-------------------\n');
    
    // ここでClaudeに転送する処理を追加
    console.log('🚀 TODO: Send this to Claude!');
  });
  
  watcher.on('ready', () => {
    console.log('💚 Codex is ready for input');
  });
  
  watcher.start(500); // 500msごとにチェック
  
  // 30秒後に停止
  setTimeout(() => {
    watcher.stop();
    process.exit(0);
  }, 30000);
}

module.exports = CodexOutputWatcher;