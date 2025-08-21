# Nyash開発ガイド for Claude

Nyashプログラミング言語開発に必要な情報をまとめたクイックリファレンス。

## 🧭 Start Here (最初に見る)
- **🎯 主軸タスク**: [docs/予定/native-plan/copilot_issues.txt](docs/予定/native-plan/copilot_issues.txt) **← 最重要！**
- 現在のタスク: [docs/CURRENT_TASK.md](docs/CURRENT_TASK.md)
- コア概念（速習）: [docs/nyash_core_concepts.md](docs/nyash_core_concepts.md)

## 🤖 **Claude×Copilot協調開発の主軸**
### 📋 **copilot_issues.txt - 開発の軸となるファイル**
**すべてはここに書いてある！** - Phase順タスク・優先順位・技術詳細

- **Phase 8.4**: AST→MIR Lowering完全実装（最優先）
- **Phase 8.5**: MIRダイエット（35命令→20命令）
- **Phase 8.6**: VM性能改善（0.9倍 → 2倍以上）
- **Phase 9**: JIT実装
- **Phase 10**: AOT最終形態

**迷ったらcopilot_issues.txtを確認せよ！**

## 🚀 クイックスタート

### 🎯 実行方式選択 (重要!)
- **実行バックエンド完全ガイド**: [docs/execution-backends.md](docs/execution-backends.md) 
  - インタープリター（開発・デバッグ）/ VM（高速実行）/ WASM（Web配布）
  - ⚡ **ベンチマーク機能**: `--benchmark` で3バックエンド性能比較（13.5倍実行高速化実証済み！）

### 🐧 Linux/WSL版
```bash
# ビルドと実行（32スレッド並列ビルド）
cargo build --release -j32
./target/release/nyash program.nyash

# 高速VM実行
./target/release/nyash --backend vm program.nyash

# WASM生成
./target/release/nyash --compile-wasm program.nyash

# ⚡ ベンチマーク実行（性能比較）
./target/release/nyash --benchmark --iterations 100
```

### 🪟 Windows版 (NEW!)
```bash
# クロスコンパイルでWindows実行ファイル生成
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc --release

# 生成された実行ファイル (916KB)
target/x86_64-pc-windows-msvc/release/nyash.exe
```

### 🌐 WebAssembly版
```bash
# WASMビルド方法1: nyash-wasmプロジェクトで直接ビルド
cd projects/nyash-wasm
wasm-pack build --target web

# WASMビルド方法2: build.shスクリプト使用（古い方法）
cd projects/nyash-wasm
./build.sh

# 開発サーバー起動（ポート8010推奨）
python3 -m http.server 8010

# ブラウザでアクセス
# http://localhost:8010/nyash_playground.html
# http://localhost:8010/enhanced_playground.html
# http://localhost:8010/canvas_playground.html
```

**注意**: WASMビルドでは一部のBox（TimerBox、AudioBox等）は除外されます。

## 📚 ドキュメント構造

### 🎯 **最重要ドキュメント（開発者向け）**
- **[copilot_issues.txt](docs/予定/native-plan/copilot_issues.txt)** - **Phase順開発計画の軸**
- **[CURRENT_TASK.md](docs/CURRENT_TASK.md)** - 現在進行状況詳細
- **[native-plan/README.md](docs/予定/native-plan/README.md)** - ネイティブビルド計画

### 📖 利用者向けドキュメント
- 説明書: docs/説明書/README.md
  - ネイティブビルド: docs/説明書/native-build/README.md
  - WASM: docs/説明書/wasm/
  - リファレンス: docs/説明書/reference/
- 予定: docs/予定/README.md
  - CURRENT_TASK, ネイティブ計画, フェーズ課題
- archive: docs/archive/
### 🎯 よく使う情報
- Getting Started: docs/説明書/GETTING_STARTED.md
- Language Guide: docs/説明書/LANGUAGE_GUIDE.md
- Playground Guide: docs/説明書/guides/playground_guide.md
### 📊 最新開発状況
- 現在のタスク: docs/CURRENT_TASK.md
- 予定インデックス: docs/予定/README.md
### 📖 詳細リファレンス
- リファレンス: docs/説明書/reference/
  - 言語: docs/説明書/reference/language-reference.md
  - デリゲーション構文: docs/説明書/reference/override-delegation-syntax.md
  - ビルトイン: docs/説明書/reference/builtin-boxes.md
  - fini/弱参照: docs/説明書/reference/finalization-system.md
