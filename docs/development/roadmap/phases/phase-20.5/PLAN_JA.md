# Phase 20.5 — Gate計画（脱Rust大作戦）

状態: 実行中
範囲: 最小、検証可能、Gate方式実行計画

## エグゼクティブサマリー（5行）

- **Goal（目標）**: Rustから脱却し、自己ホスト可能なHakoruneラインをブートストラップする。
- **Strategy（戦略）**: Rustを凍結（v1.1-frozen）し、小さく検証可能なGateで積み上げる。
- **Shape（形状）**: Parser → MIR → (Pure VM Foundations)、HostBridgeが唯一の外部境界。
- **Proof（証明）**: 決定的出力、パリティ、固定点（v1==v2==v3）で検証。
- **Policy（方針）**: 変更は最小限、決定的、テストファースト（前提条件未達→SKIP）。

---

## Gates（順番に通過必須）

### Gate A: Parser v1（Hakorune製）→ Canonical AST JSON
- **DoD**: ~10ケース PASS、非決定性なし、キーソート済み
- **実装CLI**: `--dump-ast-json`（stdout）、`--emit-ast-json <file>`（マクロ前）
- **Smokes（quick-selfhost）**:
  - `parser_ast_json_canonical_vm.sh`, `parser_ast_json_return_vm.sh`
  - `parser_ast_json_unary_vm.sh`, `parser_ast_json_array_literal_vm.sh`
  - `parser_ast_json_map_literal_vm.sh`, `parser_ast_json_empty_array_vm.sh`
  - `parser_ast_json_empty_map_vm.sh`, `parser_ast_json_if_else_vm.sh`
  - `parser_ast_json_emit_file_vm.sh`
- **MirIoBox正規化**: `mirio_canonicalize_vm.sh`（`HAKO_JSON_CANON`でガード）

### Gate B: MIR Builder v1（Hakorune製）→ 最小MIR（16命令）
- **DoD**: 16命令到達可能、ゴールデンJSON一致、キーソート済み

### Gate C: VM Foundations（Pure Hakorune）→ 5命令PoC via HostBridge
- **命令**: const, binop, compare, jump, ret
- **DoD**: シンプルなプログラムがend-to-end実行、測定可能なパフォーマンス

### Gate D: op_eq Migration（NoOperatorGuard + 8型）
- **DoD**: 20ゴールデンテストPASS、Rust-VM比≥70%パフォーマンス

### Gate E: Integration & Docs
- **DoD**: E2E 10ケースPASS、ドキュメント完備、CI minimal green

---

**注**: C Code Generatorトラックは設計参考として保存されているが、主要ではない。Pure Hakorune VMが実行中の計画。

---

## 最小命令セット（Phase 20.5スコープ）

- **値/制御**: const, ret
- **算術**: binop(Add/Sub/Mul/Div/Mod)
- **比較**: Eq/Ne/Lt/Le/Gt/Ge（boolean 0/1）
- **制御**: jump, branch
- **Phi**: 事前計算による降格（ブロックローカル一時変数）

---

## 決定性 & DoD

- **JSON正規化**: キーソート、安定した空白/改行
- **非決定性排除**: タイムスタンプ/PID/乱数/ハッシュ反復順序の分散なし
- **GateごとのDoD**: 上記リスト参照、集約: Parser→MIR→VM PoC end-to-end

---

## テスト計画

- **Level 1（Sanity）**: 4テスト → const/ret/binop/compare
- **Level 2（Coverage）**: 10テスト → if/loops/arrays/strings/最小再帰
- **Level 3（PoC）**: 1テスト → VM 5命令プログラム実行（Pure Hakorune）
- **方針**: 前提条件未達 → SKIP（WARN）、回帰のみFAIL

---

## CI & パッケージング（最小限）

- **CI最小限**: `cargo build --release` + quick-selfhost（SKIP前提でグリーン） + `make release`（manifest）
- **Windowsビルド**: 手動MSVC推奨、MinGW参考
- **配布**: Linux + Windows（MSVC/MinGW）、README_FROZEN_QUICKSTART + リリースノート

