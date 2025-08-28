// codex-claude-bridge.js
// Codex と Claude を自動的に橋渡しするシステム
// 安全装置と制御機能付き

const WebSocket = require('ws');
const fs = require('fs').promises;
const path = require('path');

// 設定
const CONFIG = {
  codexWs: 'ws://localhost:8766',
  claudeApiUrl: process.env.CLAUDE_API_URL || 'http://localhost:8080/claude', // 要実装
  bridgePort: 8768,
  
  // 安全設定
  maxBridgesPerHour: 50,
  cooldownMs: 5000,
  idleTimeoutMs: 30000,
  contextWindowSize: 5, // 最後のN個のメッセージを含める
  
  // ログ設定
  logDir: './bridge-logs',
  enableLogging: true
};

// 検出ボックス
class DetectionBox {
  constructor() {
    this.patterns = {
      question: /\?$|どうしますか|どう思いますか|教えて|どうすれば|何が/,
      waiting: /waiting|待機中|入力待ち|▌/i,
      stuck: /エラー|失敗|できません|わかりません|困った/,
      needHelp: /助けて|ヘルプ|相談|アドバイス/,
      planning: /次は|つぎは|計画|予定/
    };
    
    this.lastActivity = Date.now();
    this.quietPeriods = [];
  }
  
  analyze(output) {
    const now = Date.now();
    const idleTime = now - this.lastActivity;
    
    // 複数のパターンをチェックしてスコアリング
    let score = 0;
    let reasons = [];
    
    for (const [type, pattern] of Object.entries(this.patterns)) {
      if (pattern.test(output)) {
        score += 0.3;
        reasons.push(type);
      }
    }
    
    if (idleTime > CONFIG.idleTimeoutMs) {
      score += 0.5;
      reasons.push('idle');
    }
    
    // 最後のアクティビティを更新
    this.lastActivity = now;

    return {
      shouldBridge: score >= 0.5,
      confidence: Math.min(score, 1.0),
      reasons,
      idleTime
    };
  }
}

// フィルターボックス
class FilterBox {
  constructor() {
    this.safetyRules = {
      blocked: [
        /password|secret|token|key|credential/i,
        /rm -rf|delete all|destroy|drop database/i,
        /private|confidential|機密|秘密/i
      ],
      
      requireConfirm: [
        /production|本番|live environment/i,
        /payment|billing|課金|money/i,
        /critical|breaking change|重要な変更/i
      ],
      
      allowed: [
        /実装|implement|設計|design|architecture/,
        /error|bug|fix|修正|デバッグ/,
        /suggest|proposal|提案|アイデア/,
        /explain|説明|なぜ|どうして/
      ]
    };
    
    this.contextPatterns = {
      jit: /JIT|cranelift|compile|lower/i,
      box: /Box|箱|カプセル|Everything is Box/i,
      architecture: /設計|アーキテクチャ|構造|structure/i
    };
  }
  
  filter(content, context = []) {
    // 危険なコンテンツチェック
    for (const pattern of this.safetyRules.blocked) {
      if (pattern.test(content)) {
        return { 
          allow: false, 
          reason: 'blocked-content',
          action: 'reject'
        };
      }
    }
    
    // 確認が必要なコンテンツ
    for (const pattern of this.safetyRules.requireConfirm) {
      if (pattern.test(content)) {
        return { 
          allow: false, 
          reason: 'requires-confirmation',
          action: 'queue'
        };
      }
    }
    
    // コンテキストスコアリング
    let contextScore = 0;
    for (const [type, pattern] of Object.entries(this.contextPatterns)) {
      if (pattern.test(content)) {
        contextScore += 0.3;
      }
    }
    
    // 許可されたパターン
    for (const pattern of this.safetyRules.allowed) {
      if (pattern.test(content)) {
        return { 
          allow: true, 
          confidence: Math.min(0.5 + contextScore, 1.0),
          action: 'forward'
        };
      }
    }
    
    // デフォルトは確認待ち
    return { 
      allow: false, 
      reason: 'no-pattern-match',
      action: 'queue'
    };
  }
}

// ブリッジボックス
class BridgeBox {
  constructor() {
    this.detection = new DetectionBox();
    this.filter = new FilterBox();
    
    this.state = {
      active: false,
      bridgeCount: 0,
      lastBridge: 0,
      queue: [],
      history: []
    };
    
    this.stats = {
      total: 0,
      forwarded: 0,
      blocked: 0,
      queued: 0
    };
  }
  
  async start() {
    console.log('🌉 Starting Codex-Claude Bridge...');
    
    // ログディレクトリ作成
    if (CONFIG.enableLogging) {
      await fs.mkdir(CONFIG.logDir, { recursive: true });
    }
    
    // Codexに接続
    this.connectToCodex();
    
    // 管理用WebSocketサーバー
    this.startControlServer();
    
    this.state.active = true;
    console.log('✅ Bridge is active');
  }
  
  connectToCodex() {
    this.codexWs = new WebSocket(CONFIG.codexWs);
    
    this.codexWs.on('open', () => {
      console.log('📡 Connected to Codex');
    });
    
    this.codexWs.on('message', async (data) => {
      const msg = JSON.parse(data);
      
      if (msg.type === 'codex-event' || msg.type === 'codex-output') {
        await this.handleCodexOutput(msg);
      }
    });
    
    this.codexWs.on('error', (err) => {
      console.error('❌ Codex connection error:', err);
    });
  }
  
