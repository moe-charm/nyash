# Box-SSA Core-15 最終仕様

Date: 2025-08-31  
Status: **凍結** (Frozen Specification)  
Author: ChatGPT5 + Claude協調

## ✅ 凍結命令セット（正味15個）

```
{ Const, UnaryOp, BinOp, Compare, TypeOp,
  Load, Store,
  Jump, Branch, Return, Phi,
  Call, NewBox, BoxCall, ExternCall }
```

## 📋 命令詳細

### 1. 値生成
- **Const**(value) → 定数（i64/f64/bool/string等）
- **NewBox**(type, init_args...) → 新しいBoxオブジェクト生成

### 2. 演算
- **UnaryOp**(op, arg) → 単項演算（neg, not等）
- **BinOp**(op, left, right) → 二項演算（add, sub, mul, div等）
- **Compare**(op, left, right) → 比較演算（eq, lt, le等）
- **TypeOp**(op, value, type) → 型演算（is, as, typeof等）

### 3. メモリ
- **Load**(local_id) → ローカル変数読み込み
- **Store**(local_id, value) → ローカル変数書き込み

### 4. 制御フロー
- **Jump**(block) → 無条件ジャンプ
- **Branch**(cond, then_block, else_block) → 条件分岐
- **Return**(value?) → 関数からの復帰
- **Phi**([(block, value), ...]) → SSA用（VMは展開）

### 5. 呼び出し
- **Call**(func, args...) → 通常の関数呼び出し
- **BoxCall**(box, selector, args...) → Boxメソッド呼び出し（万能）
- **ExternCall**(symbol, args..., attrs) → FFI呼び出し

## 🎯 BoxCall統一マッピング

| 操作 | 旧命令 | 新BoxCall表現 |
|------|--------|---------------|
| フィールド読み取り | RefGet | BoxCall(obj, "getField", field_name) |
| フィールド書き込み | RefSet | BoxCall(obj, "setField", field_name, value) |
| 配列要素読み取り | ArrayGet | BoxCall(arr, "get", index) |
| 配列要素書き込み | ArraySet | BoxCall(arr, "set", index, value) |
| プラグイン呼び出し | PluginInvoke | BoxCall(plugin, "invoke", method, args...) |
| メソッド呼び出し | - | BoxCall(obj, method_name, args...) |

## 🔒 不変条件（Verifier必須チェック）

1. **直接フィールドアクセス禁止**
   - ❌ `Load/Store`でBoxの内部フィールドに直接アクセス
   - ✅ 必ず`BoxCall`経由でアクセス

2. **Write Barrier自動挿入**
   - BoxCallのLowering時に必要に応じて挿入
   - 世代別GCで若→若の場合は省略可

3. **ExternCall属性必須**
   - `noalloc`, `readonly`, `atomic`, `nothrow`等を明示
   - 無指定は保守的（全バリア有効）

4. **型安全性**
   - TypeOpで型チェック後のみ特定操作を許可
   - 動的ディスパッチはPIC経由

## 🛠️ Lowering戦略

### BoxCall → 最適化されたコード

```llvm
; BoxCall(arr, "get", 5) のLowering例

; 1. 形状ガード（PIC）
%type_id = load i64, i64* %arr.type_id
%is_array = icmp eq i64 %type_id, ARRAY_TYPE_ID
br i1 %is_array, label %fast_path, label %slow_path

fast_path:
  ; 2. 境界チェック
  %len = load i64, i64* %arr.length
  %in_bounds = icmp ult i64 5, %len
  br i1 %in_bounds, label %do_load, label %bounds_error

do_load:
  ; 3. 直接アクセス（最適化後）
  %ptr = getelementptr %ArrayBox, %arr, 0, 2, 5
  %value = load %Box*, %ptr
  br label %continue

slow_path:
  ; 4. 汎用ディスパッチ
  %value = call @nyash_box_call(%arr, "get", 5)
  br label %continue
```

## 📊 削減効果

| 項目 | 旧（26命令） | 新（15命令） | 削減率 |
|------|-------------|-------------|---------|
| 命令数 | 26 | 15 | 42%削減 |
| 実装箇所 | 26×N | 15×N | 42%削減 |
| 最適化パターン | 多様 | 統一（BoxCall中心） | 大幅簡素化 |
| テストケース | O(26²) | O(15²) | 66%削減 |

## 🚦 実装ロードマップ

### Phase 1: 仕様更新（即時）
- [x] このドキュメントで仕様凍結
- [ ] INSTRUCTION_SET.md を更新
- [ ] テストの期待値を15に変更

### Phase 2: Verifier実装（1日）
- [ ] Box直接アクセス検出
- [ ] ExternCall属性チェック
- [ ] 命令数15の強制

### Phase 3: Lowering実装（3日）
- [ ] BoxCall → 形状別分岐
- [ ] Write Barrier挿入ロジック
- [ ] PIC統合

### Phase 4: VM/JIT更新（1週間）
- [ ] VM: Phi展開、BoxCall dispatch
- [ ] JIT: PIC生成、インライン化
- [ ] 性能検証

## 🎉 結論

**Box-SSA Core-15**により：
- Everything is Box哲学の完全実現
- 実装・保守の劇的簡素化
- 最適化の統一的適用
- 真の15命令達成

これが「あほみたいに簡単」で「恐ろしく速い」Nyashの最終形態です！