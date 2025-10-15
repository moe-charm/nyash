# Phase 4 クイックスタート

**最終更新**: 2025-10-16
**想定読者**: Phase 4 実装担当者（Claude/ChatGPT/開発者）

---

## 🎯 Phase 4 の目標（3行要約）

1. **Rust Parser** と **Hakorune Selfhost Parser** の両方で同じスモークテストを実行
2. **JSON v0 ヘッダ** (`{"version":"0","kind":"Program","stats":{"stmts":N}}`) を比較
3. **`SMOKES_PARSER_MODE=both`** で Phase-A スモーク（4個）が緑 ✅

---

## 📊 現状（既に完成している部分）

✅ **Rust Parser**: 完成（227行、リファクタリング済み）
✅ **Hakorune Selfhost Parser**: M2/M3達成（170/185 PASS）
✅ **Facade構造**: `src/front/parser_layer/facade.rs` で実装済み
✅ **JSON v0ヘッダ生成**: Hakorune側で実装済み（`header_emit_box.hako`）
✅ **スモークテストインフラ**: v2システム完成
✅ **セミコロン寛容モード**: `NYASH_PARSER_ALLOW_SEMICOLON=1` で既に動作

---

## 🚀 実装すべきこと（3つだけ）

### 1. Rust Parser側: JSON v0ヘッダ生成機能
```rust
// src/front/parser_layer/facade.rs (追加)
pub fn emit_json_v0_header(ast: &crate::ast::ASTNode) -> String {
    use crate::ast::ASTNode;
    let stmt_count = match ast {
        ASTNode::Program { statements, .. } => statements.len(),
        _ => 0,
    };
    format!(r#"{{"version":"0","kind":"Program","stats":{{"stmts":{}}}}}"#, stmt_count)
}
```

**工数**: 30分

---

### 2. CLI: --emit-json-v0-header フラグ
```rust
// src/cli/args.rs (追加)
#[arg(long, help = "Emit JSON v0 header and exit")]
pub emit_json_v0_header: bool,
```

```rust
// src/runner/mod.rs (追加)
if args.emit_json_v0_header {
    let source = std::fs::read_to_string(&args.input)?;
    let ast = crate::front::parser_layer::facade::parse_source_to_ast(&source)?;
    let header = crate::front::parser_layer::facade::emit_json_v0_header(&ast);
    println!("{}", header);
    return Ok(());
}
```

**工数**: 1時間

---

### 3. テストハーネス: SMOKES_PARSER_MODE=both
```bash
# tools/smokes/v2/lib/test_runner.sh (追加)

compare_json_v0_headers() {
    local rust_json="$1"
    local hako_json="$2"

    local rust_version=$(echo "$rust_json" | jq -r '.version')
    local rust_kind=$(echo "$rust_json" | jq -r '.kind')
    local rust_stmts=$(echo "$rust_json" | jq -r '.stats.stmts')

    local hako_version=$(echo "$hako_json" | jq -r '.version')
    local hako_kind=$(echo "$hako_json" | jq -r '.kind')
    local hako_stmts=$(echo "$hako_json" | jq -r '.stats.stmts')

    [ "$rust_version" = "$hako_version" ] || return 1
    [ "$rust_kind" = "$hako_kind" ] || return 1
    [ "$rust_stmts" = "$hako_stmts" ] || return 1
    return 0
}

_run_nyash_vm_both() {
    local program="$1"
    shift

    local rust_header=$("$NYASH_BIN" --emit-json-v0-header "$program" 2>&1 | filter_noise)
    [ $? -eq 0 ] || { log_error "Rust Parser failed"; return 1; }

    # 暫定: Hakorune Selfhost Parserは後で統合
    local hako_header=$("$NYASH_BIN" --emit-json-v0-header "$program" 2>&1 | filter_noise)
    [ $? -eq 0 ] || { log_error "Hakorune Selfhost Parser failed"; return 1; }

    compare_json_v0_headers "$rust_header" "$hako_header"
}
```

**工数**: 2時間

---

## 📅 2日間スケジュール（MVP）