### 🎮 実用例・アプリ
- **[実用例](docs/説明書/guides/)** - サンプルコード・パターン集
- **実装済みアプリ**: サイコロRPG・統計計算・LISPインタープリター

## ⚡ 重要な設計原則

### 🏗️ Everything is Box
- すべての値がBox（StringBox, IntegerBox, BoolBox等）
- ユーザー定義Box: `box ClassName { init { field1, field2 } }`

### 🌟 完全明示デリゲーション（2025-08-11革命）
```nyash
// デリゲーション構文
box Child from Parent {  // from構文でデリゲーション
    init(args) {  // コンストラクタは「init」に統一
        from Parent.init(args)  // 親の初期化
    }
    
    override method() {  // 明示的オーバーライド必須
        from Parent.method()  // 親メソッド呼び出し
    }
}
```

### 🔄 統一ループ構文
```nyash
// ✅ 唯一の正しい形式
loop(condition) { }

// ❌ 削除済み構文
while condition { }  // 使用不可
loop() { }          // 使用不可
```

### 🌟 birth構文 - 生命をBoxに与える（2025-08-15実装）
```nyash
// 🌟 「Boxに生命を与える」直感的コンストラクタ
box Life {
    init { name, energy }
    
    birth(lifeName) {  // ← Everything is Box哲学を体現！
        me.name = lifeName
        me.energy = 100
        print("🌟 " + lifeName + " が誕生しました！")
    }
}

// 🔄 デリゲーションでのbirth
box Human from Life {
    init { intelligence }
    
    birth(humanName) {
        from Life.birth(humanName)  // 親のbirthを呼び出し
        me.intelligence = 50
    }
}

// ✅ 優先順位: birth > pack > init > Box名形式
local alice = new Human("Alice")  // birthが使われる
```

### 🚨 pack構文 - ビルトインBox継承専用
```nyash
// ⚠️ pack構文はビルトインBox継承専用！ユーザー定義Boxでは使わない
box EnhancedP2P from P2PBox {
    init { features }
    
    pack(nodeId, transport) {
        from P2PBox.pack(nodeId, transport)  // ビルトイン初期化
        me.features = new ArrayBox()
    }
    
    override send(intent, data, target) {
        me.features.push("send:" + intent)
        return from P2PBox.send(intent, data, target)
    }
}

// ❌ 間違い: ユーザー定義Boxでpack使用
box RegularUser {
    pack(name) {  // これは間違い！birth()を使う
        me.name = name
    }
}
```

### 🎯 正統派Nyashスタイル（2025-08-09実装）
```nyash
// 🚀 Static Box Main パターン - エントリーポイントの統一スタイル
static box Main {
    init { console, result }  // フィールド宣言
    
    main() {
        // ここから始まる！他の言語と同じエントリーポイント
        me.console = new ConsoleBox()
        me.console.log("🎉 Everything is Box!")
        
        // local変数も使用可能
        local temp
        temp = 42
        me.result = temp
        
        return "Revolution completed!"
    }
}
```

### 📝 変数宣言厳密化システム（2025-08-09実装）
```nyash
// 🔥 すべての変数は明示宣言必須！（メモリ安全性・非同期安全性保証）

// ✅ static box内のフィールド
static box Calculator {
    init { result, memory }  // 明示宣言
    
    calculate() {
        me.result = 42  // ✅ フィールドアクセス
        
        local temp     // ✅ local変数宣言
        temp = me.result * 2
    }
}

// ✅ static関数内の所有権移転
static function Factory.create() {
    outbox product  // 呼び出し側に所有権移転
    product = new Item()
    return product
}

// ❌ 未宣言変数への代入はエラー
x = 42  // Runtime Error: 未宣言変数 + 修正提案
```

### ⚡ 実装済み演算子（Production Ready）
```nyash
// 論理演算子（完全実装）
not condition    // NOT演算子
a and b         // AND演算子  
a or b          // OR演算子

// 算術演算子
a / b           // 除算（ゼロ除算エラー対応済み）
a + b, a - b, a * b  // 加算・減算・乗算
```

