# Nyash ABI/TypeBox 統合インデックス

このドキュメントは、分散しているNyash ABI関連ドキュメントへの統合入口です。

## 🎯 現在の実装状態（Phase 15）

### 実装済み
- **TypeBox FFI基本構造** - TLVベースの統一`invoke_id`
- **struct_sizeフィールド** - 前方互換性確保
- **プラグインローダーv2** - 7個の基本Boxプラグイン動作中

### 計画中
- **専用関数追加** - `create`/`destroy`/`get_type_info`
- **パフォーマンス最適化** - 直接関数ポインタ呼び出し
- **C ABI完全互換** - 他言語との相互運用性向上

## 📚 主要ドキュメント

### 1. コア設計ドキュメント

#### [NYASH_ABI_MIN_CORE.md](../../private/reference/abi/NYASH_ABI_MIN_CORE.md)
**最小コアABIとエボリューション戦略**
- NyrtValue 16B固定表現
- struct_sizeによる前方互換性
- feature_bits交渉メカニズム
- エボリューション・ハッチ（将来拡張への逃げ道）

#### [typebox-api-reference.md](../../development/roadmap/phases/phase-12/specs/typebox-api-reference.md)  
**統一TypeBox APIリファレンス**
- NyashTypeBox C構造体仕様
- NyValue型システム
- 関数ポインタ仕様（create/destroy/invoke_id）
- エラーハンドリング・所有権ルール

#### [NYASH-ABI-C-IMPLEMENTATION.md](../../development/roadmap/phases/phase-12/design/NYASH-ABI-C-IMPLEMENTATION.md)
**Nyash ABIをTypeBoxとして実装する革新的設計**
- "Everything is Box" → ABIもBox
- 3段階実装戦略（C Shim → フルC → Nyash再実装）
- Tagged Pointers・セレクターキャッシング
- 3大AI（Gemini/Codex/ChatGPT5）による評価

### 2. 実装詳細

#### [unified-typebox-abi.md](../../development/roadmap/phases/phase-12/unified-typebox-abi.md)
**Phase 12統一TypeBox ABI実装計画**
- 移行ロードマップ
- パフォーマンス目標（33倍高速化）
- 互換性維持戦略

#### [PLUGIN_ABI.md](PLUGIN_ABI.md)
**プラグインABI概要**
- 現在のTLVベース実装
- プラグイン開発ガイド

### 3. 現在のソースコード実装

#### Rust側（ランタイム）
```rust
// src/runtime/plugin_loader_v2/enabled/types.rs
#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,
    pub version: u16,
    pub struct_size: u16,    // 前方互換性キー！
    pub name: *const c_char,
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(u32, u32, *const u8, usize, *mut u8, *mut usize) -> i32>,
    pub capabilities: u64,
}
```

#### プラグイン側（C/Rust）
- `plugins/nyash-string-plugin/`
- `plugins/nyash-array-plugin/`
- その他基本Boxプラグイン

## 🔄 ABI拡張方法

### 1. 構造体末尾に新フィールド追加
```rust
pub struct NyashTypeBoxFfi {
    // 既存フィールド...
    
    // 新規追加（末尾に！）
    pub create: Option<extern "C" fn(*const u8) -> u32>,
    pub destroy: Option<extern "C" fn(u32) -> i32>,
}
```

### 2. struct_sizeで互換性チェック
```rust
let actual_size = (*tb).struct_size;
if actual_size >= expected_size {
    // 新機能使用可能
}
```

## 🚀 移行タイミング検討

### Option 1: **LLVM完成後に移行**（推奨）
**メリット**：
- LLVMの最適化機会を考慮した設計可能
- JIT/AOTでの関数ポインタ直接呼び出し最適化
- 実行パスが確定してからの最適設計

**デメリット**：
- 移行が遅れる
- 現在のTLVオーバーヘッドが継続

### Option 2: **今すぐ段階的移行**
**メリット**：
- プラグイン数が少ない今がチャンス
- 早期にパフォーマンス改善
- 将来の技術的負債を回避

**デメリット**：
- LLVM実装と並行作業
- 優先度の分散

### 結論：**LLVM Phase完了後が最適**
理由：
1. 現在のTLVベースでも実用上問題なし
2. LLVM最適化と同時に設計見直し
3. 一度の大きな変更で済む
4. Phase 16でのABI安定化に向けた準備

## 関連リンク

- [Phase 15 README](../../development/roadmap/phases/phase-15/README.md) - 現在のフェーズ
- [Phase 22 構想](../../development/roadmap/phases/phase-22/) - Nyash LLVMコンパイラ
- [Box設計哲学](../../development/architecture/box-externcall-design.md) - Box/ExternCall境界

---
最終更新: 2025-09-12
