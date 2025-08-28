// common-message-file.js
// 共通テキストファイル経由の通信システム

const fs = require('fs');
const path = require('path');
const EventEmitter = require('events');

class FileBasedMessenger extends EventEmitter {
  constructor(config = {}) {
    super();
    this.config = {
      messageFile: config.messageFile || './shared-messages.txt',
      lockFile: config.lockFile || './shared-messages.lock',
      pollInterval: config.pollInterval || 500,
      ...config
    };
    
    this.lastReadPosition = 0;
    this.isWatching = false;
  }

  // メッセージを書き込む
  async sendMessage(from, to, message) {
    const timestamp = new Date().toISOString();
    const entry = JSON.stringify({
      timestamp,
      from,
      to,
      message
    }) + '\n';
    
    // ロックを取得
    await this.acquireLock();
    
    try {
      // ファイルに追記
      fs.appendFileSync(this.config.messageFile, entry);
      console.log(`📤 Sent: ${from} → ${to}: ${message}`);
    } finally {
      // ロック解放
      this.releaseLock();
    }
  }

  // メッセージを監視
  startWatching(myName) {
    this.isWatching = true;
    console.log(`👁️ Watching messages for: ${myName}`);
    
    // 初期位置を設定
    if (fs.existsSync(this.config.messageFile)) {
      const stats = fs.statSync(this.config.messageFile);
      this.lastReadPosition = stats.size;
    }
    
    // 定期的にチェック
    const checkMessages = () => {
      if (!this.isWatching) return;
      
      try {
        if (!fs.existsSync(this.config.messageFile)) {
          setTimeout(checkMessages, this.config.pollInterval);
          return;
        }
        
        const stats = fs.statSync(this.config.messageFile);
        if (stats.size > this.lastReadPosition) {
          // 新しいメッセージがある
          const buffer = Buffer.alloc(stats.size - this.lastReadPosition);
          const fd = fs.openSync(this.config.messageFile, 'r');
          fs.readSync(fd, buffer, 0, buffer.length, this.lastReadPosition);
          fs.closeSync(fd);
          
          const newLines = buffer.toString().trim().split('\n');
          
          for (const line of newLines) {
            if (line) {
              try {
                const msg = JSON.parse(line);
                // 自分宛のメッセージ
                if (msg.to === myName || msg.to === '*') {
                  this.emit('message', msg);
                  console.log(`📨 Received: ${msg.from} → ${msg.to}: ${msg.message}`);
                }
              } catch (e) {
                console.error('Parse error:', e);
              }
            }
          }
          
          this.lastReadPosition = stats.size;
        }
      } catch (err) {
        console.error('Watch error:', err);
      }
      
      setTimeout(checkMessages, this.config.pollInterval);
    };
    
    checkMessages();
  }

  // 監視停止
  stopWatching() {
    this.isWatching = false;
    console.log('🛑 Stopped watching');
  }

  // 簡易ロック機構
  async acquireLock(maxWait = 5000) {
    const startTime = Date.now();
    
    while (fs.existsSync(this.config.lockFile)) {
      if (Date.now() - startTime > maxWait) {
        throw new Error('Lock timeout');
      }
      await this.sleep(50);
    }
    
    fs.writeFileSync(this.config.lockFile, process.pid.toString());
  }

  releaseLock() {
    if (fs.existsSync(this.config.lockFile)) {
      fs.unlinkSync(this.config.lockFile);
    }
  }

  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // メッセージ履歴をクリア
  clearMessages() {
    if (fs.existsSync(this.config.messageFile)) {
      fs.unlinkSync(this.config.messageFile);
    }
    console.log('🗑️ Message history cleared');
  }
}

// CLIとして使用
if (require.main === module) {
  const messenger = new FileBasedMessenger();
  const myName = process.argv[2];
  const command = process.argv[3];
  
  if (!myName || !command) {
    console.log(`
使い方:
  node common-message-file.js <名前> watch              # メッセージを監視
  node common-message-file.js <名前> send <宛先> <内容>  # メッセージ送信
  node common-message-file.js <名前> clear              # 履歴クリア
    `);
    process.exit(1);
  }
  
  switch (command) {
    case 'watch':
      messenger.on('message', (msg) => {
        // 自動返信（デモ用）
        if (msg.message.includes('?')) {
          setTimeout(() => {
            messenger.sendMessage(myName, msg.from, 'はい、了解しました！');
          }, 1000);
        }
      });
      messenger.startWatching(myName);
      console.log('Press Ctrl+C to stop...');
      break;
      
    case 'send':
      const to = process.argv[4];
      const message = process.argv.slice(5).join(' ');
      messenger.sendMessage(myName, to, message);
      break;
      
    case 'clear':
      messenger.clearMessages();
      break;
  }
}

module.exports = FileBasedMessenger;