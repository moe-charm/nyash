# GC切り替え可能言語 - 史上初の柔軟なメモリ管理

Status: Research
Created: 2025-08-26
Priority: Medium
Related: メモリ管理、参照カウント、開発効率

## 🌟 コンセプト

Nyashを**世界初のGC切り替え可能言語**にする革新的アイデア。開発時はGCオンで快適に、本番ではGCオフで高性能に。

## 🎯 基本アイデア

```nyash
// 開発時: メモリリークを気にせず開発
nyash --gc-mode=ref-counting app.nyash

// 本番: 最高性能で実行
nyash --gc-mode=explicit app.nyash
```

## 📊 メモリ管理モード

### 1. Explicit Mode（デフォルト）
- 現在のNyashの動作
- スコープ抜けたら即fini()
- 予測可能な性能
- リアルタイムアプリ向け

### 2. Reference Counting Mode（新規）
- 参照カウントが0になったらfini()
- share_box()で参照カウント++
- 循環参照はweak参照で解決
- 一般アプリ向け

## 🔧 実装戦略

### インタープリター（簡単）
```rust
// 実行時のモード切り替えで対応
match memory_mode {
    RefCounting => {
        // 代入時に参照カウント操作
        old_value.dec_ref();
        new_value.inc_ref();
    }
    Explicit => {
        // 現在の動作
    }
}
```

### MIR（中程度）
```rust
// 新しい抽象命令を追加
enum MirInstruction {
    Acquire(temp_id),  // 値の取得（GCならref++）
    Release(temp_id),  // 値の解放（GCならref--/fini）
}

// MIRは同じ、実行時に挙動を変える
```

### VM（やや複雑）
- Acquire/Release命令の実装
- モードフラグによる分岐
- 参照カウント0検出時のfini呼び出し

### Cranelift JIT（複雑）
```rust
// モード別のコード生成
if gc_mode == Explicit {
    // Acquire/Release命令をスキップ
    // 直接fini呼び出し
} else {
    // 参照カウント操作のインライン展開
    // 条件付きfini呼び出し
}
```

## 🚀 実装フェーズ

### Phase 1: 基礎実装（1-2週間）
- [ ] BoxBaseに参照カウント追加
- [ ] インタープリターでのモード切り替え
- [ ] 基本的なテスト

### Phase 2: MIR/VM対応（1ヶ月）
- [ ] Acquire/Release命令の設計
- [ ] MIRビルダーの対応
- [ ] VM実行時の最適化

### Phase 3: JIT最適化（Cranelift後）
- [ ] モード別コード生成
- [ ] デッドコード除去
- [ ] インライン最適化

## 💡 革新的な使い方

### 開発フロー
```bash
# 1. 開発中: GCオンで快適開発
nyash --gc-mode=ref-counting --detect-leaks dev.nyash

# 2. テスト: メモリリーク検出
nyash --gc-mode=ref-counting --memory-report test.nyash

# 3. 本番: GCオフで最高性能
nyash --gc-mode=explicit --optimize prod.nyash
```

### ハイブリッドモード
```nyash
// ファイル単位での制御
// @gc-mode: explicit
box RealtimeAudioProcessor { }

// @gc-mode: ref-counting
box UIController { }
```

## 🤔 技術的課題

### 1. 統一的なMIR表現
- GCモードに依存しないMIR設計
- 実行時の最適化余地を残す

### 2. JITコード生成の複雑化
- モード別の最適化パス
- コードキャッシュの管理

### 3. デバッグ情報の保持
- どのモードで実行されているか
- 参照カウントの可視化

## 🎉 期待される効果

1. **開発効率**: GCありで快適開発
2. **実行性能**: GCなしで最高速
3. **教育価値**: メモリ管理の学習に最適
4. **柔軟性**: アプリケーションに応じた選択

## 📚 参考資料
- Rust: 所有権システム（コンパイル時）
- Swift: ARC（自動参照カウント）
- Go/Java: 強制GC
- C/C++: 手動管理

## 🔮 将来の拡張
- Mark & Sweep GCモードの追加
- 世代別GC
- リージョンベースメモリ管理
- プロファイルベース自動選択

## 実装タイミング
- Cranelift JIT実装後が理想的
- まずはインタープリターで実験
- 実用性を確認してから本格実装