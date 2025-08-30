# Paper 11: コンパイラは世界を知らない：PluginInvoke一元化と"フォールバック廃止"の実践

Status: Planning  
Target: Systems/Engineering Conference (OSDI/SOSP/ASPLOS)  
Lead Author: Nyash Team  

## 📋 論文概要

### タイトル
**The Compiler Knows Nothing: PluginInvoke Unification and the Practice of "No Fallback" Policy**

### 中心的主張
- コンパイラからドメイン知識を完全排除し、全てをPluginInvokeに一元化
- フォールバック全廃により複雑性爆発を回避し、保守性を最大化
- 対応表1枚（mir→vm→jit）で全ての拡張を管理可能に

### 主要貢献
1. **設計原則**：「コンパイラは世界を知らない」哲学の体系化
2. **実装手法**：フォールバック廃止による複雑性制御
3. **運用知見**：プラグインエコシステムの実践的構築法

## 🔧 実証データ計画

### フォールバック削減の定量化
```
削減前（Phase 9）:
- 型名分岐: 47箇所
- 特殊処理: 23箇所
- フォールバック: 15箇所

削減後（Phase 10.11）:
- 型名分岐: 0箇所
- 特殊処理: 0箇所
- フォールバック: 0箇所
```

### 保守性メトリクス
```
コード変更影響範囲：
- 新Box追加時の変更行数: 0行（プラグインのみ）
- 新機能追加時の変更行数: 0行（プラグインのみ）
- コンパイラ本体の安定性: 100%（変更不要）
```

### プラグイン統合実績
```
統合成功例：
- Python統合: 2日（eval/import/getattr/call）
- ファイルI/O: 1日
- ネットワーク: 1日
- 数学関数: 0.5日
```

## 📊 実践的証明

### 型名分岐の回避例（Before/After）

**Before（アンチパターン）**:
```rust
match box_type {
    "StringBox" => self.emit_string_length(),
    "ArrayBox" => self.emit_array_length(),
    "FileBox" => self.emit_file_size(),
    // 新しいBoxごとに分岐追加... 😱
}
```

**After（PluginInvoke一元化）**:
```rust
self.emit_plugin_invoke(type_id, method_id, args)
// 新しいBox追加時もコード変更不要！ 🎉
```

### CI/CDによる品質保証

```yaml
禁止パターンCI:
- 型名文字列による分岐
- ビルトイン特殊処理
- フォールバック実装

必須テスト:
- trace_hash等価性（VM/JIT/AOT）
- プラグイン境界テスト
- ABI互換性チェック
```

## 🏗️ アーキテクチャ設計

### 対応表による一元管理
```
MIR → VM → JIT/AOT マッピング:
┌─────────────┬────────────────┬─────────────────┐
│ MIR命令     │ VM実装         │ JIT/AOT実装     │
├─────────────┼────────────────┼─────────────────┤
│ PluginInvoke│ plugin_invoke()│ emit_plugin_call│
└─────────────┴────────────────┴─────────────────┘
（1行で全てを表現）
```

### ABI v0の設計
```
最小限のFFI契約:
- invoke(type_id, method_id, args) → TLV
- 引数: TLVエンコード
- 戻り値: TLVエンコード
- エラー: Result<TLV, String>
```

## 📝 論文構成（予定）

### 1. Introduction（2ページ）
- 問題：言語実装の複雑性爆発
- 解決：ドメイン知識のプラグイン分離
- 影響：保守性と拡張性の両立

### 2. Design Philosophy（3ページ）
- 「コンパイラは世界を知らない」原則
- フォールバック廃止の必要性
- プラグイン境界の設計

### 3. Implementation（4ページ）
- PluginInvoke一元化の実装
- 型名分岐の除去プロセス
- CI/CDによる品質維持

### 4. Case Studies（3ページ）
- Python統合（複雑な例）
- FileBox（I/O例）
- NetworkBox（非同期例）

### 5. Evaluation（3ページ）
- 保守性の定量評価
- 性能オーバーヘッド分析
- 開発効率の改善

### 6. Lessons Learned（2ページ）
- 成功要因の分析
- 失敗と回避策
- ベストプラクティス

### 7. Related Work（2ページ）
- プラグインアーキテクチャ
- 言語拡張機構
- モジュラーコンパイラ

### 8. Conclusion（1ページ）

## 🎯 執筆スケジュール

- Week 1: 実装データ収集・整理
- Week 2: Philosophy + Implementation執筆
- Week 3: Case Studies執筆
- Week 4: Evaluation + Lessons執筆
- Week 5: 推敲・コード例整備

## 💡 期待される影響

### 学術的影響
- コンパイラ設計の新しいパラダイム
- 複雑性管理の体系的手法
- プラグインエコシステムの理論

### 実務的影響
- 言語実装のベストプラクティス集
- 保守可能な言語の作り方
- 拡張可能なアーキテクチャ設計

## 📚 参考文献（予定）
- The Cathedral and the Bazaar (ESR)
- Design Patterns (GoF)
- Clean Architecture (Robert C. Martin)
- LLVM: A Compilation Framework for Lifelong Program Analysis
- The Art of Unix Programming