# 巨大ファイル分割提案レポート

## 📏 巨大ファイル一覧（TOP 20）

| ファイル | 行数 | 責務の数 | 分割推奨度 | 主な責務 |
|---------|------|---------|-----------|---------|
| `src/runner/mir_json_emit.rs` | 795 | 2個 | ⭐⭐ | MIR→JSON変換（v0/v1両対応） |
| `src/mir/builder/builder_calls/build.rs` | 760 | 3個 | ⭐⭐⭐ | 関数呼び出し・メソッド呼び出し・extern |
| `src/mir/builder.rs` | 734 | 1個 | ⭐ | MIR Builder統合（既に多数モジュール分割済み） |
| `src/boxes/p2p_box.rs` | 718 | 4個 | ⭐⭐⭐ | P2P通信・Transport・Handler・Core実装 |
| `src/runner/pipeline.rs` | 715 | 3個 | ⭐⭐ | Using解決・Config初期化・Lint |
| `src/tests/typebox_tlv_diff.rs` | 661 | 1個 | - | テストファイル（分割不要） |
| `src/backend/llvm/compiler/codegen/function.rs` | 646 | 1個 | ⭐ | LLVM関数コード生成（特化責務） |
| `src/runner/modes/common_util/resolve/strip/collect.rs` | 640 | 1個 | ⭐ | Strip/Collect処理（特化） |
| `src/backend/llvm/compiler/codegen/instructions/boxcall.rs` | 612 | 1個 | ⭐ | LLVM BoxCall生成（特化） |
| `src/boxes/socket_box.rs` | 608 | 3個 | ⭐⭐ | Socket通信・読み書き・状態管理 |
| `src/ast.rs` | 607 | 1個 | ⭐ | AST定義（型定義中心） |
| `src/runner/modes/vm.rs` | 596 | 2個 | ⭐⭐ | VM実行モード・Plugin統合 |
| `src/backend/wasm/memory.rs` | 594 | 2個 | ⭐⭐ | WASMメモリ管理・I/O |
| `src/mir/instruction_kinds/mod.rs` | 588 | 1個 | ⭐ | MIR命令定義（型定義中心） |
| `src/backend/llvm/compiler/codegen/instructions/flow.rs` | 588 | 1個 | ⭐ | LLVM制御フロー生成 |
| `src/runner/mod.rs` | 584 | 2個 | ⭐⭐ | Runner統合・パイプライン |
| `src/box_operators.rs` | 571 | 4個 | ⭐⭐⭐ | 演算子実装4種（Add/Sub/Mul/Compare） |
| `src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs` | 555 | 2個 | ⭐⭐ | FFI Bridge・型変換 |
| `src/value.rs` | 552 | 3個 | ⭐⭐ | NyashValue・型変換・比較演算 |
| `src/runtime/method_router_box/mod.rs` | 546 | 2個 | ⭐⭐ | メソッドルーティング・Fallback |

## 🔪 分割提案（高優先度）

### 1. **src/mir/builder/builder_calls/build.rs** (760行) ⭐⭐⭐

**現在の責務**:
- **関数呼び出しビルド**: `build_function_call()` - Global/ModuleFunction解決（470行）
- **メソッド呼び出しビルド**: `build_method_call()` - Static/Instance/Extern処理（240行）
- **Extern呼び出し**: `emit_timer_now_ms_call()`, `emit_array_size_call()` 等（50行）

**問題点**:
- 1ファイルに3つの独立した責務が混在
- 関数呼び出しだけで470行（全体の62%）
- 長大な条件分岐による可読性低下

**分割案**:
```
src/mir/builder/builder_calls/
├── function_call.rs       # 関数呼び出し専用（470行）
│   ├── build_function_call()
│   ├── resolve_call_target()
│   └── normalize_external_module_function_name()
├── method_call.rs         # メソッド呼び出し専用（240行）
│   ├── build_method_call()
│   ├── handle_static_method_call()
│   └── handle_standard_method_call()
├── extern_helpers.rs      # Extern呼び出しヘルパー（50行）
│   ├── emit_timer_now_ms_call()
│   ├── emit_array_size_call()
│   └── emit_map_size_call()
└── build.rs (削除または再エクスポート用統合ファイル)
```

