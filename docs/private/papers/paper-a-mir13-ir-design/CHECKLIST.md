# 論文A チェックリスト（MIR14/最小IR設計）

## スコープ確定
- [ ] MIR13 → MIR14 の用語整合（本文・図・ファイル名注記）
- [ ] 命令セットの最終リストを固定（参照リンクで一元化）
- [ ] BoxCall/externcall/call の用語・図の統一

## 実験と再現性
- [ ] PyVM ↔ llvmlite パリティ: `tools/parity.sh` の代表ケース通過
- [ ] 代表スモーク: `tools/pyvm_stage2_smoke.sh`, `tools/llvm_smoke.sh` 結果採録
- [ ] 性能測定: Interpreter/VM/JIT/AOT の速度・起動時間・メモリ
- [ ] GUI 応答性（<16ms）データ取得（代表操作）

## LLVM Harness / PHI 不変条件
- [ ] PHI はブロック先頭に集約される説明と根拠
- [ ] incoming は型付き `i64 <v>, %bb` を例示
- [ ] 空PHI防止の最終化手順（finalize_phis）説明

## 図表
- [ ] 命令縮約の年表（27→13→14）
- [ ] BoxCall 呼出し経路（ABI 境界含む）
- [ ] PHI 配線模式図

## 原稿
- [ ] Abstract（JP/EN）
- [ ] 本文（JP/EN）: main‑paper‑jp.md / main‑paper.md
- [ ] 関連研究と差分の明確化

## 生成物
- [ ] `tools/papers/build.sh a-jp` / `a-en` 成功
- [ ] `docs/private/out/paper-a-*.pdf` 出力を確認
- [ ] `paper-a-mir13-ir-design/out/` にも最終版を複製 or リンク（運用方針をREADMEに明記）

## 提出準備
- [ ] arXiv用体裁チェック（図の解像度/フォント）
- [ ] 参考文献整備（BibTex or 手動）
- [ ] ライセンス/付録の整合

---

メモ: 仕様の一次ソースは `docs/reference/` を規範にし、重複ドキュメントはリンクで参照する（複製管理を避ける）。
