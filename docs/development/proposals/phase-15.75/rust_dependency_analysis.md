# Rust依存関係 - 完全分析

**Status**: Analysis Report
**Created**: 2025-10-13
**Purpose**: Rust依存の完全な詳細分析

---

## 📊 総合統計

### 全体像
- **総行数**: 99,406行
- **総ファイル数**: 714ファイル
- **外部クレート**: 24個の主要依存

### ディレクトリ別内訳
```
src/
├── backend/               15,722行  (15.8%)
│   ├── mir_interpreter/    5,123行   ← Rust VM (最重要)
│   ├── llvm/              ~5,000行   ← LLVM Backend
│   ├── wasm/              ~3,000行   ← WASM Backend
│   └── aot/               ~2,000行   ← AOT Backend
├── parser/                ~4,000行  (4.0%)
├── tokenizer/             ~3,637行  (3.7%)
├── boxes/                 12,752行  (12.8%)
├── runtime/                9,311行  (9.4%)
│   ├── gc_*.rs               335行   ← GC実装
│   └── plugin_loader_v2/   3,098行   ← Plugin Loader
├── cli/                      619行  (0.6%)
├── tests/                 ~10,000行  (10.1%)
├── ast/                   ~5,000行  (5.0%)
├── runner/                ~8,000行  (8.0%)
├── config/                ~3,000行  (3.0%)
├── using/                 ~2,000行  (2.0%)
└── その他                ~25,365行  (25.6%)

合計: 99,406行 (100%)
```

---

## 🎯 1. Rust VM (5,123行) - 最重要

### 1.1 概要
**場所**: `src/backend/mir_interpreter/`
**役割**: MIR命令を実行するインタープリタ
**脱Rust化可能性**: **高** (Hakorune VMで完全代替可能)

### 1.2 ファイル構成
```
src/backend/mir_interpreter/
├── mod.rs                     ~500行  - メインモジュール
├── exec.rs                    ~530行  - 実行エンジン
├── method_router.rs           ~200行  - メソッドルーティング
├── handlers/                 ~2,500行 - 命令ハンドラー
│   ├── arithmetic.rs          ~300行  - 算術演算
│   ├── boxes_instance.rs      ~400行  - Box生成
│   ├── calls/                ~1,000行 - 呼び出し処理
│   │   ├── function.rs        ~300行
│   │   ├── method.rs          ~300行
│   │   └── legacy/            ~400行
│   ├── externals.rs           ~200行  - Extern呼び出し
│   ├── memory.rs              ~200行  - メモリ操作
│   └── misc.rs                ~200行  - その他
├── resolve/                   ~800行  - 名前解決
└── contracts/                 ~600行  - 契約・ポリシー

合計: 5,123行
```

### 1.3 主要機能
1. **16命令の完全実装**
   - 基本演算: Const, UnaryOp, BinOp, Compare, TypeOp
   - メモリ: Load, Store
   - 制御: Branch, Jump, Return, Phi
   - 呼び出し: Call, BoxCall, ExternCall (MirCallへ統合予定)
   - GC: Barrier, Safepoint
   - 構造: Copy, Nop

2. **トレース機能**
   ```rust
   // HAKO_VM_TRACE環境変数でトレース有効化
   HAKO_VM_TRACE="op=compare,binop,externcall;regs=1;block=*"
   ```

3. **ステッパ機能**
   ```rust
   // HAKO_VM_STEP環境変数でステッパ有効化
   HAKO_VM_STEP=1 HAKO_VM_STEP_ALLOW_BLOCK=1
   ```

4. **PHI命令処理**
   - SSA形式のサポート
   - Predecessor追跡
   - 未定義入力の許容（開発モード）

5. **エラーハンドリング**
   - VMError型の統一
   - スタックトレース
   - 診断メッセージ

