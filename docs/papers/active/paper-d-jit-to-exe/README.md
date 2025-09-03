# Paper D: From JIT to Native - A Unified Compilation Pipeline for Box-based Languages

## 📋 概要

NyashのJIT実行からネイティブEXE生成までの統一コンパイルパイプラインに関する論文。
MIR13という極小IRからCranelift経由でネイティブバイナリを生成する革新的アプローチ。

## 🎯 論文の新規性

### 1. **極小IR（MIR13）による統一実行**
- たった13命令で全機能実現
- JIT/インタープリター/AOT/WASMすべてに対応
- Box哲学による統一的メモリモデル

### 2. **Cranelift + lld内蔵戦略**
- 外部コンパイラ依存の完全排除
- JITコンパイル結果をそのままEXE化
- プラットフォーム非依存の美しい設計

### 3. **C ABIファサード**
```c
// 最小限の美しいインターフェース
ny_mir_to_obj(mir_bin, target_triple) -> obj_bytes
ny_mir_jit_entry(mir_bin) -> exit_code
ny_free_buf(buffer)
```

### 4. **パフォーマンス最適化**
- ホットパス検出→選択的JIT
- JIT結果のキャッシュ→AOT変換
- TypedArray bounds-check併合

## 📊 評価計画

### ベンチマーク対象
1. **コンパイル時間**: MIR→EXE
2. **実行性能**: JIT vs AOT
3. **バイナリサイズ**: 最小実行ファイル
4. **起動時間**: JIT warmup vs AOT instant

### 比較対象
- Go（単一バイナリ生成）
- Rust（ネイティブコンパイル）
- Node.js（JIT実行）
- GraalVM（JIT→AOT）

## 🔬 技術的詳細

### パイプライン構成
```
AST → MIR13 → Cranelift IR → Machine Code → Object File → EXE
     ↓           ↓              ↓
  Interpreter    JIT          Direct Execute    
```

### 実装のポイント
1. **MIR最適化パス**
   - デッドコード除去
   - 定数畳み込み
   - インライン展開

2. **Cranelift統合**
   - Module構築
   - Function定義
   - コード生成

3. **リンカー統合**
   - lld-link（Windows）
   - ld.lld（Linux）
   - nyashrtランタイム

## 📅 執筆スケジュール

### Phase 1: 実装（2-3週間）
- [ ] JIT→Object生成実装
- [ ] lld統合
- [ ] 基本的なEXE生成

### Phase 2: 評価（1-2週間）
- [ ] ベンチマーク実装
- [ ] 性能測定
- [ ] 結果分析

### Phase 3: 執筆（2週間）
- [ ] Abstract作成
- [ ] 各章執筆
- [ ] 図表作成

## 🎯 投稿先候補

### トップ会議
- **PLDI 2026**: プログラミング言語設計の最高峰
- **CC 2026**: コンパイラ構築専門
- **CGO 2026**: コード生成最適化

### ジャーナル
- **TOPLAS**: ACM最高峰ジャーナル
- **Software: Practice and Experience**: 実装重視

## 💡 期待されるインパクト

1. **学術的**: 極小IRによる統一実行モデルの提案
2. **実用的**: 外部依存ゼロのコンパイラ実現
3. **教育的**: シンプルで理解しやすい実装
4. **産業的**: 組み込みシステムへの応用可能性

## 🔗 関連資料

- [Phase 10: JIT実装](../../development/roadmap/phases/phase-10/)
- [Phase 15: 自己ホスティング](../../development/roadmap/phases/phase-15/)
- [MIR13仕様書](../../reference/mir/INSTRUCTION_SET.md)
- [Cranelift統合設計](../../development/roadmap/phases/phase-15/self-hosting-lld-strategy.md)