---

## リスク & 対策（短文）

- **v1 != v2ドリフト** → 決定的JSON + 正規化emit + ゴールデンテスト
- **パフォーマンス低下** → PoCコスト受容、ホットパスプロファイル、後で自己コンパイル≤30s予算
- **Box不足** → 事前調査済み最小セット、Gateの下でのみ追加

---

## 次のステップ（実行可能）

1. Gate A/Bテストリスト確定、ゴールデンフィクスチャ追加（jsonソート済み）
2. VM PoC（5命令）実装 via HostBridge（`apps/hakorune-vm`）
3. op_eq移行 with NoOperatorGuard + 20ゴールデンテスト
4. SKIPゲートを段階的にPASSへ（quick-selfhost → plugins → integration）
5. MILESTONE/INDEXを週次更新、artifacts/manifestを新鮮に保つ

---

## 詳細: 各Gate

### 🎯 **Gate A: Parser canonical JSON**

**目的**: Hakorune製Parserが決定的なAST JSONを出力

**実装内容**:
```bash
# 新しいCLIフラグ
./hako --dump-ast-json program.hako        # stdout
./hako --emit-ast-json output.json program.hako  # file
```

**DoD（完了定義）**:
- [ ] ~10テストケースPASS
- [ ] 非決定性ゼロ（タイムスタンプ/PID/ハッシュ順序なし）
- [ ] JSONキー辞書順ソート
- [ ] 9個のスモークテスト追加（quick-selfhost）

**スモークテスト一覧**:
1. `parser_ast_json_canonical_vm.sh` - 基本カノニカル化
2. `parser_ast_json_return_vm.sh` - return文
3. `parser_ast_json_unary_vm.sh` - 単項演算子
4. `parser_ast_json_array_literal_vm.sh` - 配列リテラル
5. `parser_ast_json_map_literal_vm.sh` - マップリテラル
6. `parser_ast_json_empty_array_vm.sh` - 空配列
7. `parser_ast_json_empty_map_vm.sh` - 空マップ
8. `parser_ast_json_if_else_vm.sh` - if/else
9. `parser_ast_json_emit_file_vm.sh` - ファイル出力

**MirIoBox統合**:
- `mirio_canonicalize_vm.sh` - MirIoBox.normalize()テスト
- 環境変数ガード: `HAKO_JSON_CANON=1`

---

### 🎯 **Gate B: MIR Builder v1**

**目的**: 最小16命令のMIR JSON生成

**命令セット**:
```
const, ret                    # 値/制御
binop(Add/Sub/Mul/Div/Mod)   # 算術（5種類）
compare(Eq/Ne/Lt/Le/Gt/Ge)   # 比較（6種類）
jump, branch                  # 制御フロー
```

**DoD（完了定義）**:
- [ ] 16命令すべて到達可能
- [ ] ゴールデンJSON一致（Rust MIR vs Hako MIR）
- [ ] キーソート済み
- [ ] 4本のスモークテストPASS:
  - `const+ret` - 定数返却
  - `add+ret` - 加算返却
  - `eq+branch` - 等価比較分岐
  - `lt+branch` - 大小比較分岐

**実装場所**:
- `selfhost/compiler/mir_builder/` - 既存骨格再利用
- 出力: 共通正規化適用（`JsonCanonicalBox`）

---

### 🎯 **Gate C: VM Foundations PoC**

**目的**: 5命令のみでend-to-end実行（HostBridge経由）

**対象命令**:
```
const     # 定数ロード
binop     # 二項演算（Add/Sub/Mul/Div/Mod）
compare   # 比較（Eq/Ne/Lt/Le/Gt/Ge）
jump      # 無条件ジャンプ
ret       # 返却
```

**DoD（完了定義）**:
- [ ] 5命令がHostBridge経由で実行可能
- [ ] 2本のスモークテストPASS:
  - `const+ret` - 定数返却
  - `compare+branch相当` - if条件分岐
- [ ] シンプルなプログラムがend-to-end実行
- [ ] 測定可能なパフォーマンス