### ⚠️ 重要な注意点
```nyash
// ✅ 正しい書き方
init { field1, field2 }  // カンマ必須（CPU暴走防止）

// ❌ 間違い
init { field1 field2 }   // カンマなし→CPU暴走
```

## 🎨 GUI開発（NEW!）

### EguiBox - GUIアプリケーション開発
```nyash
// EguiBoxでGUIアプリ作成
local app
app = new EguiBox()
app.setTitle("Nyash GUI App") 
app.setSize(800, 600)

// 注意: 現在メインスレッド制約により
// app.run() は特別な実行コンテキストが必要
```

**実装状況**: 基本実装完了、GUI実行コンテキスト対応中

## 📖 ドキュメントファースト開発（重要！）

### 🚨 開発手順の鉄則
**絶対にソースコードを直接読みに行かない！必ずこの順序で作業：**

1. **📚 ドキュメント確認** - まず既存ドキュメントをチェック
2. **🔄 ドキュメント更新** - 古い/不足している場合は更新
3. **💻 ソース確認** - それでも解決しない場合のみソースコード参照

### 🎯 最重要ドキュメント（2つの核心）

#### 🔤 言語仕様
- **[構文早見表](docs/quick-reference/syntax-cheatsheet.md)** - 基本構文・よくある間違い
- **[完全リファレンス](docs/説明書/reference/)** - 言語仕様詳細
- **[予約語一覧](docs/説明書/reference/keywords.md)** - from, override, pack等

#### 📦 主要BOXのAPI
- **[ビルトイン型](docs/説明書/reference/built-in-boxes.md)** - 全Box型API仕様
- **P2PBox & IntentBox** - docs/説明書/reference/built-in-boxes.md内
- **StringBox, IntegerBox, ConsoleBox** - 基本Box API
- **EguiBox, DebugBox, MathBox** - 特殊Box API

### ⚡ API確認の実践例
```bash
# ❌ 悪い例：いきなりソース読む
Read src/boxes/p2p_box.rs  # 直接ソース参照

# ✅ 良い例：ドキュメント優先
Read docs/説明書/reference/built-in-boxes.md  # まずドキュメント
# → 古い/不足 → ドキュメント更新
# → それでも不明 → ソース確認
```

## 🔧 開発サポート

### 🤖 AI相談
```bash
# Gemini CLIで相談
gemini -p "Nyashの実装で困っています..."
```

### 🧪 テスト実行

#### 📁 **テストファイル配置ルール（重要！）**
- **local_testsフォルダを使用**: 一時的なテストファイルは`local_tests/`に配置
- **ルートディレクトリには置かない**: プロジェクトルートが散らからないように
- **実行例**: `./target/debug/nyash local_tests/test_example.nyash`

```bash
# 基本機能テスト
cargo test

# テストファイル作成・実行例
mkdir -p local_tests
echo 'print("Hello Nyash!")' > local_tests/test_hello.nyash
./target/debug/nyash local_tests/test_hello.nyash

# 演算子統合テスト
./target/debug/nyash test_comprehensive_operators.nyash

# 実用アプリテスト
./target/debug/nyash app_dice_rpg.nyash
```

#### 🔌 **プラグインテスター（BID-FFI診断ツール）**
```bash
# プラグインテスターのビルド
cd tools/plugin-tester
cargo build --release

# プラグインの診断実行
./target/release/plugin-tester ../../plugins/nyash-filebox-plugin/target/debug/libnyash_filebox_plugin.so

# 出力例：
# Plugin Information:
#   Box Type: FileBox (ID: 6)  ← プラグインが自己宣言！
#   Methods: 6
#   - birth [ID: 0] (constructor)
#   - open, read, write, close
#   - fini [ID: 4294967295] (destructor)
```

**plugin-testerの特徴**:
- Box名を決め打ちしない汎用設計
- プラグインのFFI関数4つ（abi/init/invoke/shutdown）を検証
- birth/finiライフサイクル確認
- 将来の拡張: TLV検証、メモリリーク検出

### ⚠️ **ビルド時間に関する重要な注意**
**wasmtime依存関係により、フルビルドは2-3分かかります。**
- タイムアウトエラーを避けるため、ビルドコマンドには十分な時間を設定してください
- 例: `cargo build --release -j32` （3分以上待つ）
- プラグインのみのビルドは数秒で完了します
- Phase 9.75fで動的ライブラリ分離により改善作業中