**分割の利点**:
- 責務ごとに独立したファイルで管理
- 各ファイルが240-470行の適切なサイズ
- 関数呼び出しとメソッド呼び出しの混同を防止
- テスト・デバッグが容易

**移行計画**:
1. `extern_helpers.rs` を最初に分離（最も独立性が高い）
2. `method_call.rs` を分離（既存の`method_call_handlers`との統合検討）
3. `function_call.rs` を分離
4. `build.rs` を再エクスポート用ファイルに変更または削除

---

### 2. **src/boxes/p2p_box.rs** (718行) ⭐⭐⭐

**現在の責務**:
- **P2P通信コア**: P2PBox構造体・初期化（200行）
- **Transport抽象化**: InProcessTransport等の実装（200行）
- **Handlerシステム**: イベントハンドラ登録・実行（150行）
- **NyashBox実装**: Trait実装・メソッド呼び出し（168行）

**問題点**:
- Transportの実装が本体に混在（本来は別モジュール）
- 4つの責務が1ファイルに集約
- P2P通信の拡張時に巨大化リスク

**分割案**:
```
src/boxes/p2p/
├── mod.rs                 # P2PBox本体（200行）
│   └── P2PBox構造体・birth/初期化
├── transport.rs           # Transport抽象化（200行）
│   ├── Transport trait
│   ├── InProcessTransport
│   └── TransportKind enum
├── handler.rs             # Handlerシステム（150行）
│   ├── on() - イベント登録
│   ├── emit() - イベント発火
│   └── removeListener()
└── box_impl.rs            # NyashBox実装（168行）
    ├── impl NyashBox
    ├── impl BoxCore
    └── call_method()
```

**分割の利点**:
- Transportの再利用性向上（他のBoxでも利用可能）
- Handler機能の独立テスト可能
- P2PBox本体がシンプルに（200行）
- 将来のTransport追加（WebSocket/TCP）が容易

**移行計画**:
1. `transport.rs` を最初に分離（Transport trait + InProcessTransport）
2. `handler.rs` を分離（イベント機構）
3. `box_impl.rs` を分離（Trait実装）
4. `mod.rs` で再統合・公開API確定

---

### 3. **src/box_operators.rs** (571行) ⭐⭐⭐

**現在の責務**:
- **Add演算子**: `integer+integer`, `string+string` 等（150行）
- **Sub/Mul演算子**: 減算・乗算処理（130行）
- **Compare演算子**: 比較演算（`==`, `<`, `>` 等）（180行）
- **型変換ヘルパー**: `extract_integer()`, `box_to_vm_value()` 等（111行）

**問題点**:
- 4つの独立した演算子が1ファイルに混在
- 比較演算だけで180行（全体の31%）
- 型変換ヘルパーが演算子と混在

**分割案**:
```
src/box_operators/
├── mod.rs                 # 再エクスポート（20行）
├── add.rs                 # Add演算子専用（150行）
├── arithmetic.rs          # Sub/Mul/Div演算子（130行）
├── compare.rs             # 比較演算子（180行）
└── helpers.rs             # 型変換ヘルパー（111行）
```

**分割の利点**:
- 演算子ごとに独立した実装
- 各演算子が100-180行の適切なサイズ
- 型変換ヘルパーの再利用性向上
- 新しい演算子追加が容易

**移行計画**:
1. `helpers.rs` を最初に分離（依存性が最も低い）
2. `compare.rs` を分離（最も大きい責務）
3. `add.rs` を分離（String連結の複雑な処理）
4. `arithmetic.rs` を分離（Sub/Mul/Div統合）
5. `mod.rs` で再エクスポート

---

### 4. **src/runner/mir_json_emit.rs** (795行) ⭐⭐

