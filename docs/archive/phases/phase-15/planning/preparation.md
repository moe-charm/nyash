# Phase 15 セルフホスティング準備まとめ
作成日: 2025-09-03
作成者: Claude (Gemini・Codex協議結果統合)

## 専門家の技術評価まとめ

### Gemini先生の分析
- **実現可能性**: MIR 13命令で十分実現可能、BoxCallの設計が鍵
- **推奨バックエンド**: Cranelift + lld（開発速度・安全性・成熟度）
- **コード削減**: 75%削減は現実的（Arc<Mutex>→GC、動的ディスパッチ）
- **段階的アプローチ**: まず動くものを作り、後から最適化

### Codex先生の具体設計
- **BoxCall実装**: 隠れクラス（Shape）+ vtable + セレクタインターン
- **JIT最適化**: IC/PIC、Shapeガード、devirtualization
- **ブートストラップ**: c0→c1→c1'の具体手順、決定論的ビルド
- **並列化**: GCでロック削減、フェーズ境界でバリア同期

## 今すぐ着手可能な準備作業

### 1. BoxCall設計の詳細化（最優先）
```nyash
// BoxCall命令のメタデータ設計
BoxCall {
    dst: ValueId,
    receiver: ValueId,
    selector: Sel(u32),  // インターン化されたメソッド名
    args: Vec<ValueId>,
    flags: {
        op_kind: OpKind,     // Get/Set/Invoke/Convert
        target_type: Option<TypeId>,
        site_id: u32,        // IC/PIC管理用
    }
}
```

### 2. 最小言語サブセット定義
**必須機能**:
- 基本型（Integer, String, Bool, Array, Map）
- Box定義（box, birth, field, method）
- 制御構造（if, loop, return）
- 関数定義（static/instance method）
- エラー処理（Result型）

**初期は省略**:
- ジェネリクス
- trait/interface
- マクロ
- 非同期（async/await）

### 3. セレクタインターン実装
```rust
// src/runtime/selector_intern.rs
pub struct SelectorInterner {
    string_to_id: HashMap<String, Sel>,
    id_to_string: Vec<String>,
}
```

### 4. TypeDesc/VTable構造定義
```rust
// crates/nyrt/src/types.rs
pub struct TypeDesc {
    id: TypeId,
    vtable: *const VTable,
    shape_epoch: u32,
}

pub struct VTable {
    get: fn(recv: *mut BoxHdr, sel: Sel) -> Value,
    set: fn(recv: *mut BoxHdr, sel: Sel, val: Value),
    invoke: fn(recv: *mut BoxHdr, sel: Sel, args: &[Value]) -> Value,
    // ... 他のメソッド
}
```

### 5. MIR最適化パス準備
- BoxCallのdevirtualization検出
- Shapeガード生成
- IC/PICサイト管理

## 実装ロードマップ

### Phase 1: 基盤整備（1-2ヶ月）
1. BoxCall命令の完全定義
2. セレクタインターンシステム
3. TypeDesc/VTable基盤
4. 最小サブセット言語仕様

### Phase 2: プロトタイプ（2-3ヶ月）
1. 素朴なBoxCall実装（文字列ディスパッチ）
2. Cranelift統合
3. 最小コンパイラ（c0.5）実装

### Phase 3: 最適化（2-3ヶ月）
1. Shape/vtableハイブリッド
2. IC/PIC実装
3. devirtualization

### Phase 4: セルフホスティング（2-3ヶ月）
1. c1実装（Nyashで20,000行）
2. ブートストラップ検証
3. 性能チューニング

## 技術的リスクと対策

### リスク
1. BoxCallの性能オーバーヘッド
2. ブートストラップの決定論性
3. デバッグの困難さ

### 対策
1. 早期にIC/PIC実装で緩和
2. SOURCE_DATE_EPOCH等で環境統一
3. MIRダンプ比較ツール整備

## 成功の指標
- c1がc1'を正しくコンパイル（バイナリ一致）
- 80,000行→20,000行達成
- VM比2倍以上の性能（Cranelift JIT）

## 次のアクション
1. BoxCall詳細設計ドキュメント作成
2. セレクタインターン実装開始
3. 最小サブセット言語仕様確定
4. MIRへのBoxCallメタデータ追加