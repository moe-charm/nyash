const TmuxCodexController = require('./tmux-codex-controller');

async function sendMessage(message) {
  const controller = new TmuxCodexController();
  
  try {
    // メッセージを送信（Enterも自動！）
    await controller.sendKeys(message);
    console.log(`✅ Sent: "${message}"`);
    
    // 少し待って画面を確認
    await controller.sleep(2000);
    const screen = await controller.capture();
    console.log('\n📺 Current screen (last 10 lines):');
    console.log(screen.split('\n').slice(-10).join('\n'));
    
  } catch (err) {
    console.error('❌ Error:', err);
  }
}

// コマンドライン引数からメッセージを取得
const message = process.argv.slice(2).join(' ') || 'Hello from Nyash!';
sendMessage(message);