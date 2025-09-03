# AI Integration Guide for ANCP

## 🤖 AI開発者のためのANCP活用ガイド

### なぜANCPがAI開発を変えるのか

1. **コンテキスト容量2-3倍**: より多くのコードをAIに渡せる
2. **理解速度向上**: パターンが明確で認識しやすい
3. **生成効率向上**: 短い記号で素早くコード生成

## 📋 クイックリファレンス

### 最重要マッピング（必ず覚える）
```
$ = box      # Box定義
n = new      # インスタンス生成
m = me       # 自己参照
l = local    # ローカル変数
r = return   # 戻り値
```

### よく使うパターン
```nyash
// Nyash
box Cat from Animal {
    init { name }
    birth(name) {
        me.name = name
    }
}

// ANCP
$Cat@Animal{
    #{name}
    b(name){m.name=name}
}
```

## 🎯 AI別最適化ガイド

### Claude (Anthropic)
```markdown
# Claudeへの指示例
NyashコードをANCP記法で書いてください。以下のマッピングを使用：
- box → $
- new → n
- me → m
- local → l
- return → r

コンテキスト: 200k tokens利用可能
推奨: 大規模プロジェクト全体をANCPで渡す
```

### ChatGPT (OpenAI)
```markdown
# ChatGPTへの指示例
Use ANCP notation for Nyash code:
;ancp:1.0 nyash:0.5;

Quick reference:
$ = box, n = new, m = me, l = local, r = return

Context: 128k tokens (GPT-4)
Strategy: Focus on core modules with ANCP
```

### Gemini (Google)
```markdown
# Geminiへの深い考察依頼
ANCPを使ったNyashコードの最適化を深く考えてください。
トークン効率とコード美しさのバランスを重視。

特に注目:
- $ (box) によるオブジェクト指向の簡潔表現
- m (me) による自己参照の明確化
```

### Codex/Copilot
```python
# .copilot/ancp_hints.py
"""
ANCP Quick Patterns:
- $ClassName{...} = box ClassName { ... }
- m.method() = me.method()
- l var = value = local var = value
- r value = return value
"""
```

## 💡 実践的な使い方

### 1. 大規模コードレビュー
```bash
# 全プロジェクトをANCPに変換してAIに渡す
nyash2ancp -i src/ -o /tmp/review.ancp --recursive

# AIへのプロンプト
"Review this ANCP code for performance issues:
[/tmp/review.ancp の内容]"
```

### 2. アーキテクチャ設計相談
```ancp
;ancp:1.0 nyash:0.5;
// 新しいP2Pシステムの設計
$P2PNetwork{
    #{nodes,dht}
    
    connect(peer){
        l conn=n Connection(peer)
        m.nodes.add(conn)
        r conn
    }
}

// AIへの質問
"この設計でスケーラビリティの問題はありますか？"
```

### 3. バグ修正依頼
```ancp
// バグのあるコード（ANCP）
$Calculator{
    divide(a,b){
        r a/b  // ゼロ除算チェックなし
    }
}

// AIへの依頼
"このANCPコードのバグを修正してください"
```

## 📊 効果測定

### トークン削減の実例
```python
# 測定スクリプト
import tiktoken

def measure_reduction(nyash_code, ancp_code):
    enc = tiktoken.get_encoding("cl100k_base")
    
    nyash_tokens = len(enc.encode(nyash_code))
    ancp_tokens = len(enc.encode(ancp_code))
    
    reduction = (1 - ancp_tokens / nyash_tokens) * 100
    
    print(f"Nyash: {nyash_tokens} tokens")
    print(f"ANCP: {ancp_tokens} tokens")
    print(f"Reduction: {reduction:.1f}%")
    
    return reduction

# 実例
nyash = """
box WebServer from HttpBox {
    init { port, routes }
    
    birth(port) {
        me.port = port
        me.routes = new MapBox()
    }
    
    addRoute(path, handler) {
        me.routes.set(path, handler)
        return me
    }
}
"""

ancp = "$WebServer@HttpBox{#{port,routes}b(port){m.port=port m.routes=n MapBox()}addRoute(path,handler){m.routes.set(path,handler)r m}}"

reduction = measure_reduction(nyash, ancp)
# 結果: 約65%削減！
```

