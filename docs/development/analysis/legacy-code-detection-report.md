# レガシーコード特定レポート

**調査日**: 2025-10-13
**調査範囲**: `/home/tomoaki/git/hakorune-selfhost/src/` ディレクトリ全体
**総ファイル数**: 717ファイル
**総行数**: 99,928行

---

## 📊 エグゼクティブサマリー

### 削減見積もり
- **高優先度削除**: 約2,400行（2.4%）
- **中優先度要確認**: 約1,900行（1.9%）
- **低優先度リファクタリング**: 約900行（0.9%）
- **合計削減可能**: 約5,200行（5.2%）

### 主要発見
1. **Legacy実装層**: VM interpreter内に1,400+行のlegacy互換層が存在
2. **Phase 15.6完了でsrc/boxes/削除可能**: 57ファイル、推定3,000+行
3. **未使用Backend実装**: Cranelift JIT（246行）、AOT scaffolding（約300行）
4. **BID Codegen実験コード**: 1,894行（参照ゼロ）
5. **バックアップファイル**: 308行の.bakファイルが1個残存

---

## 🗑️ 削除推奨（高優先度）

### 1. バックアップファイル

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047`

- **理由**: タイムスタンプ付きバックアップファイル（2025年）、Gitに履歴あり
- **行数**: 327行
- **影響範囲**: なし（参照されていない）
- **削減**: 327行

**推奨アクション**:
```bash
rm src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047
```

---

### 2. Legacy Call/Box Handlers（VM Interpreter内）

#### 2.1 Legacy Call Handlers

**場所**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/calls/legacy/`

**内訳**:
- `callee_dispatcher.rs`: 2,699行
- `extern_handler.rs`: 15,168行
- `legacy_resolver.rs`: 16,890行
- `method_handler.rs`: 12,701行
- `mod.rs`: 1,632行
- **合計**: 49,090行 → ただし実際には**885行**（wc確認済み）

**理由**:
- LAYER_GUARD.mdで「long-term: deprecate and delete after plugin parity stays green」と明記
- Phase 15でプラグインシステム完成により不要
- 「最後の砦のフォールバック」として残存中

**影響範囲**:
- MirCall Phase 2完了（2025-10-11）により新システムが動作中
- プラグインパリティが安定すれば削除可能

**削減**: 約885行

**推奨アクション**: Phase 15.6完了後、プラグイン安定性確認してから削除

---

#### 2.2 Legacy Box Handlers

**場所**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/boxes/legacy/`

**内訳**:
- `mod.rs`: 260行
- `LAYER_GUARD.md`: ポリシー文書

**理由**: 同上（プラグインシステム完成により不要）

**削減**: 約260行

---

### 3. Legacy MIR Builder Expression Handler

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/mir/builder/exprs_legacy.rs`

- **行数**: 52行
- **用途**: Print/If/Loop/TryCatch等の古いAST→MIR変換
- **理由**: 現在は`exprs.rs`に統合済み
- **参照**: `exprs.rs`からのみ呼び出し（互換性維持目的）

**影響範囲**: `build_expression_impl_legacy()`を呼んでいるコード確認が必要

**削減**: 52行

---

### 4. Plugin Box Legacy Proxy

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/plugin_box_legacy.rs`

- **行数**: 158行
- **用途**: 旧プラグインBoxプロキシ（FFI境界）
- **理由**: Phase 15で新plugin_loader_v2に移行済み
- **参照**: コードベース内で参照ゼロ（grep確認）

**削減**: 158行

---

### 5. MIR Verification Legacy

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/mir/verification/legacy.rs`

- **行数**: 39行
- **用途**: 旧MIR検証ロジック
- **理由**: verification.rsに統合済み
- **参照要確認**: legacy検証が必要な古いMIRが残っている可能性

**削減**: 39行

---

### 6. BID Codegen Copilot実験コード（未使用）

**場所**:
- `/home/tomoaki/git/hakorune-selfhost/src/bid-codegen-from-copilot/`
- `/home/tomoaki/git/hakorune-selfhost/src/bid-converter-copilot/`

- **総行数**: 1,894行
- **理由**: Copilot実験コード、コードベース内で**参照ゼロ**（grep確認）
- **影響範囲**: なし（完全に独立）

**削減**: 1,894行

**推奨アクション**:
1. READMEを確認して実験の目的を把握
2. 有用なアイデアがあればdocs/proposals/ideas/へ移動
3. ディレクトリごと削除

---

### 高優先度削除 小計: 約3,615行

---

## ⚠️ 要確認（中優先度）

