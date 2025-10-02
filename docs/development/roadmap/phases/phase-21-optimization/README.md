# Phase 21: LLVM最適化フェーズ 🚀

**開始予定**: Phase 15完了後（Phase 21-30の間）
**目標**: nyashの14命令 → LLVM最適化能力の最大活用

---

## 🎯 **フェーズの目的**

現在の実装は**正確性優先**:
- ✅ 型変換統一（TypeCoercion箱）
- ✅ 値解決統一（PhiDispatchPoint）
- ✅ 文字列判定（StringTagPolicy）

Phase 21では**パフォーマンス最適化**にフォーカス:
- LLVMの強力な最適化機能を活用
- ポリシー箱による戦略的最適化
- 未定義動作（UB）の完全回避

---

## 📋 **実装ステージ（4段階）**

### **Stage 1: 基礎インフラ（必須）** 🏗️

**目的**: 安全な最適化の土台作り

#### 1.1 LLVM Verifier統合
```python
# 全関数生成後に自動検証
def verify_function(func):
    """LLVM Verifierで検証"""
    if not func.verify():
        raise CompilerError(f"Invalid IR in {func.name}")
```

**実装タイミング**: Stage 1最優先
**効果**: バグの早期発見（開発速度向上）

#### 1.2 未定義動作（UB）検出
```python
class UBChecker:
    """未定義動作の自動検出"""

    def check_null_deref(self, ptr):
        """nullポインタ参照チェック"""
        if ptr might be null:
            insert_null_check(ptr)

    def check_overflow(self, val):
        """整数オーバーフローチェック"""
        if signed_int:
            use_checked_arithmetic(val)
```

**対象UB**:
- ✅ nullポインタ参照
- ✅ 符号付き整数オーバーフロー
- ✅ 未初期化変数の使用
- ✅ 配列境界外アクセス

#### 1.3 データレイアウト標準化
```python
# ターゲット別のレイアウト定義
target_layouts = {
    "x86_64": "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
    "wasm32": "e-m:e-p:32:32-i64:64-n32:64-S128",
}
```

---

### **Stage 2: 型情報活用** 📦

**目的**: Everything is Box → LLVMに構造を教える

#### 2.1 Box構造のLLVM定義
```python
# StringBox の構造体定義
StringBox = ir.LiteralStructType([
    ir.IntType(32),        # length
    ir.IntType(8).as_pointer()  # data*
])

# extractvalue で高速アクセス
length = builder.extract_value(string_box, 0)  # 関数呼び出し不要！
```

**効果**:
- BoxCall呼び出し削減
- メモリアクセス最適化
- インライン展開促進

#### 2.2 TBAA（Type-Based Alias Analysis）
```python
class TBAAPolicy:
    """型ベースエイリアシング情報"""

    def annotate_box_access(self, inst, box_type):
        """Box型ごとのTBAA付与"""
        tbaa_metadata = {
            "StringBox": self.string_tbaa_node,
            "IntegerBox": self.integer_tbaa_node,
        }
        inst.set_metadata("tbaa", tbaa_metadata[box_type])
```

**効果**:
- 命令並び替え最適化
- 不要なメモリアクセス削減
- ループ最適化向上

#### 2.3 extractvalue命令への置き換え
```python
# Before: 関数呼び出し（遅い）
length = builder.call(get_length_func, [string_box])

# After: 構造体アクセス（速い）
length = builder.extract_value(string_box, 0, name="length")
```

---

### **Stage 3: 関数最適化** ⚡

**目的**: LLVM Intrinsics活用 + 関数属性付与

#### 3.1 LLVM Intrinsics活用
```python
# memcpy最適化
# Before: ループでコピー
for i in range(len):
    dst[i] = src[i]

# After: LLVM Intrinsic
builder.call(llvm_memcpy, [dst, src, length])
```

**対象Intrinsics**:
- `@llvm.memcpy`: メモリ一括コピー
- `@llvm.memmove`: オーバーラップ対応コピー
- `@llvm.memset`: メモリ初期化
- `@llvm.sqrt`: 平方根計算
- `@llvm.abs`: 絶対値計算

#### 3.2 関数属性付与
```python
class FunctionAttributePolicy:
    """関数の性質を明示"""

    def annotate_pure_function(self, func):
        """副作用なし関数"""
        func.attributes.add("readnone")
        # LLVMが同じ引数の呼び出しを1回に最適化！

    def annotate_readonly_function(self, func):
        """読み込み専用関数"""
        func.attributes.add("readonly")
```

**属性一覧**:
- `readnone`: 完全な純粋関数（Math.add等）
- `readonly`: 読み込みのみ（Array.length等）
- `nounwind`: 例外を投げない
- `alwaysinline`: 常にインライン展開