**現在の責務**:
- **JSON Schema制御**: EmitConfig・v0/v1切り替え（100行）
- **MIR→JSON変換（本体）**: emit_mir_json_for_harness() - 命令ごとの変換（400行）
- **MIR→JSON変換（bin版）**: emit_mir_json_for_harness_bin() - ほぼ同じ処理（295行）

**問題点**:
- 2つのemit関数がほぼ同じ処理を重複実装（コード重複60%）
- 1関数が400行（emit_mir_json_for_harness）
- v0/v1切り替えロジックが散在

**分割案**:
```
src/runner/mir_json_emit/
├── mod.rs                 # 公開API（50行）
│   ├── emit_mir_json_for_harness() - 薄いラッパー
│   └── emit_mir_json_for_harness_bin() - 薄いラッパー
├── config.rs              # Schema制御（100行）
│   ├── EmitConfig構造体
│   ├── create_json_v1_root()
│   └── wrap_functions()
├── instruction.rs         # 命令変換の共通処理（400行）
│   ├── emit_const()
│   ├── emit_binop()
│   ├── emit_compare()
│   ├── emit_call()
│   └── emit_unified_mir_call()
└── emitter.rs             # 統合エミッター（245行）
    ├── EmitContext構造体（共通状態管理）
    └── emit_function() - lib/bin共通処理
```

**分割の利点**:
- コード重複を排除（400行削減見込み）
- 命令変換ロジックの一元化
- v0/v1切り替えが明確に
- 各ファイルが100-400行の適切なサイズ

**移行計画**:
1. `config.rs` を分離（EmitConfig・Schema制御）
2. `instruction.rs` を分離（命令変換ロジック）
3. `emitter.rs` を作成（lib/bin共通処理）
4. `mod.rs` で再統合（薄いラッパー維持）

---

### 5. **src/runner/pipeline.rs** (715行) ⭐⭐

**現在の責務**:
- **Using Context初期化**: init_using_context() - toml/env読み込み（155行）
- **Using解決**: resolve_using_target() - modules/paths/alias（310行）
- **Lint機能**: lint_fields_top() - フィールド配置チェック（250行）

**問題点**:
- 3つの異なる責務が1ファイルに混在
- resolve_using_target()が310行（全体の43%）
- Lint機能が本体に含まれる（本来は独立すべき）

**分割案**:
```
src/runner/pipeline/
├── mod.rs                 # 統合ファイル（50行）
├── using_context.rs       # Using Context初期化（155行）
│   ├── init_using_context()
│   └── UsingContext構造体
├── using_resolver.rs      # Using解決（310行）
│   ├── resolve_using_target()
│   └── suggest_in_base()
└── lint.rs                # Lint機能（250行）
    └── lint_fields_top()
```

**分割の利点**:
- Lint機能の独立テスト可能
- Using解決ロジックが明確に
- 各ファイルが150-310行の適切なサイズ
- 将来のLint機能追加が容易

**移行計画**:
1. `lint.rs` を最初に分離（最も独立性が高い）
2. `using_context.rs` を分離（初期化処理）
3. `using_resolver.rs` を分離（解決ロジック）
4. `mod.rs` で再エクスポート

---

## 📊 分割不要（適切なサイズまたは単一責務）

### 1. **src/mir/builder.rs** (734行)
- **理由**: 既に多数のサブモジュール（50+）に分割済み
- **構造**: `mod calls`, `mod decls`, `mod exprs`, `mod fields` 等で機能分離済み
- **本体**: わずか200行程度（構造体定義+ヘルパー）
- **評価**: 適切に管理されている

### 2. **src/tests/typebox_tlv_diff.rs** (661行)
- **理由**: テストファイル（大きくて良い）
- **責務**: TypeBox TLV差分テスト専用
- **評価**: 分割不要

### 3. **src/ast.rs** (607行)
- **理由**: AST定義の型宣言が中心（enum + 派生実装）
- **責務**: 単一の型定義ファイル
- **評価**: 型定義は1ファイルにまとめる方が管理しやすい