### 1. Cranelift JIT Backend（未使用機能）

**場所**: `/home/tomoaki/git/hakorune-selfhost/src/runner/modes/cranelift.rs`

- **行数**: 45行
- **理由**: `#[cfg(feature = "cranelift-jit")]`でゲート、Cargo.tomlに定義なし
- **状況**: 実装はスケルトンのみ（"skeleton"コメント多数）
- **参照**: テストファイルで17箇所参照（すべて`#[cfg(feature = "cranelift-jit")]`付き）

**影響範囲**: feature未定義なので実行不可能

**削減**: 45行（小さいが削除候補）

**推奨アクション**:
- Cranelift JIT計画が存在するか確認
- 計画なければ削除、計画あれば`docs/development/proposals/ideas/`へ移動

---

### 2. AOT Backend（Phase 9実装、未完成）

**場所**: `/home/tomoaki/git/hakorune-selfhost/src/backend/aot/`

- **行数**: 約300行（mod.rs 146行 + サブモジュール）
- **理由**: "Phase 9 Implementation"とあるが、実装途中（TODOコメント多数）
- **状況**: wasmtime precompilation使用（`#[cfg(feature = "wasm-backend")]`）
- **参照**: runner/modes/aot.rs（55行）から使用

**影響範囲**: WASM featureと連動

**削減**: 約350行

**推奨アクション**:
- Phase 9 AOT計画の現状確認
- WASM backendとの統合計画確認
- 使用予定なければ削除

---

### 3. LLVM Backend（Deprecated Shim）

**場所**: `/home/tomoaki/git/hakorune-selfhost/src/backend/llvm/mod.rs`

- **行数**: 16行（shim layer）
- **理由**: "Deprecated shim module for legacy Rust/inkwell backend"と明記
- **状況**: `llvm_legacy`への再エクスポートのみ
- **参照**: `#[cfg(feature = "llvm-inkwell-legacy")]`でゲート

**影響範囲**: llvm_legacyが存在する限り必要

**削減**: 16行（小さいがdeprecated明記）

**推奨アクション**:
- Python/llvmliteへの完全移行後に削除
- src/llvm_py/が主力になれば不要

---

### 4. WASM Backend v2（Phase 12 Scaffolding）

**場所**: `/home/tomoaki/git/hakorune-selfhost/src/backend/wasm_v2/`

- **行数**: 約60行
- **理由**: "Phase 12 scaffolding"、"最小構成"とコメント
- **状況**: vtable/slot解決の実験的実装
- **参照**: コードベース内で**1箇所のみ参照**（backend/mod.rs）

**影響範囲**: WASM backend統合計画次第

**削減**: 約60行

**推奨アクション**:
- WASM backend統合計画確認
- `src/backend/wasm/`へ統合するか判断
- 不要なら削除

---

### 5. src/boxes/ ディレクトリ（Phase 15.6で削除予定）

**場所**: `/home/tomoaki/git/hakorune-selfhost/src/boxes/`

- **ファイル数**: 57ファイル
- **推定行数**: 約3,000行（詳細調査必要）
- **理由**: CLAUDE.mdで「Phase 15.6 - Everything is Plugin」で`src/boxes/`完全削除予定
- **状況**: plugins/への移行中

**Phase 15.6計画**:
```
plugins/          ← すべてのBox実装（唯一の管理場所）
  ├── core系      ← 静的リンク候補
  └── 拡張系      ← 動的ロード

src/boxes/        ← 完全削除（段階的）
```

**影響範囲**: 現在の実行基盤への影響大（段階的削除必須）

**削減**: 約3,000行（Phase 15.6完了後）

**推奨アクション**:
1. ChatGPT5のPhase 15.6実装進捗確認
2. plugins/への移行完了を待つ
3. 段階的に削除（実行基盤から順次）

**重要**: これは**最大の削減機会**だが、Phase 15.6完了が前提

---

### 中優先度要確認 小計: 約3,471行（src/boxes/含む）

---

## 💡 リファクタリング候補（低優先度）

### 1. 重複した型定義・コンバーター

**問題**: bid-converter-copilot/とbid/で型定義が重複している可能性

**調査項目**:
- `src/bid/types.rs`
- `src/bid-converter-copilot/types.rs`
- TLV codecs の重複確認

**推奨アクション**: 統合可能性調査（別タスク）

---

### 2. TODO/FIXME/HACKの整理

**統計**:
- TODO/FIXME/XXX/HACK: 44箇所（30ファイル）
- unimplemented!/todo!/unreachable!: 8ファイル