### 1.4 Hakorune VMとの比較
| 機能 | Rust VM | Hakorune VM | 差分 |
|------|---------|-------------|------|
| **行数** | 5,123行 | 4,998行 | -2.4% |
| **実装済み命令** | 16/16 (100%) | 15/16 (93%) | MirCallのみ |
| **トレース機能** | 完備 | 基本のみ | 拡張必要 |
| **ステッパ機能** | あり | なし | 追加必要 |
| **エラーハンドリング** | 統一 | ResultBox | 同等 |
| **PHI処理** | 完全 | 完全 | 同等 |

### 1.5 脱Rust化の戦略
**Phase 1**: Hakorune VM MirCall実装（1週間）
- MirCall実装で16命令完全実装
- トレース機能の拡張
- 509テストすべてPASS維持

**期待される効果**:
- ✅ Rust VM (5,123行) の完全削除
- ✅ セルフホスティングの完全実現
- ✅ デバッグ容易性の維持

**リスク**:
- パフォーマンス劣化の可能性 → Phase 2 (AOT化) で解決

---

## 📝 2. Parser/Tokenizer (7,637行)

### 2.1 概要
**場所**: `src/parser/`, `src/tokenizer/`
**役割**: Hakoruneソースコードのパース
**脱Rust化可能性**: **高** (セルフホストコンパイラで代替)

### 2.2 ファイル構成
```
src/tokenizer/              ~3,637行
├── mod.rs                  ~1,000行  - メイントークナイザー
├── cursor.rs                 ~500行  - カーソル管理
├── lex_*.rs                ~1,500行  - 字句解析
│   ├── lex_string.rs         ~400行
│   ├── lex_number.rs         ~300行
│   └── lex_ident.rs          ~300行
└── whitespace.rs             ~200行  - 空白処理

src/parser/                 ~4,000行
├── mod.rs                    ~800行  - メインパーサー
├── expressions.rs          ~1,200行  - 式のパース
├── sugar.rs                  ~600行  - 構文糖衣
├── common.rs                 ~400行  - 共通処理
└── entry_sugar.rs            ~300行  - エントリーポイント糖衣

合計: 7,637行
```

### 2.3 主要機能
1. **字句解析（Tokenizer）**
   - 文字列リテラル、数値リテラル、識別子
   - コメント処理
   - 空白・改行処理

2. **構文解析（Parser）**
   - AST生成
   - 式のパース（算術、論理、比較等）
   - 文のパース（if, loop, return等）
   - Box定義、関数定義

3. **構文糖衣**
   - 演算子の優先順位
   - メソッド呼び出しの糖衣構文
   - パイプライン演算子

### 2.4 セルフホストコンパイラの状況
**M2達成**: 自己ホストコンパイラで再ビルド可能（2025-10-09）
**M3達成**: VM/LLVM Parity（2025-10-11）
**進捗**: 85-90%完成

**残り10-15%**:
- エラーメッセージの改善
- エッジケースの対応
- パフォーマンス最適化

### 2.5 脱Rust化の戦略
**Phase 2**: Parser/Tokenizerのセルフホスト化（1-2週間）
- セルフホストコンパイラの完成（残り15%）
- デュアルパス方式（Rust + Hakorune並行実行）
- パリティテストで互換性確認

**期待される効果**:
- ✅ Parser/Tokenizer (7,637行) の完全削除
- ✅ セルフホスティングの完全独立

**リスク**:
- エッジケースの互換性 → テストカバレッジ高い

---

## 📦 3. Boxes実装 (12,752行)

### 3.1 概要
**場所**: `src/boxes/`
**役割**: 基本的なBoxの実装（String, Integer, Array, Map等）
**脱Rust化可能性**: **高** (プラグイン化で代替)