**HostBridge設計**:
```
HostBridge.extern_invoke() 最小パス
  ↓
ExternAdapter/HostBridge活用
  ↓
VM 5命令実行
  ↓
Result返却
```

**実装メモ**: `phase-20.5/PLAN.md`に「PoCの入口・戻り値・制限」を明記

---

### 🎯 **Gate D: op_eq Migration**

**目的**: 等価性ロジックをRust→Hakoruneへ移行（下準備）

**仕様整理**:
- MIRでEq/Neの意味論確定
- Box同士は呼び出し正規化
- ドキュメントに原則・禁止パターン・Verifier観点を追記（短文）

**最小ゴールデン（5-8ケース）**:
- プリミティブ（i64/string/bool）
- BoxRef（pointer equalityのみ）を固定

**DoD（完了定義）**:
- [ ] 仕様ドキュメント完成
- [ ] 5-8ゴールデンケース作成
- [ ] quick-selfhostでPASS

---

### 🎯 **Gate E: Integration & Docs**

**目的**: 全体統合、ドキュメント整備

**DoD（完了定義）**:
- [ ] E2E 10ケースPASS
- [ ] ドキュメント完備（日本語+英語）
- [ ] CI minimal green
- [ ] Phase 20.5完了報告書作成

---

## 🔄 実装フロー

```
Week 1-2: Gate A Polish + Gate B実装
  ├─ Parser canonical JSON仕上げ（10ケース）
  ├─ MirIoBox.normalize()統合
  └─ MIR Builder v1実装（16命令）

Week 3-4: Gate C実装 + Full VM統合準備
  ├─ VM Foundations PoC（5命令）
  ├─ HostBridge設計・実装
  └─ Full Hakorune VM（selfhost/hakorune-vm/）調査

Week 5: Full VM統合
  ├─ 5命令PoC → 22命令Full VM拡張
  ├─ selfhost/hakorune-vm/ 統合
  └─ HostBridge経由実行確認

Week 6: Gate D/E
  ├─ op_eq Migration（下準備）
  ├─ Golden Testing
  └─ Integration & Docs

Week 7-8（オプション）: Polish & CI
  ├─ CI minimal green
  └─ リリース準備
```

---

## 📊 進捗追跡

### **Gate A: Parser canonical JSON**（実装中）

- [x] CLI実装（`--dump-ast-json`, `--emit-ast-json`）
- [x] 9個のスモークテスト追加
- [x] MirIoBox.normalize()統合
- [ ] ~10ケース全PASS
- [ ] ドキュメント追記（docs/guides/env-variables.md）

### **Gate B: MIR Builder v1**（次）

- [ ] 最小16命令仕様明記（docs/reference/ir/mir-json-v0.md）
- [ ] 最小ビルダー実装（selfhost側）
- [ ] 4本スモークPASS

### **Gate C: VM Foundations PoC**（その次）

- [ ] HostBridge設計
- [ ] 5命令実装 via HostBridge
- [ ] 2本スモークPASS

### **Gate D: op_eq Migration**（その次）

- [ ] 仕様整理
- [ ] 5-8ゴールデンケース

### **Gate E: Integration & Docs**（最後）

- [ ] E2E 10ケース
- [ ] ドキュメント完備

---

## 🎉 成功の定義

Phase 20.5完了時:

1. ✅ **Parser canonical JSON動作**（Gate A）
2. ✅ **MIR Builder v1動作**（Gate B）
3. ✅ **VM PoC動作**（Gate C: 5命令）
4. ✅ **Full Hakorune VM統合**（Gate C+: 22命令）
5. ✅ **op_eq下準備完了**（Gate D）
6. ✅ **E2E統合テスト**（Gate E）
7. ✅ **ドキュメント完備**（日本語+英語）
8. ✅ **CI minimal green**

次のPhase（Phase 20.6）への準備完了！

---

**作成日**: 2025-10-14
**最終更新**: 2025-10-14（Gate A実装中）
**状態**: Gate A Polish中
**次回マイルストーン**: Gate B完了（Week 2終了時）
