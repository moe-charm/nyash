# AI Agents Challenge - ワークフロー設計

## 基本的な流れ

### 1. **Nyash側** - トリガー送信
```nyash
box PriceMonitorAgent {
    init { webhookUrl, products }
    
    checkPrices() {
        local net = new NetBox()
        local data = new MapBox()
        data.set("action", "check_prices")
        data.set("products", me.products)
        
        // n8nのWebhookをトリガー
        net.post(me.webhookUrl, data.toJsonBox())
    }
}
```

### 2. **n8n側** - ワークフロー
```
[Webhook] → [AI Agent] → [Bright Data] → [Process] → [Response]
```

- **Webhook Node**: Nyashからのリクエストを受信
- **AI Agent Node**: 
  - どのサイトをスクレイピングするか決定
  - Bright Dataへのクエリを構築
- **Bright Data Node**: 
  - 実際のWebスクレイピング実行
  - 商品価格などのデータ取得
- **Process**: データ整形・比較
- **Response**: 結果をWebhookで返す

### 3. **具体例: 価格監視エージェント**

#### Nyash側の実装
```nyash
static box Main {
    main() {
        local agent = new PriceMonitorAgent()
        agent.webhookUrl = "https://your-n8n-instance.n8n.cloud/webhook/price-monitor"
        
        // 監視したい商品
        local products = new ArrayBox()
        products.push({
            "name": "iPhone 15",
            "url": "https://example.com/iphone15",
            "targetPrice": 800
        })
        
        agent.products = products
        agent.checkPrices()
    }
}
```

#### n8nでの設定手順

1. **Webhook Node設定**
   - Method: POST
   - Path: /price-monitor
   - Response Mode: Last Node

2. **AI Agent Node設定**
   - Model: GPT-3.5/4
   - Prompt: 
     ```
     商品リストから、Bright Dataでスクレイピングすべき
     URLとセレクタを生成してください。
     商品: {{$json.products}}
     ```

3. **Bright Data Node設定**
   - Scraper API使用
   - Dynamic URL from AI Agent
   - Extract: 価格情報

4. **Code Node（価格比較）**
   ```javascript
   const currentPrice = $node["Bright Data"].json.price;
   const targetPrice = $node["Webhook"].json.products[0].targetPrice;
   
   if (currentPrice < targetPrice) {
     return {
       alert: true,
       message: `価格が下がりました！${currentPrice}円`
     };
   }
   ```

### チャレンジの要件チェック
- ✅ n8n AI Agent Node使用
- ✅ Bright Data Verified Node使用
- ✅ 実用的で複雑
- ✅ 創造的（Nyash言語使用）

### デモ動画に含めるべき内容
1. Nyashコードの実行
2. n8nワークフローの動作
3. Bright Dataでのデータ取得
4. 結果の表示

### 簡単に始めるには

まず超シンプルな例から：
1. Webhookを受け取る
2. AI Agentに「今日の天気は？」と聞く
3. 結果を返す

これが動いたら、Bright Dataを追加していく！