### 3.2 ファイル構成
```
src/boxes/                  12,752行
├── core/                   ~4,000行  - コアBox
│   ├── string_box.rs       ~1,000行
│   ├── integer_box.rs        ~500行
│   ├── bool_box.rs           ~300行
│   ├── array_box.rs        ~1,200行
│   ├── map_box.rs          ~1,000行
│   └── null_box.rs           ~200行
├── io/                     ~2,000行  - IO関連
│   ├── file_box.rs         ~1,000行
│   ├── buffer_box.rs         ~500行
│   └── path_box.rs           ~500行
├── async/                  ~1,500行  - 非同期
│   ├── future_box.rs         ~800行
│   ├── task_box.rs           ~400行
│   └── promise_box.rs        ~300行
├── error/                  ~1,000行  - エラー処理
│   ├── result_box.rs         ~600行
│   └── exception_box.rs      ~400行
├── network/                ~1,500行  - ネットワーク
│   ├── http_box.rs           ~800行
│   └── socket_box.rs         ~700行
└── その他                  ~2,752行  - その他Box

合計: 12,752行
```

### 3.3 Phase 15.6計画（既存）
**核心コンセプト**:
```
plugins/          ← すべてのBox実装（唯一の管理場所）
  ├── core系      ← 静的リンク候補（hako_kernel features）
  └── 拡張系      ← 動的ロード

src/boxes/        ← 完全削除（段階的）

方針: 単一ソース + ビルド分岐（動的 or 静的）
```

**実装戦略**（30-40時間見積もり）:
- Week 1: 基盤系プラグイン化（FutureBox, ResultBox, NullBox等 7個）
- Week 2: IO/ネットワーク系（BufferBox, HTTPBox, SocketBox等）
- Week 3: 統合テスト、src/boxes/ 削除

### 3.4 脱Rust化の戦略
**Phase 3**: Boxes実装のプラグイン化（4-6週間）
- Phase 15.6計画の実行
- 単一ソース原則の実現
- 静的/動的ビルド分岐

**期待される効果**:
- ✅ Boxes実装 (12,752行) の完全削除
- ✅ プラグインシステムの完全確立

**リスク**:
- 重複登録ガードの実装
- 既存コードとの互換性維持

---

## 🔧 4. Runtime (9,311行)

### 4.1 概要
**場所**: `src/runtime/`
**役割**: GC、型システム、モジュールシステム等
**脱Rust化可能性**: **中** (GCはパフォーマンス重視で残す)

### 4.2 ファイル構成
```
src/runtime/                9,311行
├── gc_*.rs                   335行  - GC実装（最小限残す）
│   ├── gc_controller.rs      206行
│   ├── gc.rs                  60行
│   ├── gc_trace.rs            35行
│   └── gc_mode.rs             34行
├── plugin_loader_v2/       3,098行  - Plugin Loader (残す)
├── type_*.rs              ~1,500行  - 型システム
├── nyash_runtime.rs         ~500行  - ランタイムコア
├── semantics.rs             ~400行  - 意味論
├── scheduler.rs             ~300行  - スケジューラ
├── modules_registry.rs      ~300行  - モジュールレジストリ
├── extern_registry.rs       ~200行  - Extern登録
├── method_router_box/       ~800行  - メソッドルーター
├── host_api*.rs             ~500行  - Host API
└── その他                 ~1,378行  - その他

合計: 9,311行
```

### 4.3 GC実装の詳細
**場所**: `src/runtime/gc_*.rs` (335行)

#### gc_controller.rs (206行)
```rust
// GCコントローラー - メイン制御ロジック
pub struct GcController {
    threshold: usize,
    enable: bool,
    mode: GcMode,
}

// 主要機能:
- collect(): GC実行
- should_collect(): GC実行判定
- update_threshold(): 閾値更新
```

#### gc.rs (60行)
```rust
// GCアルゴリズム - マークアンドスイープ
pub fn mark_and_sweep(roots: &[BoxRef])
pub fn mark_recursive(bx: &BoxRef)
```

#### gc_trace.rs (35行)
```rust
// GCトレース - デバッグ用
pub fn trace_gc_event(event: &str)
```

#### gc_mode.rs (34行)
```rust
// GCモード - 動作モード定義
pub enum GcMode {
    Auto,      // 自動
    Manual,    // 手動
    Disabled,  // 無効
}
```

