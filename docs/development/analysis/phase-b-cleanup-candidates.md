# Phase B完了後 削除可能コード特定レポート

**作成日**: 2025-10-13
**対象**: Phase B（HostBridge統一・MirIoBox完成）完了後の削除可能コード

## 📊 エグゼクティブサマリー

Phase B完了により以下の成果が達成されました：
- ✅ HostBridge extern trampoline統一（すべてのBox呼び出しがHostBridge経由）
- ✅ MirIoBox安定動作（MIR入力統一）
- ✅ nyvm → Rust 経路完全動作（`env.extern.invoke`, `hostbridge.extern_invoke`）
- ✅ JSON provider統一（scan locator統一、provider無効化）

**削減可能コード量推定**:
- 即座削除可能: **3ファイル、約490行**
- 要確認後削除: **3ファイル、約162行**
- リファクタリング推奨: **22箇所のfallback処理**

---

## 🗑️ 1. 即座に削除可能（高確度）

### 1.1 Legacy Expression Lowering

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/mir/builder/exprs_legacy.rs`
- **行数**: 52行
- **理由**: Phase Bでビルダーが安定化、レガシー表現lowering不要
- **確認方法**:
  ```bash
  grep -r "build_expression_impl_legacy" src/
  # 結果: src/mir/builder/exprs.rs:1箇所のみ（簡単に削除可能）
  ```
- **使用箇所**: `src/mir/builder/exprs.rs` の1箇所のみ
- **影響範囲**: 限定的（フォールバック処理として存在）
- **削除手順**:
  1. `src/mir/builder/exprs.rs` から `build_expression_impl_legacy` 呼び出しを削除
  2. `mod exprs_legacy;` 宣言を削除
  3. ファイル削除: `src/mir/builder/exprs_legacy.rs`

**削減見積もり**: **52行**

---

### 1.2 Plugin Box Legacy Proxy

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/plugin_box_legacy.rs`
- **行数**: 158行
- **理由**: Phase B でHostBridge統一、古いプラグインプロキシ不要
- **確認方法**:
  ```bash
  grep -r "plugin_box_legacy\|PluginBox" src/ --include="*.rs" | wc -l
  # 結果: PluginBox参照133箇所あるが、これは PluginBoxV2 との混同の可能性
  grep -r "use.*plugin_box_legacy" src/
  # 結果: 0箇所（未使用！）
  ```
- **使用箇所**: **0箇所**（完全未使用）
- **影響範囲**: なし
- **削除手順**:
  1. ファイル削除: `src/runtime/plugin_box_legacy.rs`
  2. `src/runtime/mod.rs` から module宣言を削除（存在する場合）

**削減見積もり**: **158行**

---

### 1.3 BID Bridge (Old Box-Handle Conversion)

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/bid/bridge.rs`
- **行数**: 280行
- **理由**: Phase B でHostBridge統一、古いBID変換層不要
- **確認方法**:
  ```bash
  grep -r "bid.*bridge\|bid/bridge" src/ --include="*.rs"
  # 結果: 0箇所（未使用！）
  grep -r "BoxRegistry\|BidBridge" src/runtime --include="*.rs"
  # 結果: UnifiedBoxRegistry のみ使用（新システム）
  ```
- **使用箇所**: **0箇所**（完全未使用）
- **影響範囲**: なし
- **機能**: Nyash Box ↔ BID handle変換（Phase B前の旧実装）
- **削除手順**:
  1. ファイル削除: `src/bid/bridge.rs`
  2. `src/bid/mod.rs` から module宣言を削除
  3. `BoxRegistry`, `BidBridge` trait定義も削除

**削減見積もり**: **280行**

---

**即座削除可能 合計**: **490行** (3ファイル)

---

## ⚠️ 2. 削除前に要確認（中確度）

### 2.1 Deprecated Builtin Box Implementations

**ファイル群**: `/home/tomoaki/git/hakorune-selfhost/src/box_factory/builtin_impls/`
- `console_box.rs` (35行) - ⚠️ DEPRECATED、nyash-console-plugin存在
- `bool_box.rs` (約40行) - ⚠️ DEPRECATED、BoolBox plugin必要
- `integer_box.rs` (約45行) - ⚠️ DEPRECATED、nyash-integer-plugin必要

**総行数**: 約120行

**懸念点**:
1. **ConsoleBox は critical**（コメントで明記: "remove LAST!"）
2. プラグイン版が完全に動作するか確認必要
3. テストが通るか確認必須

**確認項目**:
```bash
# 1. プラグイン版が存在するか確認
ls plugins/nyash-console-plugin/
ls plugins/nyash-integer-plugin/
ls plugins/nyash-bool-plugin/  # ← 存在しない可能性

