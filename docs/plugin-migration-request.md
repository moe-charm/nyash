# 📦 Nyash ビルトインBox → プラグイン化移行依頼

## 🎯 概要
NyashのビルトインBoxをプラグイン化し、コアを軽量化したい。
FileBoxプラグインの成功例を参考に、以下のBoxを順次プラグイン化してください。

## 📋 移行対象Box一覧

### 🌐 Phase 1: ネットワーク・通信系（最優先）
```
plugins/nyash-http-plugin/
├── HttpClientBox   - HTTP通信クライアント（GET/POST/PUT/DELETE）
├── HTTPServerBox   - HTTPサーバー機能
├── HTTPRequestBox  - HTTPリクエスト表現
└── HTTPResponseBox - HTTPレスポンス表現

plugins/nyash-socket-plugin/
└── SocketBox      - TCP/UDPソケット通信
```

### 🖼️ Phase 2: GUI・グラフィック系
```
plugins/nyash-egui-plugin/
└── EguiBox        - デスクトップGUI（既にfeature分離済み）

plugins/nyash-canvas-plugin/
├── CanvasEventBox - Canvas描画イベント
└── CanvasLoopBox  - Canvas描画ループ

plugins/nyash-web-plugin/（WASM専用）
├── WebDisplayBox  - HTML表示
├── WebConsoleBox  - ブラウザコンソール
└── WebCanvasBox   - Canvas描画
```

### 🎵 Phase 3: 特殊用途系
```
plugins/nyash-audio-plugin/
├── AudioBox       - 音声再生・合成
└── SoundBox       - 効果音再生

plugins/nyash-qr-plugin/
└── QRBox          - QRコード生成

plugins/nyash-stream-plugin/
└── StreamBox      - ストリーム処理

plugins/nyash-timer-plugin/
└── TimerBox       - タイマー機能
```

## 🔧 実装ガイドライン

### 1. 参考にするファイル
- **成功例**: `plugins/nyash-filebox-plugin/` - 動作確認済みのFileBoxプラグイン
- **設定例**: `nyash.toml` - 型情報定義の書き方
- **テスト**: `tools/plugin-tester/` - プラグイン診断ツール

### 2. 各プラグインの構成
```
plugins/nyash-xxx-plugin/
├── Cargo.toml      # 依存関係（例: reqwest for HTTP）
├── src/
│   └── lib.rs      # FFI実装
├── nyash.toml      # 型情報定義
└── README.md       # 使用方法
```

### 3. nyash.toml記述例（HttpClientBoxの場合）
```toml
[plugins.HttpClientBox.methods]
# GETリクエスト
get = { 
    args = [{ name = "url", from = "string", to = "string" }],
    returns = "string"
}

# POSTリクエスト
post = { 
    args = [
        { name = "url", from = "string", to = "string" },
        { name = "body", from = "string", to = "string" }
    ],
    returns = "string"
}

# ヘッダー付きリクエスト
request = {
    args = [
        { name = "method", from = "string", to = "string" },
        { name = "url", from = "string", to = "string" },
        { name = "options", from = "map", to = "map" }
    ],
    returns = "map"  # { status, body, headers }
}

# DELETE リクエスト
delete = {
    args = [{ name = "url", from = "string", to = "string" }],
    returns = "string"
}

# PUT リクエスト  
put = {
    args = [
        { name = "url", from = "string", to = "string" },
        { name = "body", from = "string", to = "string" }
    ],
    returns = "string"
}
```

### 4. テスト方法
```bash
# ビルド
cd plugins/nyash-xxx-plugin
cargo build --release

# plugin-testerで診断
cd ../../tools/plugin-tester
./target/release/plugin-tester ../../plugins/nyash-xxx-plugin/target/release/libnyash_xxx_plugin.so

# Nyashで実行テスト
./target/release/nyash test_xxx.nyash
```

## 📝 特記事項

### HttpBox系
- 現在スタブ実装なので移行しやすい
- reqwest依存を復活させる
- 非同期処理の考慮が必要

### EguiBox
- 既にfeature分離されているので参考になる
- メインスレッド制約に注意

### AudioBox/SoundBox
- プラットフォーム依存性が高い
- Web/Desktop両対応を検討

### 依存関係の管理
- 各プラグインは独立したCargo.tomlを持つ
- ビルド時間短縮のため最小限の依存にする

## 💡 実装の重要ポイント

### FFI境界での注意事項
1. **メモリ管理**: 
   - Rustの所有権とCのメモリ管理の違いに注意
   - 文字列は必ずCString/CStr経由で変換
   
2. **エラーハンドリング**:
   - パニックをFFI境界で止める（catch_unwind使用）
   - エラーコードで通信（0=成功, 負値=エラー）

3. **型変換パターン** (FileBoxプラグインより):
```rust
// Nyash文字列 → Rust文字列
let path = get_string_arg(&args[0], 0)?;

// Rust文字列 → Nyash文字列
encode_string_result(&contents, result, result_len)
```

### 参考ファイルの具体的パス
- **FileBoxプラグイン実装**: `plugins/nyash-filebox-plugin/src/lib.rs`
- **FFI仕様書**: `docs/説明書/reference/plugin-system/ffi-abi-specification.md`
- **プラグインシステム説明**: `docs/説明書/reference/plugin-system/plugin-system.md`
- **BID-FFI型変換** (参考): `src/bid-converter-copilot/tlv.rs`

## 📅 推奨実装順序とロードマップ

### Week 1: HttpBox系（最も簡単）
- 既にスタブ実装済み
- reqwest依存を追加するだけ
- FileBoxと同じパターンで実装可能

### Week 2: 特殊用途系（独立性高い）
- QRBox: 単機能で簡単
- TimerBox: 非同期処理の練習に最適
- StreamBox: 中程度の複雑さ

### Week 3: GUI/グラフィック系（プラットフォーム依存）
- EguiBox: feature分離済みなので参考になる
- Canvas系: Web/Desktop両対応必要
- Audio系: 最も複雑（最後に実装）

## 🎯 期待される効果
1. **ビルド時間**: 3分 → 30秒以下
2. **バイナリサイズ**: 最小構成で500KB以下
3. **保守性**: 各プラグイン独立開発可能
4. **拡張性**: ユーザーが独自プラグイン作成可能

## 📝 質問・相談先
- プラグイン化で不明な点があれば、FileBoxプラグインの実装を参考に
- FFI実装で困ったら、plugin-testerのソースコードも参考になります
- nyash.tomlの型定義で迷ったら、既存のFileBox定義を真似してください