### 4.4 脱Rust化の戦略
**Phase 4**: Runtime部分の段階的置き換え（6-8週間）
- 型システム → Hakorune実装
- モジュールシステム → Hakorune実装
- GC → 最小限のC ABIのみ（335行 → 200行）
- スケジューラ → Hakorune実装

**期待される効果**:
- ✅ Runtime (9,311行) の大部分を脱Rust化
- ✅ GC (335行 → 200行) の最小化

**リスク**:
- パフォーマンス劣化
- メモリ安全性の確保

---

## 🔌 5. Plugin Loader (3,098行)

### 5.1 概要
**場所**: `src/runtime/plugin_loader_v2/`
**役割**: プラグインの動的ロード
**脱Rust化可能性**: **低** (C ABI必須)

### 5.2 ファイル構成
```
src/runtime/plugin_loader_v2/  3,098行
├── mod.rs                      ~300行  - メインモジュール
├── enabled/                  ~2,500行  - 有効化時の実装
│   ├── mod.rs                  ~400行
│   ├── ffi_bridge.rs           ~500行  - FFIブリッジ
│   ├── host_bridge.rs          ~400行  - ホストブリッジ
│   ├── loader/               ~1,000行  - ローダー
│   └── extern_functions/       ~200行  - Extern関数
└── stub.rs                     ~298行  - スタブ（無効化時）

合計: 3,098行
```

### 5.3 主要機能
1. **動的ロード**
   - .so/.dll/.dylibの読み込み
   - シンボル解決
   - バージョン管理

2. **FFIブリッジ**
   - C ABI経由の通信
   - 型変換（Rust ↔ C）
   - エラーハンドリング

3. **ホストブリッジ**
   - プラグイン → ホスト呼び出し
   - ホスト → プラグイン呼び出し
   - ライフサイクル管理

### 5.4 脱Rust化の戦略
**Phase 4**: 最小限のC ABI層の確立（Runtime置き換えに含む）
- Plugin Loader自体はRustで維持（または最小限のC/C++で再実装）
- C ABI層を最小化（3,098行 → 1,500行）
- Hakorune側からの呼び出しインターフェース統一

**期待される効果**:
- ✅ Plugin Loader (3,098行 → 1,500行) の最小化

**リスク**:
- C/C++での再実装の複雑性
- メモリ管理の難しさ

---

## 🖥️ 6. CLI (619行)

### 6.1 概要
**場所**: `src/cli/`, `src/bin/`
**役割**: コマンドライン引数の処理
**脱Rust化可能性**: **高** (優先度低)

### 6.2 ファイル構成
```
src/cli/                     619行
├── mod.rs                  ~200行  - メインモジュール
├── args.rs                 ~300行  - 引数パース
├── groups.rs                ~50行  - グループ定義
└── utils.rs                 ~69行  - ユーティリティ

src/bin/
├── hako.rs                  ~50行  - メインバイナリ
└── hrn.rs                   ~50行  - エイリアス

合計: 619行
```

### 6.3 主要機能
1. **引数パース**
   - clap crateによる引数解析
   - サブコマンド処理
   - フラグ・オプション

2. **環境変数**
   - HAKO_*, NYASH_*環境変数の処理
   - 優先度管理

3. **ヘルプメッセージ**
   - 自動生成
   - ドキュメント統合

### 6.4 脱Rust化の戦略
**Phase 4-5**: CLI実装の置き換え（Runtime置き換えの一部）
- Hakorune実装のCLIパーサー
- C ABI経由でランタイムに渡す

**期待される効果**:
- ✅ CLI (619行) の削除

**リスク**:
- 優先度低（後回し可能）

---

## 🌐 7. 外部クレート依存（24個）

