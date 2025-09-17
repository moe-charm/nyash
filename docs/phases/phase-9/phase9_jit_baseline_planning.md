# ⚠️ Phase 9: JIT (baseline) planning → 📦 実用優先戦略により変更

## 🔄 戦略変更通知（2025-08-14）

**この Phase 9 は実用優先戦略により以下に変更されました：**

### 🚀 **新 Phase 9: AOT WASM実装**
- **実装内容**: wasmtime compileによるAOT実行ファイル生成
- **期間**: 2-3週間  
- **詳細**: [phase9_aot_wasm_implementation.md](phase9_aot_wasm_implementation.md)

### 🔄 **JIT実装の新位置づけ**
- **Phase 12以降**: 将来オプション機能
- **理由**: Rust開発環境改善効果限定的、実用価値より最適化重視

---

## 📋 以下は従来計画（参考保存）

### Summary
- baseline JIT の設計と MIR→JIT の変換方針固め。Deopt/Safepoint/Effects を明示し、将来の最適化に耐える骨格を用意する。

### Scope
- 値表現の選定: Tagged/NaN-box vs 型テーブル参照（最小は i64 tagged or enum 型でOK）。
- Safepoint の配置規約: 関数入口・ループ先頭・Call直後（既存の MIR.Safepoint と整合）。
- Deopt テーブル: JIT 最適化時に巻き戻すための SSA マップ（値ID→ロケーション）。
- Effects の扱い: PURE/READS_HEAP/WRITES_HEAP/IO/FFI/PANIC を JIT バリアに伝播。
- コード生成の骨格: MIR → IR（Cranelift 等は未導入でもよく、当面スケルトン/ダミーで可）。

### References
- docs/予定/native-plan/copilot_issues.txt（実用優先戦略決定）
- docs/予定/ai_conference_native_compilation_20250814.md（AI大会議結果）
