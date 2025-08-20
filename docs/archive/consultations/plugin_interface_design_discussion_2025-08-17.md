# プラグインインターフェース設計討論 (2025-08-17)

## 🎯 議題：BID-FFIプラグインシステムの設計

### 背景
- ビルトインFileBoxが既に存在
- BID-FFIプラグイン版FileBoxで置き換えたい
- ビルド時間短縮とプラグインアーキテクチャの実証が目的

### 主な論点
1. ビルトインBox → プラグインBoxの透過的な置き換え
2. `FileBox.open()` のような静的メソッドの結びつけ方
3. プラグインインターフェース定義の外部化方式

## 💡 検討した案

### 案1: 汎用ラッパー
```nyash
// 呼び出しが汚い
local file = Plugin.call("FileBox", "open", ["test.txt", "r"])
```

### 案2: 専用ラッパー
```nyash
// きれい！でも各Boxごとに手書きが必要
local file = FileBox.open("test.txt")
```

### 案3: BoxDispatcher（透過的ディスパッチ）
```rust
pub enum BoxImpl {
    Builtin(Box<dyn NyashBox>),     // ビルトイン実装
    Plugin(BidHandle, PluginRef),    // プラグイン実装
}
```

### 案4: Unified Box Factory
```rust
pub struct BoxFactory {
    providers: HashMap<String, Box<dyn BoxProvider>>,
}
```

## 🎉 最終解：YAML/JSON + 署名DSL

### Codex先生の推奨設計
```yaml
# filebox.plugin.yaml
schema: 1
plugin:
  name: filebox
  version: 1
  
apis:
  # 静的メソッド（::）
  - sig: "FileBox::open(path: string, mode?: string) -> FileBox"
    doc: "Open a file"
    
  # インスタンスメソッド（#）
  - sig: "FileBox#read(size?: int) -> string"
    doc: "Read file content"
```

### 利点
1. **記号で静的/インスタンスを区別**
   - `::` = 静的メソッド（C++風）
   - `#` = インスタンスメソッド（Ruby風）

2. **フラット構造**
   - `apis` 配列にすべて並べる
   - 階層が深くならない

3. **署名DSL**
   - 型情報を1行で表現
   - パーサーも簡単

4. **YAML → JSON変換**
   - 開発時：YAML（人間に優しい）
   - 実行時：JSON（マシンに優しい）

## 🤔 Gemini先生への質問事項

1. **透過的な置き換え**
   - 既存のNyashコードを一切変更せずに、ビルトインBoxをプラグインBoxに置き換える最良の方法は？
   - パフォーマンスインパクトをどう最小化するか？

2. **署名DSLの設計**
   - `Type::method()` vs `Type.method()` の選択理由
   - オーバーロードの表現方法
   - ジェネリクスの将来的な拡張性

3. **実装戦略**
   - インタープリター実行時のディスパッチ最適化
   - プラグインの遅延ロード実装
   - エラーハンドリングのベストプラクティス

4. **Everything is Box哲学との整合性**
   - プラグインBoxもビルトインBoxも「同じBox」として扱う方法
   - Box型の統一インターフェースの維持

5. **実用性**
   - 他の言語（Python、Ruby、JavaScript）の成功例から学べること
   - プラグイン作者にとっての開発体験
   - デバッグ・プロファイリングの考慮事項

## 📚 参考情報
- 現在のFileBox実装: `src/boxes/file/mod.rs`
- BID-FFIプラグインFileBox: `src/bid/plugins/filebox/mod.rs`
- Everything is Box哲学: すべての値がBoxオブジェクト
- Nyashの目標: シンプル、分かりやすい、階層が深くならない