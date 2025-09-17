# ANCP実装に向けた実用的洞察まとめ

Date: 2025-09-03

## 🎯 すぐに実装すべき優先事項

### 1. 正規化ルールの明文化（Codex提案）
```rust
// P* (正規形) の定義が最重要
pub struct Canonicalizer {
    // コメント処理: 保持 or 削除？
    // 空白処理: 正規化ルール
    // エイリアス解決: import as の扱い
}
```
**理由**: 可逆性の数学的保証に必須

### 2. トークン最適化戦略（Codex提案）
```
// GPTトークナイザーに合わせた記号選択
F層記号候補:
- 高頻度: $ @ # ^ ~ 
- 避ける: 長いUnicode、稀な記号
```
**理由**: 圧縮率はバイト数でなくトークン数で測定すべき

### 3. IDE統合の最小実装（Gemini提案）
```typescript
// VS Code拡張: F層ホバーでP層表示
onHover(position) {
    const fToken = getFusionToken(position)
    const pCode = sourceMap.lookup(fToken)
    return new Hover(pCode)
}
```
**理由**: デバッグ体験が開発普及の鍵

## 📊 実装順序の推奨

### Phase 1: ミニマルPoC（1週間）
1. **AST正規化器**
   - Canonicalizer実装
   - P→P*変換の決定的動作
   
2. **基本変換器**
   - Box定義の圧縮
   - 関数定義の圧縮
   - MIRハッシュ検証

3. **双方向マップ**
   - 最小限のソースマップ
   - ラウンドトリップテスト

### Phase 2: 実用化（2週間目）
1. **CLI実装**（Codex提案）
   ```bash
   ancp encode --layer F input.nyash -o output.f
   ancp decode output.f --map output.map
   ancp verify input.nyash output.f  # MIR等価性チェック
   ```

2. **プロジェクト辞書**（Gemini提案）
   ```yaml
   # .ancprc
   symbols:
     WebServer: WS
     HttpRequest: HR
     handleRequest: hR
   ```

3. **エラー位置変換**
   - F層エラー→P層位置
   - スタックトレース変換

### Phase 3: 高度な最適化（3週間目）
1. **文法圧縮**（Codex提案）
   - Re-Pair/Sequiturアルゴリズム
   - 頻出パターン辞書化

2. **混合モード**（Gemini提案）
   ```nyash
   // 通常のP層コード
   box NormalClass { ... }
   
   fusion {
       // F層圧縮コード
       $FC@B{...}
   }
   ```

3. **意味論的圧縮**（Gemini提案）
   - パターン認識
   - 高レベル抽象化

## 🔍 検証計画（両者統合）

### 自動テストスイート
```rust
#[test]
fn roundtrip_property_test() {
    // Codex提案: Property-based testing
    proptest!(|(ast: RandomAST)| {
        let encoded = ancp.encode(ast, Level::F);
        let decoded = ancp.decode(encoded);
        assert_eq!(normalize(ast), decoded);
        assert_eq!(mir_hash(ast), mir_hash(decoded));
    });
}
```

### ベンチマーク項目
| 指標 | 測定内容 | 目標値 |
|------|----------|--------|
| 圧縮率 | トークン削減率 | 90% |
| 変換速度 | ms/1000行 | <100ms |
| マップサイズ | % of P | <5% |
| MIR一致率 | Pass/Fail | 100% |

## 💡 回避すべき落とし穴

### 1. 文字列リテラルの罠（Codex警告）
```nyash
// 問題: 文字列内のF層記号
local msg = "User sent $100"  // $ が誤解釈される
```
**対策**: エスケープメカニズム必須

### 2. デバッグ地獄（Codex警告）
```
Error at $WS@H{p;r=@M|b(p){$.p=p}:12:5
```
**対策**: デコーダー常駐でP層位置を即座に表示

### 3. プラグイン非互換（Codex警告）
```nyash
// プラグインが新構文追加
plugin syntax { ... }  // F層エンコーダーが対応できない
```
**対策**: プラグイン登録API必須

## 🚀 即座に始められること

1. **仕様書ドラフト作成**
   - P*正規化ルール
   - C/F層文法定義
   - ソースマップフォーマット

2. **最小実装**
   ```bash
   # まずBoxだけで動作確認
   echo "box Test { }" | ancp encode -l F
   # => $T{}
   ```

3. **コーパス収集**
   - 既存Nyashコード収集
   - 頻度解析でF層記号決定

## 📈 成功指標

### 短期（1ヶ月）
- [ ] 10個のサンプルで90%圧縮達成
- [ ] MIR等価性100%保証
- [ ] 基本的なCLI動作

### 中期（3ヶ月）
- [ ] Nyashコンパイラ自身を圧縮
- [ ] VS Code拡張リリース
- [ ] 論文ドラフト完成

### 長期（6ヶ月）
- [ ] 他言語への応用
- [ ] 標準規格提案
- [ ] AI開発ツール統合

---

**次の一歩**: AST正規化ルール（P*）の仕様を1ページで書く！