**推奨アクション**:
1. 緊急度でトリアージ
2. issueトラッカーへ移行
3. 完了済みコメント削除

**削減**: 直接の行数削減はないが、コード品質向上

---

### 3. 未使用#[allow(dead_code)]属性

**問題**: `src/runtime/type_registry.rs:92`で未使用警告

```rust
#[allow(dead_code)]  // ← この属性自体が未使用
```

**削減**: 1行（微小）

---

### 低優先度 小計: 約100行（見積もり）

---

## 📈 統計サマリー

### 削減可能行数
| カテゴリ | 行数 | 割合 |
|---------|------|------|
| 高優先度削除 | 3,615 | 3.6% |
| 中優先度要確認 | 3,471 | 3.5% |
| 低優先度リファクタリング | 100 | 0.1% |
| **合計** | **7,186** | **7.2%** |

### ファイル数削減
- バックアップファイル: 1個
- BID Codegen実験: 2ディレクトリ（約15ファイル）
- Phase 15.6完了後: src/boxes/ 57ファイル

---

## 🎯 推奨実行計画

### Phase 1: 即座削除可能（影響ゼロ）

1. **バックアップファイル削除** - 327行
   ```bash
   rm src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047
   ```

2. **BID Codegen実験コード削除** - 1,894行
   ```bash
   # READMEを確認してから
   rm -rf src/bid-codegen-from-copilot
   rm -rf src/bid-converter-copilot
   ```

**Phase 1 削減**: 2,221行

---

### Phase 2: Phase 15.6完了待ち（最大削減機会）

**前提条件**: ChatGPT5のPhase 15.6実装完了

1. **src/boxes/削除** - 約3,000行
2. **Legacy VM handlers削除** - 約1,145行
   - calls/legacy/
   - boxes/legacy/

**Phase 2 削減**: 約4,145行

---

### Phase 3: Backend統合計画確認後

**要調査事項**:
- Cranelift JIT計画
- AOT Backend計画
- WASM Backend統合計画

**削減見積もり**: 約400行

---

## 🔍 詳細調査が必要な項目

### 1. exprs_legacy.rs の参照調査

```bash
grep -r "build_expression_impl_legacy" src --include="*.rs"
```

呼び出し箇所を確認し、削除可能性を判断

---

### 2. LLVM Backend 移行状況

**現状**:
- Rust/inkwell版: `src/backend/llvm_legacy/` (deprecated)
- Python/llvmlite版: `src/llvm_py/` (実用レベル)

**調査項目**:
- Python版でのカバレッジ
- Rust版の削除タイミング
- feature flagの整理

---

### 3. Plugin System 安定性確認

**Phase 15.6の成功条件**:
- plugins/での全Box動作確認
- スモークテスト全PASS
- パフォーマンス劣化なし

**Legacy削除の前提**: 上記3条件クリア

---

## 📝 補足情報

### コンパイル警告の要約

```
warning: unused attribute `allow`
  --> src/runtime/type_registry.rs:92:1

warning: unused import: `std::io::Write`
   --> src/runner/dispatch.rs:349:13

warning: unused variable: `box_type`
   --> src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs:419:24

warning: unused variable: `entry_id_u32`
   --> src/runner/mir_json_emit.rs:205:13
```

これらは微小だが、コード品質向上のため修正推奨。

---

### plugins/ディレクトリの状況

**既存プラグイン**: 20個
```
nyash-array-plugin
nyash-console-plugin
nyash-file
nyash-filebox-plugin
nyash-json-plugin
nyash-map-plugin
nyash-math-plugin
nyash-net-plugin
...
```

**legacy/**: READMEのみ（実装なし）

Phase 15.6でsrc/boxes/からの移行が進行中。

---

## 🚀 次のステップ

1. **即座実行**: Phase 1（バックアップ・BID Codegen削除） - 2,221行削減
2. **Phase 15.6監視**: ChatGPT5の進捗確認、完了後にPhase 2実行 - 4,145行削減
3. **Backend計画確認**: Cranelift/AOT/WASM v2の方針決定 - 400行削減

**合計削減見込み**: 約6,766行（6.8%）

---

## ⚠️ 注意事項

1. **段階的削除**: 一度に大量削除せず、テストを挟みながら進める
2. **Git履歴保持**: 削除前にcommitして復元可能にする
3. **ドキュメント更新**: 削除後、関連ドキュメントも更新
4. **スモークテスト**: 各Phase後に必ず実行

---

**調査完了日**: 2025-10-13
**レポート作成者**: Claude Code (Analysis Task)