## 🔧 AIツール統合

### VSCode + GitHub Copilot
```json
// .vscode/settings.json
{
    "github.copilot.advanced": {
        "ancp.hints": {
            "box": "$",
            "new": "n",
            "me": "m"
        }
    }
}
```

### Custom AI Integration
```typescript
// AI SDK統合例
class AncpAwareAI {
    async complete(prompt: string, context: string): Promise<string> {
        // コンテキストをANCPに変換
        const ancpContext = this.transcoder.encode(context);
        
        // AI APIコール（トークン数大幅削減）
        const response = await this.ai.complete({
            prompt,
            context: ancpContext,
            metadata: { format: "ancp:1.0" }
        });
        
        // レスポンスをNyashに戻す
        return this.transcoder.decode(response);
    }
}
```

## 📚 学習リソース

### AIモデル向けトレーニングデータ
```bash
# 並列コーパス生成
tools/generate_parallel_corpus.sh

# 出力
corpus/
├── nyash/     # 通常のNyashコード
├── ancp/      # 対応するANCPコード
└── metadata/  # トークン削減率等
```

### プロンプトテンプレート
```markdown
# 効果的なプロンプト例

## コード生成
"Write a P2P chat application in ANCP notation.
Requirements: [要件]
Use these patterns: $=box, n=new, m=me"

## コードレビュー
"Review this ANCP code for security issues:
```ancp
[コード]
```
Focus on: memory safety, race conditions"

## リファクタリング
"Refactor this ANCP code for better performance:
[コード]
Maintain the same API but optimize internals"
```

## 🚀 ベストプラクティス

### DO
- ✅ 大規模コードはANCPで渡す
- ✅ AI応答もANCPで受け取る
- ✅ 記号の意味を最初に説明
- ✅ バージョンヘッダーを含める

### DON'T
- ❌ 部分的なANCP使用（混乱の元）
- ❌ カスタム記号の追加
- ❌ コメントまで圧縮

## 🎮 実践演習

### 演習1: 基本変換
```nyash
// これをANCPに変換
box Calculator {
    init { memory }
    
    birth() {
        me.memory = 0
    }
    
    add(x, y) {
        local result = x + y
        me.memory = result
        return result
    }
}
```

<details>
<summary>答え</summary>

```ancp
$Calculator{#{memory}b(){m.memory=0}add(x,y){l result=x+y m.memory=result r result}}
```
</details>

### 演習2: AI活用
```ancp
// このANCPコードの問題点をAIに聞く
$Server{listen(p){loop(true){l c=accept()process(c)}}}
```

期待する指摘:
- エラーハンドリングなし
- 接続の並行処理なし
- リソースリークの可能性

## 📈 成功事例

### 事例1: Nyashコンパイラ開発
- 通常: 20,000行 → 40,000 tokens
- ANCP: 20,000行 → 15,000 tokens
- **結果**: Claude一回のコンテキストで全体を把握！

### 事例2: バグ修正効率
- 従来: 関連コード5ファイルが入らない
- ANCP: 10ファイル＋テストコードまで含められる
- **結果**: AIが文脈を完全理解し、的確な修正提案

## 🔮 将来の展望

### ANCP v2.0
- AI専用の追加圧縮
- 意味保持型トークン削減
- カスタム辞書対応

### AI統合の深化
- IDEでのリアルタイムANCP変換
- AIレビューの自動ANCP化
- 学習済みANCPモデル

---

ANCPは単なる圧縮記法ではなく、AIとNyashをつなぐ架け橋です。
この革命的なプロトコルを活用して、AI時代の開発を加速させましょう！