  async handleCodexOutput(msg) {
    this.stats.total++;
    
    // 停止検出
    const detection = this.detection.analyze(msg.data);
    
    if (!detection.shouldBridge) {
      return;
    }
    
    // フィルタリング
    const filterResult = this.filter.filter(msg.data, this.state.history);
    
    if (!filterResult.allow) {
      if (filterResult.action === 'queue') {
        this.queueForReview(msg, filterResult);
      } else {
        this.stats.blocked++;
        console.log(`🚫 Blocked: ${filterResult.reason}`);
      }
      return;
    }
    
    // クールダウンチェック
    if (!this.canBridge()) {
      this.queueForReview(msg, { reason: 'cooldown' });
      return;
    }
    
    // ブリッジ実行
    await this.bridge(msg);
  }
  
  canBridge() {
    const now = Date.now();
    
    // クールダウン
    if (now - this.state.lastBridge < CONFIG.cooldownMs) {
      return false;
    }
    
    // レート制限
    const hourAgo = now - 3600000;
    const recentBridges = this.state.history.filter(h => h.timestamp > hourAgo);
    if (recentBridges.length >= CONFIG.maxBridgesPerHour) {
      return false;
    }
    
    return true;
  }
  
  async bridge(msg) {
    console.log('🌉 Bridging to Claude...');
    
    try {
      // コンテキスト構築
      const context = this.buildContext(msg);
      
      // Claude API呼び出し（要実装）
      const claudeResponse = await this.callClaudeAPI(context);
      
      // Codexに返信
      this.sendToCodex(claudeResponse);
      
      // 記録
      this.recordBridge(msg, claudeResponse);
      
      this.stats.forwarded++;
      this.state.lastBridge = Date.now();
      
    } catch (err) {
      console.error('❌ Bridge error:', err);
    }
  }
  
  buildContext(currentMsg) {
    // 最近の履歴を含める
    const recentHistory = this.state.history.slice(-CONFIG.contextWindowSize);
    
    return {
      current: currentMsg.data,
      history: recentHistory.map(h => ({
        from: h.from,
        content: h.content,
        timestamp: h.timestamp
      })),
      context: {
        project: 'Nyash JIT Development',
        focus: 'Phase 10.7 - JIT Branch Wiring',
        recentTopics: this.extractTopics(recentHistory)
      }
    };
  }
  
  async callClaudeAPI(context) {
    // TODO: 実際のClaude API実装
    // ここはプレースホルダー
    return {
      response: "Claude's response would go here",
      confidence: 0.9
    };
  }
  
  sendToCodex(response) {
    this.codexWs.send(JSON.stringify({
      op: 'send',
      data: response.response
    }));
  }
  
  queueForReview(msg, reason) {
    this.state.queue.push({
      message: msg,
      reason,
      timestamp: Date.now()
    });
    
    this.stats.queued++;
    console.log(`📋 Queued for review: ${reason.reason}`);
  }
  
  recordBridge(input, output) {
    const record = {
      timestamp: Date.now(),
      from: 'codex',
      to: 'claude',
      input: input.data,
      output: output.response,
      confidence: output.confidence
    };
    
    this.state.history.push(record);
    this.state.bridgeCount++;
    
    // ログ保存
    if (CONFIG.enableLogging) {
      this.saveLog(record);
    }
  }
  
  async saveLog(record) {
    const filename = `bridge-${new Date().toISOString().split('T')[0]}.jsonl`;
    const filepath = path.join(CONFIG.logDir, filename);
    
    await fs.appendFile(
      filepath,
      JSON.stringify(record) + '\n'
    );
  }
  
  extractTopics(history) {
    // 最近の話題を抽出
    const topics = new Set();
    
    history.forEach(h => {
      if (/JIT|cranelift/i.test(h.content)) topics.add('JIT');
      if (/box|箱/i.test(h.content)) topics.add('Box Philosophy');
      if (/PHI|branch/i.test(h.content)) topics.add('Control Flow');
    });
    
    return Array.from(topics);
  }
  
  startControlServer() {
    // 管理用WebSocketサーバー
    const wss = new WebSocket.Server({ port: CONFIG.bridgePort });
    
    wss.on('connection', (ws) => {
      ws.on('message', (data) => {
        const cmd = JSON.parse(data);
        
        switch (cmd.op) {
          case 'status':
            ws.send(JSON.stringify({
              type: 'status',
              state: this.state,
              stats: this.stats
            }));
            break;
            
          case 'queue':
            ws.send(JSON.stringify({
              type: 'queue',
              items: this.state.queue
            }));
            break;
            
          case 'approve':
            // キューから承認して転送
            if (cmd.id && this.state.queue[cmd.id]) {
              const item = this.state.queue[cmd.id];
              this.bridge(item.message);
              this.state.queue.splice(cmd.id, 1);
            }
            break;
            
          case 'toggle':
            this.state.active = !this.state.active;
            ws.send(JSON.stringify({
              type: 'toggled',
              active: this.state.active
            }));
            break;
        }
      });
    });
    
    console.log(`🎮 Control server on ws://localhost:${CONFIG.bridgePort}`);
  }
}

// メイン
if (require.main === module) {
  const bridge = new BridgeBox();
  bridge.start();
  
  // グレースフルシャットダウン
  process.on('SIGINT', () => {
    console.log('\n👋 Shutting down bridge...');
    process.exit(0);
  });
}

module.exports = { BridgeBox, DetectionBox, FilterBox };