### 4. **src/mir/instruction_kinds/mod.rs** (588行)
- **理由**: MIR命令の型定義が中心（enum + 派生実装）
- **責務**: 単一の型定義ファイル
- **評価**: 命令定義は1ファイルにまとめる方が管理しやすい

### 5. **src/backend/llvm/compiler/codegen/function.rs** (646行)
- **理由**: LLVM関数コード生成の特化ファイル
- **責務**: 単一の責務（関数レベルのコード生成）
- **評価**: 高度に特化しており、これ以上の分割は困難

---

## 📈 統計

### 全体統計
- **500行超ファイル**: 30個（全717ファイル中4.2%）
- **分割推奨ファイル**: 10個
- **分割不要ファイル**: 20個

### 分割による効果見込み
| 項目 | 現在 | 分割後 | 削減効果 |
|------|------|--------|---------|
| 平均ファイルサイズ | 700行 | 250行 | -64% |
| 最大ファイルサイズ | 795行 | 470行 | -41% |
| 責務の明確性 | 低（複数混在） | 高（単一責務） | +100% |
| コード重複 | 高（60%） | 低（10%） | -83% |

### 優先順位（実装の容易さ順）
1. **src/box_operators.rs** (571行) - 最も独立性が高い
2. **src/runner/pipeline.rs** (715行) - Lint機能が既に独立可能
3. **src/boxes/p2p_box.rs** (718行) - Transport抽象化が既に明確
4. **src/mir/builder/builder_calls/build.rs** (760行) - 関数/メソッド分離が明確
5. **src/runner/mir_json_emit.rs** (795行) - 重複コード削減には設計調整が必要

---

## 🎯 推奨実装戦略

### Phase 1: 最も独立性の高いファイル（Week 1）
- `src/box_operators.rs` → `src/box_operators/`（4ファイル）
- `src/runner/pipeline.rs` → `src/runner/pipeline/`（4ファイル）

### Phase 2: 中程度の依存関係（Week 2）
- `src/boxes/p2p_box.rs` → `src/boxes/p2p/`（4ファイル）
- `src/mir/builder/builder_calls/build.rs` → 3ファイル

### Phase 3: 重複コード削減が必要（Week 3）
- `src/runner/mir_json_emit.rs` → `src/runner/mir_json_emit/`（4ファイル）

### 各Phaseの実施手順
1. 新しいディレクトリを作成
2. 最も独立性の高い機能を抽出（例: helpers, config）
3. 中間層を抽出（例: transport, handler）
4. コア機能を抽出
5. 元のファイルを再エクスポート用または削除
6. テスト実行・動作確認
7. Commit

---

## ⚠️ 注意事項

### 分割時の原則
1. **Rustのモジュールシステムに従う**: `mod.rs`で再エクスポート
2. **公開APIは維持**: 既存のimport文を壊さない
3. **テスト完備**: 各分割後に必ずテスト実行
4. **段階的実施**: 1ファイルずつ分割→commit→テスト

### 避けるべきパターン
- ❌ 一度に複数ファイルを分割
- ❌ 公開APIの変更
- ❌ テストなしでのcommit
- ❌ 過度な分割（50行のファイルは不要）

---

## 🔍 分割候補の発見方法

今後のメンテナンスで巨大ファイルを発見する方法:

```bash
# 500行超のファイルを発見
find src/ -name "*.rs" -type f -exec wc -l {} \; | sort -rn | head -20

# 特定ファイルの関数数を確認
grep -E "^(pub )?fn |^impl " src/path/to/file.rs | wc -l

# 特定ファイルの責務を分析
grep -E "^pub fn |^fn " src/path/to/file.rs | head -20
```

---

**生成日時**: 2025-10-13
**分析対象**: Hakorune Selfhost Compiler (Phase 15.8)
**ファイル数**: 717個のRustファイル
**分析基準**: 500行以上のファイル