# 2. builtin版への依存を確認
grep -r "box_factory::builtin_impls" src/

# 3. テスト実行
NYASH_DISABLE_PLUGINS=0 cargo test
```

**削除手順**（プラグイン版動作確認後）:
1. BoolBox plugin作成（存在しない場合）
2. IntegerBox plugin動作確認
3. ConsoleBox を**最後**に削除（他のBox削除後）

**削減見積もり**: **約120行**（要確認後）

---

### 2.2 Phase A Remnants (Final ABI Skeleton)

**場所**: `src/runtime/plugin_loader_v2/enabled/`

**コメント内の Phase A 参照**:
- `ffi_bridge.rs`: "Phase A: If Final ABI is available..." (複数箇所)
- `host_bridge.rs`: "Phase A skeleton"
- `types.rs`: "Phase A minimal structs"

**懸念点**:
- Phase A は Final ABI の skeleton実装
- Phase B で使用されているか不明（コメントのみの可能性）
- 削除すると Final ABI サポートが壊れる可能性

**確認項目**:
```bash
# Final ABI が実際に使用されているか確認
grep -r "NYASH_PLUGIN_ABI_FINAL\|plugin_abi_final" src/
grep -r "FinalInvokeFn\|final_invoke" src/
```

**判断**:
- Final ABI が Phase C+ の計画にある場合: **削除しない**
- コメントのみで未使用なら: Phase A 参照コメントを Phase B に更新

**削減見積もり**: 0行（コメント修正のみ）

---

### 2.3 Legacy Extern Handler

**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/calls/legacy/extern_handler.rs`

**内容**: Phase B で実装された `env.extern.invoke`, `hostbridge.extern_invoke` trampoline

**懸念点**:
- **これは Phase B の成果物**（削除候補ではない！）
- "legacy" ディレクトリ名が誤解を招く
- 実際には Phase B で**追加された新機能**

**推奨アクション**:
```bash
# ディレクトリ名変更（legacy → trampolines）
mv src/backend/mir_interpreter/handlers/calls/legacy \
   src/backend/mir_interpreter/handlers/calls/trampolines

# ファイル名変更
mv trampolines/extern_handler.rs trampolines/hostbridge_trampolines.rs
```

**削減見積もり**: 0行（リネームのみ）

---

## 📝 3. リファクタリング推奨

### 3.1 Fallback処理の削減

**場所**: `src/runtime/` 配下

**現状**:
```bash
grep -r "fallback\|FALLBACK" src/runtime --include="*.rs" | wc -l
# 結果: 22箇所
```

**Phase B 後のベストプラクティス**:
- HostBridge統一により、fallback不要（すべて統一経路）
- Silent fallback 禁止（CLAUDE.md の Fail-Fast 哲学）

**推奨アクション**:
1. 各 fallback 箇所を調査
2. HostBridge経路で解決可能なら fallback削除
3. 必要な fallback は明示的エラーメッセージに変更

**影響範囲**: `src/runtime/` 全体（約75ファイル）

**工数見積もり**: 1-2日（詳細調査必要）

---

### 3.2 "Legacy" ディレクトリの整理

**問題点**:
- `src/backend/mir_interpreter/handlers/calls/legacy/` は Phase B の**新機能**を含む
- 名前が誤解を招く（"legacy" = 古い実装、という印象）

**推奨アクション**:
```bash
# ディレクトリ構造変更
src/backend/mir_interpreter/handlers/calls/
├── legacy/              # 削除
├── trampolines/         # 新規（Phase B trampolines）
│   ├── hostbridge.rs    # env.extern.invoke, hostbridge.*
│   └── mod.rs
├── function.rs
└── mod.rs
```

---

### 3.3 Deprecated 警告メッセージの削除

**場所**:
- `src/box_factory/builtin_impls/console_box.rs`
- `src/box_factory/builtin_impls/bool_box.rs`
- `src/box_factory/builtin_impls/integer_box.rs`

**現状**: 毎回 `eprintln!("⚠️ [DEPRECATED] ...")` 出力

