#!/usr/bin/env node
// tmux経由でメッセージを注入するヘルパー

const { exec } = require('child_process');

// コマンドライン引数から対象セッションとメッセージを取得
const args = process.argv.slice(2);
if (args.length < 2) {
  console.error('Usage: node tmux-inject-helper.js <session-name> <message>');
  process.exit(1);
}

const [sessionName, message] = args;

// tmux send-keysコマンドを実行
// C-mはEnterキー
const command = `tmux send-keys -t ${sessionName} '${message.replace(/'/g, "'\\''")}'`;
console.log(`Executing: ${command}`);

exec(command, (error, stdout, stderr) => {
  if (error) {
    console.error(`Error: ${error.message}`);
    process.exit(1);
  }
  if (stderr) {
    console.error(`stderr: ${stderr}`);
  }
  console.log('Message sent successfully!');
  
  // Enterキーを送信
  exec(`tmux send-keys -t ${sessionName} C-m`, (err) => {
    if (!err) {
      console.log('Enter key sent!');
    }
  });
});