### Day 1: 基盤実装
```
✅ emit_json_v0_header() 実装（30分）
✅ --emit-json-v0-header フラグ追加（1時間）
✅ 単体テスト（30分）
✅ compare_json_v0_headers() 実装（30分）
✅ _run_nyash_vm_both() 実装（1.5時間）
✅ 動作確認（1時間）

合計: 5.5時間
```

### Day 2: スモーク作成 + 最終確認
```
✅ Phase-Aスモーク4個作成（2時間）
✅ 動作確認（1時間）
✅ quick/quick-selfhost スモーク実行（30分）
✅ ドキュメント更新（1.5時間）

合計: 5時間
```

---

## ✅ 受け入れ基準

```bash
# Phase-A スモーク（4個）
SMOKES_PARSER_MODE=both bash tools/smokes/v2/run.sh --profile phase-a
→ 4/4 PASS ⭐

# 既存機能の維持
bash tools/smokes/v2/run.sh --profile quick
→ 既存PASS維持

SMOKES_SELFHOST_ENABLE=1 bash tools/smokes/v2/run.sh --profile quick-selfhost
→ 170/185 PASS 維持
```

---

## 📖 Phase-A スモークセット（4個）

1. **セミコロン受理**: `phase-a/semicolon_accept_vm.sh`
2. **if-else**: `phase-a/if_else_vm.sh`（既存流用: `selfhost/hakorune_pipeline_compare_branch_phi_vm.sh`）
3. **ブロック終端**: `phase-a/block_terminator_vm.sh`（既存流用: `selfhost/hakorune_pipeline_const_ret_vm.sh`）
4. **using最小**: `phase-a/using_minimal_vm.sh`（既存流用: `selfhost/selfhost_pipeline_v2_call_exec_vm.sh`）

---

## 🔍 デバッグコマンド

```bash
# Rust Parser単体テスト
./target/release/hakorune --emit-json-v0-header apps/tests/hello.hako
# 期待: {"version":"0","kind":"Program","stats":{"stmts":1}}

# 比較モードテスト
SMOKES_PARSER_MODE=both bash tools/smokes/v2/profiles/quick/selfhost/selfhost_min_json_header_vm.sh

# 既存スモーク（既定モード）
bash tools/smokes/v2/run.sh --profile quick
```

---

## 📚 詳細ドキュメント

**完全な技術要件分析**: [`phase4_dual_parser_harness_technical_requirements.md`](../../analysis/phase4_dual_parser_harness_technical_requirements.md)

**セクション**:
1. 現状分析（Rust Parser/Hakorune Selfhost Parser）
2. Phase 4 技術要件（`SMOKES_PARSER_MODE`/JSON v0ヘッダ仕様）
3. 境界設計（呼び出しフロー）
4. 受け入れ基準の具体化
5. 技術的課題（4つ）
6. 推奨アプローチ（段階的実装戦略）
7. 実装タイムライン（3日間詳細スケジュール）

---

## ⚠️ 重要な制約

**Phase 4の範囲**:
- ✅ JSON v0ヘッダ比較のみ（`version/kind/stats.stmts`）
- ❌ 完全な実行比較は Phase 5 以降

**除外（将来実装）**:
- `--selfhost-parser` フラグ
- `ast_to_stage1_json()` 完全実装
- Hakorune Selfhost Parserの完全統合

---

## 🎯 次のアクション（今すぐ実行可能）

```bash
cd /home/tomoaki/git/hakorune-selfhost

# 1. Rust側実装
vim src/front/parser_layer/facade.rs  # emit_json_v0_header() 追加
vim src/cli/args.rs                   # --emit-json-v0-header フラグ追加
vim src/runner/mod.rs                 # フラグ処理ロジック追加

# 2. ビルド
cargo build --release

# 3. 動作確認
./target/release/hakorune --emit-json-v0-header apps/tests/hello.hako

# 4. テストハーネス実装
vim tools/smokes/v2/lib/test_runner.sh  # compare_json_v0_headers() 追加

# 5. スモーク作成
mkdir -p tools/smokes/v2/profiles/phase-a
# 4つのスモークスクリプトを作成

# 6. 実行
bash tools/smokes/v2/run.sh --profile phase-a
```

---

**作成者**: Claude (Technical Analysis Agent)
**最終更新**: 2025-10-16
