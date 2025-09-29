# Phase 10: JIT実装とセルフホスティング

## 🎯 Phase 10の全体像

Phase 10は、Nyashの実行性能を大幅に向上させるJIT実装と、言語の成熟度を示すセルフホスティングを実現します。

## 📊 実装優先順位

### 1️⃣ **メイン実装: Cranelift JIT** 
→ [phase_10_cranelift_jit_backend.md](phase_10_cranelift_jit_backend.md)
- VMとのハイブリッド実行（ホットパス検出→JIT化）
- 実装期間: 2-3ヶ月
- 目標: ホットパスで2倍以上の高速化

### 🌟 **革新的機能: GC切り替え可能ランタイム**
→ [phase_10_4_gc_switchable_runtime.md](phase_10_4_gc_switchable_runtime.md)
- 世界初：実行時にGCモード切り替え可能
- 開発時はGCオンで快適、本番はGCオフで高速
- 実装期間: 2-3ヶ月（Cranelift JIT後）
- 技術的にCodex GPT-5が実現可能性を確認済み

### 2️⃣ **並行プロジェクト: セルフホスティング**
→ [phase_10_5_core_std_nyash_impl.md](phase_10_5_core_std_nyash_impl.md)
- String/Array/MapをNyash自身で実装
- Rust依存の段階的削減
- 実装期間: 1-2ヶ月

### 3️⃣ **実戦テスト: アプリケーション移植**
→ [phase_10_app_migration.md](phase_10_app_migration.md)
- Tinyproxy: ゼロコピー判定機能の検証
- Chip-8エミュレータ: fini伝播とweak参照の実戦テスト
- kiloエディタ: メモリ効率の「うっかり全体コピー」検出

### 🚫 **延期プロジェクト**
→ [Phase 11: LLVM AOT Backend](../phase-11/) - 将来の研究開発として分離

## 🛤️ 実装ロードマップ

```
Phase 9.79b (現在)
    ↓
Phase 10.0: Cranelift JIT基盤構築
    ├→ Phase 10.1-10.3: JIT実装・最適化
    ├→ Phase 10.4: GC切り替え可能ランタイム ← NEW!
    └→ Phase 10.5: セルフホスティング（並行）
    ↓
Phase 10.9: アプリケーション移植で実戦検証
    ↓
Phase 11: LLVM AOT研究（将来）
```

## 📈 期待される成果

1. **実行性能**: インタープリタ比100倍、VM比2-3倍の高速化
2. **言語成熟度**: 基本コンテナのセルフホスティング達成
3. **実用性検証**: 実アプリケーションの移植による実戦テスト

## 🔗 関連ドキュメント
- [00_MASTER_ROADMAP.md](../00_MASTER_ROADMAP.md) - 全体計画
- [Phase 9.79b](../phase-9/) - 統一Box設計（前提）
- [MIR仕様](../../../../reference/mir/) - 中間表現