### 7.1 主要依存
```
# エラー処理
anyhow = "1.0"           - エラー処理（代替: 独自実装）
thiserror = "2.0"        - エラー定義（代替: 独自実装）

# CLI
clap = "4.5"             - CLI引数パース（代替: Hakorune実装）

# シリアライゼーション
serde = "1.0"            - JSON処理（代替: 独自実装またはC library）
serde_json = "1.0"       - JSON処理（代替: 独自実装またはC library）
toml = "0.8"             - TOML処理（代替: 独自実装）

# 正規表現
regex = "1.11"           - 正規表現（代替: PCRE2またはRE2）

# プラグイン
libloading = "0.8"       - 動的ロード（代替: dlopen/LoadLibrary）

# WASM
wasm-bindgen = "0.2"     - WASM FFI（必須: 残す）
js-sys = "0.3"           - JavaScript FFI（必須: 残す）
web-sys = "0.3"          - Web API（必須: 残す）

# ユーティリティ
lazy_static = "1.5"      - 遅延初期化（代替: once_cell）
once_cell = "1.21"       - 1回初期化（代替: 独自実装）
log = "0.4"              - ログ（代替: 独自実装）
env_logger = "0.11"      - ログ（代替: 独自実装）

# 開発
criterion = "0.5"        - ベンチマーク（開発時のみ）
```

### 7.2 脱Rust化の戦略
**Phase 4-5**: 外部クレート依存の最小化
- エラー処理 → 独自実装
- JSON処理 → yyjsonまたは独自実装
- 正規表現 → PCRE2またはRE2（C library）
- WASM関連 → 必須（残す）

**期待される効果**:
- ✅ 外部クレート依存を70%削減（24個 → 7個）

**リスク**:
- 独自実装の品質
- メンテナンスコスト

---

## 📊 8. 削減見込みサマリー

### 8.1 Phase別削減見込み
| Phase | 対象 | 現状 | 削減後 | 削減率 |
|-------|------|------|--------|--------|
| **Phase 1** | Rust VM | 5,123行 | 0行 | **100%** |
| **Phase 2** | Parser/Tokenizer | 7,637行 | 0行 | **100%** |
| **Phase 3** | Boxes実装 | 12,752行 | 0行 | **100%** |
| **Phase 4** | Runtime | 9,311行 | ~5,000行 | **46%** |
| **Phase 5** | AOT化 | - | - | - |
| **維持** | LLVM/WASM/Plugin/C ABI | ~10,000行 | ~10,000行 | 0% |

### 8.2 最終的な構成
```
総行数: 約45,000行 (現在: 99,406行)
├── Rust: 15,000行 (33%)
│   ├── C ABI層: ~500行
│   ├── GC: ~200行
│   ├── LLVM Backend: ~5,000行
│   ├── WASM Backend: ~3,000行
│   ├── Plugin Loader: ~1,500行
│   └── その他Runtime: ~4,800行
└── Hakorune: 30,000行 (67%)
    ├── Hakorune VM: ~5,000行
    ├── セルフホストコンパイラ: ~10,000行
    ├── プラグイン: ~10,000行
    └── Runtime: ~5,000行

削減率: 55% (99,406行 → 45,000行)
```

---

## 🎯 9. 結論

### 実現可能性: **高**
- Hakorune VMが既に93%完成
- セルフホストコンパイラがM2/M3達成済み
- Phase 15.6で「Everything is Plugin」計画進行中

### 推奨戦略: **段階的実装**
1. Phase 1: Rust VM → Hakorune VM (1週間)
2. Phase 2: Parser/Tokenizer (1-2週間)
3. Phase 3: Boxes プラグイン化 (4-6週間)
4. Phase 4: Runtime置き換え (6-8週間)
5. Phase 5: AOT化 (4-6週間)

### 期待される効果
- ✅ Rust依存を85%削減（脱Rust化達成）
- ✅ Rust依存を55%削減（総行数ベース）
- ✅ セルフホスティングの完全実現
- ✅ プラグインシステムの完全確立

---

**最終更新**: 2025-10-13
**作成者**: Claude (analysis of 714 Rust files, 99,406 lines)
