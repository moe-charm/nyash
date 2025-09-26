# Phase 15.5 リスク分析と対策

**大規模基盤変更における潜在的リスクと対応戦略**

## 🚨 高リスク項目

### 1. 型安全性の喪失

**リスク**: JSON化により型チェックが弱くなる
**影響度**: 高（バグ検出能力低下）
**確率**: 中

**対策**:
```rust
// 型安全ビューの維持
pub struct TypedJsonView<'a> {
    json: &'a JsonValue,
    type_registry: &'a TypeRegistry,
}

impl<'a> TypedJsonView<'a> {
    fn get_value(&self, id: ValueId) -> TypedValue {
        // 型安全なアクセスを保証
    }
}
```

**検証方法**:
- [ ] 型エラーの検出率比較（移行前後）
- [ ] 実行時エラーの増減監視
- [ ] 静的解析ツールでの型安全性確認

### 2. パフォーマンス劣化

**リスク**: JSON解析コストによる実行速度低下
**影響度**: 高（ユーザー体験悪化）
**確率**: 中

**対策**:
```rust
// 遅延解析 + キャッシュ戦略
pub struct CachedJsonMir {
    json: JsonValue,
    parsed_cache: HashMap<String, ParsedFunction>,
}

impl CachedJsonMir {
    fn get_function(&mut self, name: &str) -> &ParsedFunction {
        self.parsed_cache.entry(name.to_string())
            .or_insert_with(|| self.parse_function(name))
    }
}
```

**ベンチマーク計画**:
- [ ] 大規模プログラムの実行時間測定
- [ ] メモリ使用量の比較
- [ ] コンパイル時間の測定

### 3. 互換性破綻

**リスク**: 既存機能が動作しなくなる
**影響度**: 極高（開発停止）
**確率**: 中

**対策**:
```bash
# 段階的フラグ制御
NYASH_JSON_VERSION=v0  # 既存互換
NYASH_JSON_VERSION=v1  # 統一Call対応
NYASH_JSON_VERSION=auto # 自動選択
```

**互換性マトリクス**:
| 機能 | v0互換 | v1対応 | テスト状況 |
|------|--------|--------|------------|
| print文 | ✅ | ✅ | 完了 |
| BoxCall | ✅ | 🔄 | 進行中 |
| ExternCall | ✅ | ⏳ | 未着手 |

### 4. JSON仕様の複雑化

**リスク**: JSON v1が複雑すぎて保守困難
**影響度**: 中（保守コスト増）
**確率**: 高

**対策**:
```json
// 最小限の拡張
{
  "version": 1,
  "extensions": ["unified_call"],  // 機能を明示
  "call": {
    "callee": { "kind": "Global", "name": "print" },
    "args": [42]
  }
}
```

**設計原則**:
- 既存v0との差分を最小化
- 拡張可能だが必須でない構造
- 人間が読めるシンプルさ維持

---

## ⚠️ 中リスク項目

### 5. 開発リソース不足

**リスク**: Phase 15.5完了前にリソース枯渇
**影響度**: 高（セルフホスティング遅延）
**確率**: 中

**対策**:
- **MVP優先**: Phase A核心機能のみに集中
- **並行作業**: ドキュメント・テストを先行
- **撤退戦略**: 各段階で既存状態に戻せる設計

### 6. テスト網羅性不足

**リスク**: 移行で見落としたバグが本番で発生
**影響度**: 中（品質低下）
**確率**: 高

**対策**:
```bash
# 包括的テスト戦略
./tools/phase15_5_regression_test.sh  # 回帰テスト
./tools/phase15_5_integration_test.sh # 統合テスト
./tools/phase15_5_performance_test.sh # 性能テスト
```

### 7. セルフホスティング影響

**リスク**: Phase 15.5がPhase 15を複雑化
**影響度**: 高（本来目標への影響）
**確率**: 中

**対策**:
- **クリーン分離**: Phase 15.5は独立完結
- **基盤提供**: Phase 15が確実に楽になる基盤整備
- **優先度明確化**: セルフホスティング準備が最優先

---

## 📉 低リスク項目

### 8. ツール連携問題

**リスク**: デバッガー・プロファイラが使えなくなる
**影響度**: 低（開発体験のみ）
**確率**: 中

**対策**: 段階的対応、最後に修正

### 9. ドキュメント不整合

**リスク**: 文書が実装と乖離
**影響度**: 低（保守のみ）
**確率**: 高

**対策**: CI/CDでドキュメント同期チェック

---

## 🛡️ リスク軽減戦略

### 段階的実装
```
Phase A (核心) → Phase B (拡張) → Phase C (完成)
  ↓撤退可能      ↓撤退可能      ↓撤退可能
```

### 二重実装期間
```
期間1-2週: 旧実装 + 新実装(テスト)
期間3-4週: 新実装 + 旧実装(バックアップ)
期間5週以降: 新実装のみ
```

### 機能フラグ制御
```rust
#[cfg(feature = "json_v1")]
fn emit_unified_call() { ... }

#[cfg(not(feature = "json_v1"))]
fn emit_legacy_call() { ... }
```

## 🔄 撤退戦略

### Phase A撤退
```bash
# 環境変数でv0に戻す
export NYASH_MIR_UNIFIED_CALL=0
export NYASH_JSON_VERSION=v0
```

### Phase B撤退
```rust
// MIR Module保持、JSON化を停止
pub struct MirModule {
    // #[deprecated] json_wrapper: JsonWrapper,
    rust_data: RustMirData,  // 復活
}
```

### Phase C撤退
- Rust MIR完全復活
- JSON v0をビルド時オプション化

## 📊 モニタリング計画

### メトリクス
- **型安全性**: 静的解析警告数
- **パフォーマンス**: 実行時間・メモリ使用量
- **互換性**: 既存テストパス率
- **品質**: バグ報告数・重要度

### 監視タイミング
- Phase A完了時点
- 各週のマイルストーン
- Phase 15開始前の最終確認

### 判断基準
```
継続条件:
- 型安全性維持 (警告増加<10%)
- 性能劣化なし (実行時間増加<5%)
- 互換性確保 (テストパス率=100%)

撤退条件:
- 上記いずれかが達成不可能
- スケジュール大幅遅延(2週間以上)
- 予期しない技術的障害
```

---

## 🎯 成功のための重要ポイント

1. **急がない**: 確実な段階移行
2. **測る**: 定量的な品質確認
3. **戻れる**: いつでも撤退可能
4. **集中する**: Phase A核心機能優先
5. **テストする**: 徹底的な検証

これらのリスク対策により、Phase 15.5は安全かつ確実に完了し、Phase 15セルフホスティングの強固な基盤となる。