### 🔧 **Rustビルドエラー対処法**
**Rustのコンパイルエラーは詳細が見づらいため、以下のパターンで対処：**

#### 1. エラーをファイルに出力
```bash
# エラーをファイルに保存して解析
cargo build --lib -j32 2>&1 > build_errors.txt

# 特定のエラーコードを検索
grep -A10 "error\[E0308\]" build_errors.txt
```

#### 2. 32スレッドビルドの基本ルール
- **時間制限なし**: `--timeout 300000` (5分)以上を設定
- **エラー出力**: 必ずファイルに保存して解析
- **並列度**: `-j32` で最大並列化

#### 3. よくあるエラーパターン
- `Box<dyn NyashBox>` vs `Arc<dyn NyashBox>`: `.into()` で変換
- `unsafe` ブロックでの型推論: 明示的な型指定が必要
- deprecatedワーニング: MIR命令の移行期間中は無視可

### 🐛 デバッグ

#### パーサー無限ループ対策（NEW! 2025-08-09）
```bash
# 🔥 デバッグ燃料でパーサー制御
./target/release/nyash --debug-fuel 1000 program.nyash      # 1000回制限
./target/release/nyash --debug-fuel unlimited program.nyash  # 無制限
./target/release/nyash program.nyash                        # デフォルト10万回

# パーサー無限ループが検出されると自動停止＋詳細情報表示
🚨 PARSER INFINITE LOOP DETECTED at method call argument parsing
🔍 Current token: IDENTIFIER("from") at line 17
🔍 Parser position: 45/128
```

**対応状況**: must_advance!マクロでパーサー制御完全実装済み✅  
**効果**: 予約語"from"など問題のあるトークンも安全にエラー検出

#### アプリケーション デバッグ
```nyash
// DebugBox活用
DEBUG = new DebugBox()
DEBUG.startTracking()
DEBUG.trackBox(myObject, "説明")
print(DEBUG.memoryReport())
```

## 📚 ドキュメント再編成戦略

### 🎯 現在の課題
- **CLAUDE.md肥大化** (500行) - 必要情報の検索困難
- **情報分散** - 実装状況がCLAUDE.md/current_task/docsに分散
- **参照関係不明確** - ファイル間の相互リンク不足

### 🚀 新構造プラン
```
docs/
├── quick-reference/          # よく使う情報（簡潔）
│   ├── syntax-cheatsheet.md     # 構文早見表
│   ├── operators-summary.md     # 演算子一覧
│   └── development-commands.md  # 開発コマンド集
├── status/                   # 最新開発状況
│   ├── current-implementation.md  # 実装状況詳細
│   ├── recent-achievements.md     # 最新成果
│   └── known-issues.md            # 既知の問題
├── reference/                # 完全リファレンス（現存活用）
└── examples/                 # 実用例（現存拡充）
```

### ⚡ 実装優先順位
1. **Phase 1**: CLAUDE.md簡潔化（500行→150行ハブ）
2. **Phase 2**: 基本構造作成・情報移行
3. **Phase 3**: 相互リンク整備・拡充

### 🎉 期待効果
- **検索性**: 必要情報への高速アクセス
- **メンテナンス性**: 責任分離・局所的更新
- **拡張性**: 新機能追加が容易

**📋 詳細**: [DOCUMENTATION_REORGANIZATION_STRATEGY.md](DOCUMENTATION_REORGANIZATION_STRATEGY.md)

## 🤝 プロアクティブ開発方針

### 🎯 エラー対応時の姿勢
エラーを見つけた際は、単に報告するだけでなく：

1. **🔍 原因分析** - エラーの根本原因を探る
2. **📊 影響範囲** - 他のコードへの影響を調査
3. **💡 改善提案** - 関連する問題も含めて解決策を提示
4. **🧹 機会改善** - デッドコード削除など、ついでにできる改善も実施

### ⚖️ バランスの取り方
- **積極的に分析・提案**するが、最終判断はユーザーに委ねる
- 「ChatGPTさんに任せてる」と言われても、分析結果は共有する
- 複数のAIが協調する場合でも、各自の視点で価値を提供する

