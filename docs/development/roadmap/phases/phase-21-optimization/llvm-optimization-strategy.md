# LLVM最適化戦略（Gemini提案統合版）

**出典**: Gemini AI提案（2025-10-02）
**統合者**: Claude (Sonnet 4.5)

---

## 🎯 **基本方針**

nyashの14個の命令 → LLVMの何百もの低レベル命令

この「言葉のレベルの違い」をうまく利用するのが最適化の鍵！

---

## 📚 **最適化手法（4つの柱）**

### **1. LLVM Intrinsics（魔法の関数）を使う** 🪄

LLVMには `@llvm.` で始まる特別な関数（Intrinsics）がある。
これらは**LLVMのオプティマイザが特別な意味を知っている**魔法の関数！

#### **主要Intrinsics**

| Intrinsic | 用途 | 効果 |
|-----------|------|------|
| `@llvm.memcpy` | メモリ一括コピー | CPUの最速命令に変換 |
| `@llvm.memmove` | オーバーラップ対応コピー | 安全な一括コピー |
| `@llvm.memset` | メモリ初期化 | 一括ゼロクリア |
| `@llvm.sqrt.f64` | 平方根計算 | CPU命令1個に変換 |
| `@llvm.abs.i64` | 絶対値計算 | 分岐なし高速化 |
| `@llvm.bswap.i64` | バイトスワップ | エンディアン変換 |

#### **実装例: メモリコピー最適化**

```python
# ❌ Before: 自前ループ（遅い）
def copy_array_slow(dst, src, length):
    for i in range(length):
        dst_ptr = builder.gep(dst, [ir.Constant(i64, i)])
        src_ptr = builder.gep(src, [ir.Constant(i64, i)])
        val = builder.load(src_ptr)
        builder.store(val, dst_ptr)

# ✅ After: LLVM Intrinsic（速い）
def copy_array_fast(dst, src, length):
    memcpy_func = declare_llvm_memcpy(module)
    builder.call(memcpy_func, [
        dst,      # destination
        src,      # source
        length,   # byte count
        ir.Constant(i1, 0)  # is_volatile
    ])
```

**効果**:
- ループオーバーヘッド削減
- SIMD命令への自動変換
- CPUのDMA機能活用

---

### **2. Box構造をLLVMに教える** 📦

nyashでは「Everything is Box」だが、LLVMには具体的な構造体として伝える必要がある。

#### **Box構造の明示**

```python
# StringBox の構造定義
StringBox = ir.LiteralStructType([
    ir.IntType(32),               # 0: length
    ir.IntType(8).as_pointer()    # 1: data*
])

# IntegerBox の構造定義
IntegerBox = ir.LiteralStructType([
    ir.IntType(64)  # 0: value
])
```

#### **extractvalue命令への置き換え**

```python
# ❌ Before: 関数呼び出し（遅い）
length_func = module.get_function("StringBox.length")
length = builder.call(length_func, [string_box])

# ✅ After: 構造体アクセス（速い）
length = builder.extract_value(string_box, 0, name="length")
# → たった1命令！関数呼び出しオーバーヘッドなし
```

**効果**:
- 関数呼び出しコスト削減（10-100倍高速化）
- インライン展開促進
- レジスタ割り当て最適化

---

### **3. TBAA（Type-Based Alias Analysis）ヒント** 🎯

「このポインタとあのポインタは別物だよ！」とLLVMに教えてあげる。

#### **TBAAメタデータの設定**

```python
class TBAABuilder:
    """TBAA（型ベースエイリアス解析）メタデータ生成"""

    def __init__(self, module):
        self.module = module
        self.root = self._create_tbaa_root()
        self.box_types = {}

    def _create_tbaa_root(self):
        """TBAA階層のルート"""
        return self.module.add_metadata([
            ir.MetaDataString(self.module, "nyash-tbaa")
        ])

    def create_box_type(self, box_name):
        """Box型ごとのTBAAノード"""
        if box_name not in self.box_types:
            self.box_types[box_name] = self.module.add_metadata([
                ir.MetaDataString(self.module, box_name),
                self.root,
                ir.IntType(64)(0)  # offset
            ])
        return self.box_types[box_name]

    def annotate_load(self, load_inst, box_name):
        """load命令にTBAAメタデータ付与"""
        tbaa_node = self.create_box_type(box_name)
        load_inst.set_metadata("tbaa", tbaa_node)

# 使用例
tbaa = TBAABuilder(module)

# StringBoxへのアクセス
str_load = builder.load(string_ptr)
tbaa.annotate_load(str_load, "StringBox")

# IntegerBoxへのアクセス
int_load = builder.load(integer_ptr)
tbaa.annotate_load(int_load, "IntegerBox")
```

**効果**:
```python
# TBAA情報があると、LLVMはこう最適化できる:

# ① 整数を読む
int_val = load(integer_box)

# ② 文字列を書き込む（IntegerBoxとは無関係！）
store(string_val, string_box)

# ③ もう一度同じ整数を読む
# → LLVMは「②の書き込みは①の整数に影響しない」と判断
# → ③の再読み込みを省略！ int_val を再利用
int_val_reused = int_val  # load削減！
```

---

### **4. 関数属性（Function Attributes）** 🏷️

関数の「性格」をLLVMに伝える。

