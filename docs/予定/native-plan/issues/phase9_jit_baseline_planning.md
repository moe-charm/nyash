# Phase 9: JIT (baseline) planning

## Summary
- baseline JIT の設計と MIR→JIT の変換方針固め。Deopt/Safepoint/Effects を明示し、将来の最適化に耐える骨格を用意する。

## Scope
- 値表現の選定: Tagged/NaN-box vs 型テーブル参照（最小は i64 tagged or enum 型でOK）。
- Safepoint の配置規約: 関数入口・ループ先頭・Call直後（既存の MIR.Safepoint と整合）。
- Deopt テーブル: JIT 最適化時に巻き戻すための SSA マップ（値ID→ロケーション）。
- Effects の扱い: PURE/READS_HEAP/WRITES_HEAP/IO/FFI/PANIC を JIT バリアに伝播。
- コード生成の骨格: MIR → IR（Cranelift 等は未導入でもよく、当面スケルトン/ダミーで可）。

## Tasks
- [ ] 設計ドキュメント（本ファイル）に各項目の選択肢と採用案を明記
- [ ] Deopt/Safepoint/Effects の最小ランタイム表現のドラフト
- [ ] MIR から JIT IR への変換インタフェースの草案（未実装で可）
- [ ] PoC: JIT off（インタプリタ同等）で VM と結果一致するハーネス

## Acceptance Criteria
- 設計ドキュメントに採用方針と根拠が明記されている
- Deopt/Safepoint/Effects の最小表現が固まっている
- PoC ハーネスで VM と一致（JIT off 状態）

## Out of Scope
- 実際の JIT 最適化/レジスタ割付/高度なコード生成
- GC/Weak の本番バリア

## References
- docs/予定/native-plan/README.md（Safepoint/Barrier 項）
- docs/予定/native-plan/copilot_issues.txt（Phase 9）