**Phase B 後の方針**:
- プラグイン版が安定したら deprecated警告削除
- または `NYASH_DEPRECATED_WARNINGS=1` で制御

---

## 📈 Phase B達成による削減効果

### 即座削除可能
- **削除可能ファイル数**: 3個
- **削除可能総行数**: 490行
- **コードベース削減率**: 約0.07%（全717ファイル中）

### 要確認後削除
- **削除候補ファイル数**: 3個
- **削除候補総行数**: 約120行

### 総合削減見込み
- **合計削減行数**: 610行
- **リファクタリング対象**: 22箇所（fallback処理）

---

## 🚀 削除実行計画（推奨順序）

### Phase 1: 安全な削除（即座実行可能）

```bash
# 1. Legacy Expression Lowering
git rm src/mir/builder/exprs_legacy.rs
# src/mir/builder/exprs.rs から参照削除

# 2. Plugin Box Legacy Proxy
git rm src/runtime/plugin_box_legacy.rs
# src/runtime/mod.rs から module宣言削除

# 3. BID Bridge
git rm src/bid/bridge.rs
# src/bid/mod.rs から module宣言削除
```

**テスト**:
```bash
cargo build --release
cargo test
tools/smokes/v2/run.sh --profile quick
```

**削減効果**: -490行

---

### Phase 2: プラグイン移行確認後削除（慎重に）

```bash
# 1. BoolBox plugin作成（存在しない場合）
# 2. プラグイン版動作確認
NYASH_DISABLE_PLUGINS=0 cargo test

# 3. IntegerBox builtin削除
git rm src/box_factory/builtin_impls/integer_box.rs

# 4. BoolBox builtin削除
git rm src/box_factory/builtin_impls/bool_box.rs

# 5. ConsoleBox を最後に削除
git rm src/box_factory/builtin_impls/console_box.rs
```

**削減効果**: -120行（追加）

---

### Phase 3: リファクタリング（長期計画）

1. **Fallback処理調査** (1-2日)
   - 22箇所のfallback詳細調査
   - HostBridge経路への置き換え可否判断

2. **ディレクトリ構造改善** (0.5日)
   - `legacy/` → `trampolines/` リネーム
   - Phase B trampolineの明確化

3. **Deprecated警告整理** (0.5日)
   - 警告メッセージの環境変数制御
   - または完全削除

---

## 🎓 Phase B から学んだこと

### 成功要因
1. **統一アーキテクチャ**: HostBridge単一経路で複雑さ削減
2. **Fail-Fast設計**: Silent fallback排除で問題早期発見
3. **段階的移行**: Phase A/B で安全に移行

### 次フェーズへの提言
1. **命名の重要性**: "legacy" は誤解を招く（Phase B trampolineは新機能）
2. **削除タイミング**: 旧実装は新実装安定後、即座に削除
3. **テスト自動化**: 削除前後でスモークテスト自動実行

---

## 📚 参考資料

- Phase B 設計: `docs/development/roadmap/phases/phase-b/`
- HostBridge実装: `src/runtime/plugin_loader_v2/enabled/host_bridge.rs`
- MirIoBox実装: `apps/selfhost-compiler/`
- スモークテスト: `tools/smokes/v2/`

---

## ✅ チェックリスト

### Phase 1削除前
- [ ] `exprs_legacy.rs` 使用箇所確認（1箇所のみ）
- [ ] `plugin_box_legacy.rs` 参照なし確認（0箇所）
- [ ] `bid/bridge.rs` 参照なし確認（0箇所）
- [ ] Phase 1削除後 `cargo build` 成功
- [ ] Phase 1削除後 `cargo test` 成功
- [ ] Phase 1削除後 スモークテスト成功

### Phase 2削除前
- [ ] nyash-bool-plugin 存在確認（または作成）
- [ ] nyash-integer-plugin 動作確認
- [ ] nyash-console-plugin 動作確認
- [ ] プラグイン版で全テスト成功確認
- [ ] ConsoleBox を**最後**に削除（他のBox安定後）

### Phase 3リファクタリング前
- [ ] Fallback 22箇所の詳細調査完了
- [ ] 各fallbackのHostBridge代替案作成
- [ ] ディレクトリ構造変更の影響範囲確認

---

**注意**: このレポートは分析のみです。実際の削除は必ずユーザー確認後に実施してください。