#### **主要属性**

| 属性 | 意味 | 対象関数例 |
|-----|------|-----------|
| `readnone` | 副作用なし・メモリ読まない | Math.add, Math.sqrt |
| `readonly` | 読み込みのみ・書き込みなし | Array.length, String.length |
| `nounwind` | 例外を投げない | すべてのnyash関数 |
| `alwaysinline` | 常にインライン展開 | 小さいヘルパー関数 |
| `noinline` | インライン展開禁止 | デバッグ用関数 |
| `cold` | めったに実行されない | エラーハンドラ |

#### **実装例**

```python
def annotate_pure_function(func):
    """純粋関数の最適化"""
    # 副作用なし → LLVMが同じ引数の呼び出しを1回に最適化
    func.attributes.add("readnone")
    func.attributes.add("nounwind")

    # 小さい関数は積極的にインライン化
    if count_instructions(func) < 10:
        func.attributes.add("alwaysinline")

# 使用例
math_add_func = module.get_function("Math.add")
annotate_pure_function(math_add_func)

# ✅ 最適化結果:
# result1 = Math.add(1, 2)  # → 実行される
# result2 = Math.add(1, 2)  # → LLVMが省略！result1を再利用
```

---

## 🚨 **注意点（絶対守るべきルール）**

### **1. 未定義動作（UB）を絶対に避ける** ⚠️

LLVMは未定義動作があると**とんでもないコードを生成する**ことがある！

#### **主要UB一覧**

| UB | 例 | 対策 |
|---|---|------|
| nullポインタ参照 | `*null_ptr` | null check挿入 |
| 符号付きオーバーフロー | `INT_MAX + 1` | チェック付き演算 |
| 未初期化変数使用 | `int x; return x;` | 明示的初期化 |
| 配列境界外アクセス | `arr[length]` | 境界チェック |

#### **実装例: null check**

```python
def safe_load(ptr, name="load"):
    """null check付きload"""
    # null check
    is_null = builder.icmp_unsigned('==', ptr, ir.Constant(ptr.type, None))

    with builder.if_then(is_null):
        # nullならエラー
        builder.call(panic_func, [
            ir.Constant.literal_struct([
                ir.Constant(i8p, "Null pointer dereference")
            ])
        ])

    # 安全にload
    return builder.load(ptr, name=name)
```

---

### **2. データレイアウト・呼び出し規約の統一** 📐

プラットフォーム別の標準ルールに従う。

```python
# ターゲット別のデータレイアウト
TARGET_LAYOUTS = {
    "x86_64-unknown-linux-gnu":
        "e-m:e-i64:64-f80:128-n8:16:32:64-S128",
    "wasm32-unknown-unknown":
        "e-m:e-p:32:32-i64:64-n32:64-S128",
    "aarch64-unknown-linux-gnu":
        "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
}

# モジュールに設定
module.triple = target_triple
module.data_layout = TARGET_LAYOUTS[target_triple]
```

---

### **3. PHI命令は慎重に** 🔀

全ての前任ブロックからの値を漏れなく指定！

```python
# ✅ 正しいPHI
phi = builder.phi(i64, name="phi_result")
phi.add_incoming(value_from_if, if_block)
phi.add_incoming(value_from_else, else_block)

# ❌ 間違い: else_blockからの値を忘れた
phi = builder.phi(i64, name="phi_result")
phi.add_incoming(value_from_if, if_block)
# → Verifierエラー！
```

---

### **4. Verifierを毎回実行** ✅

開発中は関数を1つ生成するたびに検証！

```python
def build_function(func_name, ...):
    # 関数生成
    func = ir.Function(module, func_type, name=func_name)
    # ... IRを生成 ...

    # 必ず検証！
    if not func.verify():
        raise CompilerError(f"Invalid IR in {func_name}")

    return func
```

---

## 📊 **期待される効果（ベンチマーク予測）**

| 最適化手法 | 期待される効果 |
|-----------|-------------|
| LLVM Intrinsics | メモリ操作: 5-10倍高速化 |
| Box構造明示 | メソッド呼び出し: 10-100倍高速化 |
| TBAA | ループ内load削減: 20-50% |
| 関数属性 | 関数呼び出しオーバーヘッド: 50-90%削減 |

**総合効果**: 現在比で **2-5倍の性能向上**を予測

---

## 🎯 **実装優先順位**

1. **Verifier統合** 🔴 最優先
2. **UBチェック** 🔴 最優先
3. **データレイアウト標準化** 🟡 重要
4. **Box構造明示** 🟡 重要
5. **LLVM Intrinsics** 🟢 中
6. **関数属性** 🟢 中
7. **TBAA** 🔵 低（効果大だが実装複雑）

---

## 📚 **参考資料**

- [LLVM Language Reference Manual](https://llvm.org/docs/LangRef.html)
- [LLVM Alias Analysis Infrastructure](https://llvm.org/docs/AliasAnalysis.html)
- [LLVM Function Attributes](https://llvm.org/docs/LangRef.html#function-attributes)
- [LLVM Intrinsics](https://llvm.org/docs/LangRef.html#intrinsic-functions)

---

**まとめ**: Geminiの提案は**実用的で効果的**！Phase 21で段階的に実装していく価値がある 🎊