#### 3.3 インライン展開ヒント
```python
# 小さい関数は積極的にインライン化
if func_size < 10:
    func.attributes.add("alwaysinline")
elif func_size > 100:
    func.attributes.add("noinline")
```

---

### **Stage 4: ポリシー箱実装** 🎁

**目的**: 戦略的最適化の統一管理

#### 4.1 TypeSafetyPolicy
```python
class TypeSafetyPolicy:
    """型安全性戦略"""

    STRICT = "strict"       # 型不一致は即エラー
    PERMISSIVE = "permissive"  # 暗黙的変換OK（現在）
    DEBUG = "debug"         # 全変換をログ出力

    def convert(self, val, target_type, mode):
        if mode == STRICT:
            if val.type != target_type:
                raise TypeError(f"Strict mode: {val.type} != {target_type}")
        elif mode == PERMISSIVE:
            return TypeCoercion.to_type(builder, val, target_type)
        elif mode == DEBUG:
            log_conversion(val, target_type)
            return TypeCoercion.to_type(builder, val, target_type)
```

#### 4.2 ErrorHandlingPolicy
```python
class ErrorHandlingPolicy:
    """エラー処理戦略"""

    FAIL_FAST = "panic"     # 即座にエラー
    FALLBACK = "default"    # デフォルト値（現在）
    COLLECT = "accumulate"  # エラー収集後一括報告

    def handle_type_error(self, error, mode):
        if mode == FAIL_FAST:
            raise error
        elif mode == FALLBACK:
            return Constant(i64, 0)  # ← 現在の実装
        elif mode == COLLECT:
            self.errors.append(error)
            return Constant(i64, 0)
```

#### 4.3 OptimizationPolicy
```python
class OptimizationPolicy:
    """最適化レベル戦略"""

    def convert_with_opt(self, val, opt_level):
        if opt_level == 0:  # -O0（デバッグ）
            # 境界チェック付き変換
            return checked_conversion(val)
        elif opt_level >= 2:  # -O2/-O3（リリース）
            # 高速変換（チェックなし）
            return TypeCoercion.to_i64(builder, val)
```

#### 4.4 PlatformPolicy
```python
class PlatformPolicy:
    """プラットフォーム戦略"""

    def get_native_int(self, target):
        """ターゲット別のネイティブ整数型"""
        if target == "x86_64":
            return ir.IntType(64)  # ← 現在
        elif target == "wasm32":
            return ir.IntType(32)
        elif target == "arm32":
            return ir.IntType(32)
```

---

## 🎯 **実装優先順位**

| Stage | 優先度 | 実装タイミング |
|-------|-------|-------------|
| Stage 1（基礎インフラ） | 🔴 最高 | Phase 21開始直後 |
| Stage 2（型情報活用） | 🟡 高 | Stage 1完了後 |
| Stage 3（関数最適化） | 🟢 中 | Stage 2完了後 |
| Stage 4（ポリシー箱） | 🔵 低 | 具体的問題発生時 |

---

## 📊 **期待される効果**

### **パフォーマンス向上**
- BoxCall削減: 30-50%削減（extractvalue化）
- メモリアクセス最適化: TBAA活用
- 関数呼び出しオーバーヘッド削減: Intrinsics/インライン化

### **開発体験向上**
- Verifier: バグの早期発見
- UBチェック: 未定義動作の完全回避
- エラー収集モード: デバッグ効率化

### **移植性向上**
- PlatformPolicy: 複数プラットフォーム対応
- データレイアウト標準化: クロスコンパイル対応

---

## 🚨 **注意点（Gemini指摘事項）**

### **1. 未定義動作（UB）を絶対に避ける**
- nullポインタ参照
- 符号付き整数オーバーフロー
- 未初期化変数の使用

### **2. PHI命令は特に慎重に**
- 全ての前任ブロックからの値を指定
- 漏れがあると検証エラー

### **3. Verifierを親友にする**
- 開発中は毎回実行
- 問題の早期発見が鍵

### **4. データレイアウト・呼び出し規約の統一**
- ターゲット別の標準ルールに従う
- C言語ライブラリとの互換性確保

---

## 📚 **関連ドキュメント**

- [llvm-optimization-strategy.md](./llvm-optimization-strategy.md) - Gemini提案統合
- [policy-box-design.md](./policy-box-design.md) - ポリシー箱詳細設計
- [integration-plan.md](./integration-plan.md) - 統合実装計画
- [Phase 15.8](../phase-15.8/) - 現在の型変換統一実装

---

## 🎊 **まとめ**

Phase 21は**正確性から性能へ**の転換点:
- Stage 1-3: 実装必須（基礎インフラ・型活用・関数最適化）
- Stage 4: YAGNI原則（必要になったら実装）

**箱理論の実践**: 最適化戦略も箱で管理！

---

**作成日**: 2025-10-02
**ベース**: Gemini最適化提案 + Claude分析統合
