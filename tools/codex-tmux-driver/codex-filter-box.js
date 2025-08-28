// codex-filter-box.js
// Codexの出力をフィルタリング・分類する箱
// ChatGPT5さんへの転送判断なども行う

class CodexFilterBox {
  constructor() {
    // フィルタルール設定
    this.rules = {
      // 緊急度の判定
      urgent: {
        patterns: [
          /緊急|urgent|critical|重大/i,
          /バグ発見|bug found|error detected/i,
          /セキュリティ|security issue/i
        ],
        action: 'notify-immediately',
        priority: 'high',
        forward: true
      },
      
      // 実装完了の通知
      implementation: {
        patterns: [
          /実装完了|implementation complete/i,
          /機能追加|feature added/i,
          /修正完了|fixed|resolved/i
        ],
        action: 'forward-to-chatgpt5',
        priority: 'medium',
        forward: true
      },
      
      // 提案・相談
      proposal: {
        patterns: [
          /提案|suggestion|proposal/i,
          /どうでしょう|how about/i,
          /検討|consider/i
        ],
        action: 'queue-for-review',
        priority: 'low',
        forward: false,
        queue: true
      },
      
      // 思考中・処理中
      thinking: {
        patterns: [
          /考え中|thinking|processing/i,
          /分析中|analyzing/i,
          /調査中|investigating/i
        ],
        action: 'log-only',
        priority: 'info',
        forward: false
      },
      
      // 雑談・無視可能
      ignore: {
        patterns: [
          /雑談|small talk/i,
          /ところで|by the way/i,
          /関係ない|unrelated/i
        ],
        action: 'archive',
        priority: 'ignore',
        forward: false
      }
    };
    
    // 統計情報
    this.stats = {
      total: 0,
      filtered: {},
      forwarded: 0,
      queued: 0
    };
    
    // キュー（後で確認用）
    this.queue = [];
  }
  
  // メインのフィルタ処理
  filter(codexOutput) {
    this.stats.total++;
    
    // 各ルールをチェック
    for (const [category, rule] of Object.entries(this.rules)) {
      if (this.matchesRule(codexOutput, rule)) {
        this.stats.filtered[category] = (this.stats.filtered[category] || 0) + 1;
        
        const result = {
          category,
          action: rule.action,
          priority: rule.priority,
          forward: rule.forward,
          timestamp: new Date().toISOString(),
          original: codexOutput
        };
        
        // キューに追加
        if (rule.queue) {
          this.queue.push(result);
          this.stats.queued++;
        }
        
        // 転送フラグ
        if (rule.forward) {
          this.stats.forwarded++;
        }
        
        return result;
      }
    }
    
    // どのルールにも一致しない場合
    return {
      category: 'default',
      action: 'log',
      priority: 'normal',
      forward: false,
      timestamp: new Date().toISOString(),
      original: codexOutput
    };
  }
  
  // ルールマッチング
  matchesRule(text, rule) {
    return rule.patterns.some(pattern => pattern.test(text));
  }
  
  // キューから項目取得
  getQueue(count = 10) {
    return this.queue.slice(-count);
  }
  
  // キューをクリア
  clearQueue() {
    const cleared = this.queue.length;
    this.queue = [];
    return cleared;
  }
  
  // 統計情報取得
  getStats() {
    return {
      ...this.stats,
      queueLength: this.queue.length,
      categories: Object.keys(this.rules)
    };
  }
  
  // カスタムルール追加
  addRule(name, config) {
    this.rules[name] = {
      patterns: config.patterns.map(p => 
        typeof p === 'string' ? new RegExp(p, 'i') : p
      ),
      action: config.action || 'log',
      priority: config.priority || 'normal',
      forward: config.forward || false,
      queue: config.queue || false
    };
  }
  
  // バッチ処理
  filterBatch(outputs) {
    return outputs.map(output => this.filter(output));
  }
}

// エクスポート
module.exports = CodexFilterBox;

// 使用例
if (require.main === module) {
  const filter = new CodexFilterBox();
  
  // テストデータ
  const testOutputs = [
    'Codex: バグ発見！メモリリークが発生しています',
    '考え中... JITの最適化方法を検討しています',
    'ところで、今日の天気はどうですか？',
    '実装完了: Phase 10.7のPHI実装が完成しました',
    '提案: 箱作戦をさらに拡張してはどうでしょう？'
  ];
  
  console.log('=== Codex Filter Box Test ===\n');
  
  testOutputs.forEach(output => {
    const result = filter.filter(output);
    console.log(`Input: "${output}"`);
    console.log(`Result: ${result.category} - ${result.action} (${result.priority})`);
    console.log(`Forward to ChatGPT5: ${result.forward ? 'YES' : 'NO'}`);
    console.log('---');
  });
  
  console.log('\nStats:', filter.getStats());
}