### 📝 例
```
❌ 受動的: 「エラーをファイルに出力しました」
✅ 能動的: 「エラーをファイルに出力しました。主な原因は型の不一致（7箇所）で、
          instance_id()のメソッド呼び出し修正で5つ解決できそうです。
          また、関連してclone_boxの実装にも同様の問題を発見しました。」
```

## 🚨 コンテキスト圧縮時の重要ルール

### ⚠️ **コンテキスト圧縮を検出した場合の必須手順**

**コンテキスト圧縮** = 会話履歴が要約される現象（conversation summaryで検出可能）

#### 🛑 **絶対にやってはいけないこと**
- **推測で作業を続行しない**
- 不完全な情報で重要な変更をしない  
- ビルドチェックを飛ばさない
- ユーザー確認なしに進行しない

#### ✅ **必ず実行すべき手順**
1. **⏸️ 作業停止** - 「コンテキスト圧縮を検出しました」と報告
2. **📊 状況確認** - 以下を必ずチェック：
   ```bash
   git status                    # 現在の変更状況
   git log --oneline -3         # 最近のcommit履歴
   cargo check                  # ビルド状況
   ```
3. **📋 現在タスク確認** - `CURRENT_TASK.md` を読み取り
4. **🤝 明示的確認** - ユーザーに「次に何をしましょうか？」と確認

#### 📍 **現在状況の記録場所**
- **進行中タスク**: `CURRENT_TASK.md`
- **最後の安定状態**: git commit hash  
- **ビルド状況**: `cargo check` の結果
- **重要な制約**: CURRENT_TASK.md内の注意事項

#### 💡 **圧縮時によくある混乱の回避**
- 「何をしていたか」→ `CURRENT_TASK.md`で確認
- 「ビルドできるか」→ `cargo check`で確認  
- 「どこまで進んだか」→ `git log`で確認
- 「次は何か」→ **ユーザーに明示的に確認**

## 🔌 プラグインBox開発時の重要な注意点

### ⚠️ **TLV Handle処理の正しい実装方法**

プラグインメソッドがBoxRef（Handle）を返す場合、以下の点に注意：

#### 🛑 **よくある間違い**
```rust
// ❌ 間違い: 元のplugin_boxの値を流用
let new_plugin_box = PluginBoxV2 {
    type_id: plugin_box.type_id,        // ❌ 返り値のtype_idを使うべき
    fini_method_id: plugin_box.fini_method_id,  // ❌ 返り値の型に対応する値を使うべき
    ...
};
```

#### ✅ **正しい実装**
```rust
// ✅ 正解: 返されたHandleから正しい値を取得
let type_id = /* TLVから取得したtype_id */;
let instance_id = /* TLVから取得したinstance_id */;

// 返り値のtype_idに対応する正しいfini_method_idを取得
let fini_method_id = /* configから返り値type_idに対応するfini_method_idを検索 */;

let new_plugin_box = PluginBoxV2 {
    type_id: type_id,           // ✅ 返り値のtype_id
    instance_id: instance_id,   // ✅ 返り値のinstance_id
    fini_method_id: fini_method_id,  // ✅ 返り値の型に対応するfini
    ...
};
```

#### 📝 **重要ポイント**
1. **type_idの正確性**: cloneSelfが返すHandleは必ずしも元のBoxと同じ型ではない
2. **fini_method_idの対応**: 各Box型は独自のfini_method_idを持つ可能性がある
3. **ローダー経由の処理**: 可能な限りplugin_loader_v2経由でメソッドを呼び出す

---

最終更新: 2025年8月20日 - **📝 プラグインBox開発の注意点追加**
- **TLV Handle処理**: type_idとfini_method_idの正しい扱い方を追記
- **Phase 9.75g-0完了**: BID-FFI Step 1-3実装成功（プラグイン・テスター・設定）
- **plugin-tester**: 汎用プラグイン診断ツール完成（CLAUDE.mdに追加）
- **設計原則達成**: Box名非決め打ち・birth/finiライフサイクル・メモリ管理明確化
- **次のステップ**: Step 4 - Nyashとの統合（src/bid/モジュール実装）
- **copilot_issues.txt**: Phase順開発計画の軸として継続
- **次期最優先**: AST→MIR Lowering完全実装（Phase 8.4）
