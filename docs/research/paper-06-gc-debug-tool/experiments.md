# 実験計画 / Experiment Plan

## 🎯 実験の目的

「Debug-Only GC」アプローチの有効性を定量的に評価し、以下を実証する：

1. **開発効率**: GC有効時の開発速度とバグ発見率
2. **品質保証**: リーク検出の精度と修正効率
3. **性能特性**: GC無効時の実行性能とメモリ効率
4. **意味論的等価性**: GCオン/オフでの動作の同一性

## 🔬 実験1: 開発効率の定量化

### 実験設定
- **被験者**: 20名（初級10名、上級10名）
- **タスク**: 3種類のプログラム実装
  - P2Pチャットアプリケーション
  - 簡易データベースエンジン
  - ゲームエンジン（物理演算含む）
- **比較対象**: 
  - Nyash (GC有効)
  - Rust (手動メモリ管理)
  - Go (常時GC)

### 測定項目
```
1. 実装完了時間（分）
2. コンパイルエラー回数
3. 実行時エラー回数
4. メモリリーク発生数
5. 主観的難易度（5段階評価）
```

### 予想結果
- Nyash ≈ Go < Rust（実装時間）
- Nyash < Go < Rust（メモリリーク数）

## 🔬 実験2: リーク検出精度

### 実験設定
- **テストケース**: 100個の既知リークパターン
  - 単純な参照忘れ（30個）
  - 複雑な循環参照（30個）
  - 非同期処理でのリーク（20個）
  - プラグイン境界でのリーク（20個）

### 測定項目
```rust
struct DetectionMetrics {
    true_positive: u32,   // 正しく検出
    false_positive: u32,  // 誤検出
    false_negative: u32,  // 見逃し
    detection_time: f64,  // 検出時間（秒）
    fix_suggestion_quality: f32, // 修正提案の質（0-1）
}
```

### 評価基準
- 検出率（Recall）: TP / (TP + FN) > 95%
- 精度（Precision）: TP / (TP + FP) > 90%

## 🔬 実験3: 性能インパクト測定

### ベンチマークスイート
1. **マイクロベンチマーク**
   - Box allocation/deallocation
   - Method dispatch
   - Field access
   - Collection operations

2. **実アプリケーション**
   - Webサーバー（リクエスト処理）
   - ゲームループ（60FPS維持）
   - データ処理（バッチ処理）

### 測定構成
```nyash
// 3つの構成で同じコードを実行
CONFIG_1: GC有効（開発モード）
CONFIG_2: GC無効（本番モード）
CONFIG_3: Rustで再実装（比較用）
```

### 期待される結果
```
性能比（CONFIG_2 / CONFIG_1）:
- スループット: 1.5-2.0倍
- レイテンシ: 0.5-0.7倍
- メモリ使用量: 0.8-0.9倍

CONFIG_2 vs CONFIG_3（Rust）:
- 性能差: ±5%以内
```

## 🔬 実験4: 意味論的等価性の検証

### 手法: Property-Based Testing
```nyash
// 1000個のランダムプログラムを生成
for i in 1..1000 {
    local program = generateRandomProgram()
    
    // GC有効で実行
    local resultWithGC = executeWithGC(program)
    
    // GC無効で実行
    local resultWithoutGC = executeWithoutGC(program)
    
    // 結果の同一性確認
    assert(resultWithGC == resultWithoutGC)
    assert(sameMemoryTrace(program))
}
```

### 検証項目
1. 実行結果の同一性
2. 例外発生の同一性
3. メモリ解放順序の決定性
4. 副作用の発生順序

## 📊 実験環境

### ハードウェア
- CPU: AMD Ryzen 9 5950X
- RAM: 64GB DDR4-3600
- Storage: Samsung 980 PRO 2TB

### ソフトウェア
- OS: Ubuntu 22.04 LTS
- Nyash: Version 1.0.0
- Rust: 1.75.0
- Go: 1.21

### 統計解析
- 有意水準: α = 0.05
- 多重比較: Bonferroni補正
- 効果量: Cohen's d

## 📅 実験スケジュール

| 週 | 実験内容 | 成果物 |
|----|---------|---------|
| 1-2 | 環境構築・予備実験 | 実験プロトコル |
| 3-4 | 実験1: 開発効率 | 生産性データ |
| 5-6 | 実験2: リーク検出 | 検出精度データ |
| 7-8 | 実験3: 性能測定 | ベンチマーク結果 |
| 9-10 | 実験4: 等価性検証 | 形式的証明 |
| 11-12 | データ解析・論文執筆 | 論文原稿 |

## 🔍 追加実験案

### 長期運用実験
- 3ヶ月間の実プロジェクトでの使用
- メンテナンス性の評価
- チーム開発での有効性

### 教育効果の測定
- プログラミング初学者への導入
- 学習曲線の比較
- メモリ管理概念の理解度

---

*実験計画は随時更新される可能性があります*