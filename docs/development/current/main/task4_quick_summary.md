# Task 4: set_param_count() 検証 - クイックサマリ

**調査結果**: ✅ **set_param_count() のロジックは正しい（問題なし）**

---

## 🎯 **結論（30秒版）**

1. **set_param_count() 実装は正しい**
   - `if param_count > self.next_id` 条件は妥当
   - next_id の後退を防ぐ安全設計

2. **真の問題は別の箇所**
   - 117箇所の `value_gen.next()` 直接呼び出しが原因
   - これらが ValueIdAllocatorBox を経由しない
   - → パラメータレジスタ v%0-v%N を上書き

3. **修正方法**
   - `value_gen.next()` → `safe_next_value()` に置換（117箇所）
   - `HAKO_USE_VALUE_ALLOCATOR_BOX=1` をデフォルトに

---

## 📊 **value_gen.next() 呼び出し箇所（117箇所）**

| ファイル | 数 | 優先度 |
|---------|---|-------|
| builder_calls/build.rs | 29 | P0 |
| ops.rs | 17 | P0 |
| utils.rs | 10 | P1 |
| その他 (15ファイル) | 61 | P2-P3 |

---

## 🔧 **即座にできる検証**

```bash
# Legacy経路（問題再現）
HAKO_USE_VALUE_ALLOCATOR_BOX=0 ./target/release/hakorune apps/examples/json_query/main.nyash

# 新規安全経路（修正確認）
HAKO_USE_VALUE_ALLOCATOR_BOX=1 ./target/release/hakorune apps/examples/json_query/main.nyash

# トレースログ
HAKO_TRACE_VALUE_ALLOC=1 HAKO_USE_VALUE_ALLOCATOR_BOX=1 ./target/release/hakorune test.hako
```

---

## 📚 **詳細レポート**

→ [task4_set_param_count_timing_report.md](./task4_set_param_count_timing_report.md)

---

**調査日**: 2025-10-17
**次のステップ**: Phase 2.P2（117箇所の置換計画）
