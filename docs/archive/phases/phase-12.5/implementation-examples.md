# Phase 12.5: 実装例集

## 1. MIRMetadataの具体例

### StringBox.lengthのヒント付与

```rust
// Before（ヒントなし）
BoxCall {
    target: ValueId(0),
    method: "length",
    args: vec![],
}

// After（ヒント付き）
BoxCall {
    target: ValueId(0),
    method: "length",
    args: vec![],
    metadata: MIRMetadata {
        pure: true,        // 同じ文字列→同じ長さ
        readonly: true,    // 文字列を変更しない
        nothrow: true,     // 例外を投げない
        noalias: true,     // 結果は新しい整数
        ..Default::default()
    },
}
```

### ループの最適化ヒント

```nyash
// Nyashコード
for i in 0..1000 {
    array.push(i * 2)
}
```

```rust
// MIRでのヒント
Branch {
    cond: ValueId(5),  // i < 1000
    then_block: BlockId(2),
    else_block: BlockId(3),
    metadata: MIRMetadata {
        likely: Some(true),     // ループ継続が高確率
        loop_count: Some(1000), // ループ回数ヒント
        ..Default::default()
    },
}
```

## 2. Cエミッタでの変換例

### 純粋関数の最適化

```c
// MIR: BoxCall(StringBox, "length") with {pure: true}
// ↓
// C出力:
static inline int64_t __attribute__((pure)) 
ny_string_length(NyashHandle h) {
    NyashString* s = ny_handle_to_string(h);
    return s->length;
}
```

### 分岐予測の最適化

```c
// MIR: Branch with {likely: Some(false)}
// ↓
// C出力:
if (__builtin_expect(!!(error_condition), 0)) {
    // エラー処理（めったに実行されない）
    ny_handle_error();
} else {
    // 通常処理（ほぼ常に実行）
    continue;
}
```

## 3. 最適化パスの実装例

### 定数畳み込み（ConstFoldingPass）

```rust
impl OptPass for ConstFoldingPass {
    fn run(&self, mir: &mut MIR) -> bool {
        let mut changed = false;
        
        for block in &mut mir.blocks {
            for inst in &mut block.instructions {
                match inst {
                    // Const(3) + Const(5) → Const(8)
                    BinOp { op: Add, left, right, result } => {
                        if let (Some(a), Some(b)) = (
                            self.get_const_value(*left),
                            self.get_const_value(*right)
                        ) {
                            *inst = Const { 
                                value: Value::Integer(a + b),
                                result: *result 
                            };
                            changed = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        changed
    }
}
```

### デッドコード除去（DeadCodeElimPass）

```rust
impl OptPass for DeadCodeElimPass {
    fn run(&self, mir: &mut MIR) -> bool {
        // 1. 使用されている値を収集
        let used = self.collect_used_values(mir);
        
        // 2. 未使用の命令を削除
        let mut changed = false;
        for block in &mut mir.blocks {
            block.instructions.retain(|inst| {
                if let Some(result) = inst.get_result() {
                    if !used.contains(&result) && !inst.has_side_effects() {
                        changed = true;
                        return false; // 削除
                    }
                }
                true
            });
        }
        changed
    }
}
```

## 4. バックエンド別の出力例

### zig cc向け（Ubuntu/macOS）

```bash
# MIRからCへ変換
nyash --emit-c program.nyash -o program.c

# 最適化コンパイル
zig cc -O3 -flto -march=native \
    -fno-plt \
    -fomit-frame-pointer \
    program.c nyrt.c \
    -o program
```

### MSVC向け（Windows）

```batch
REM リンク時最適化を有効化
cl /O2 /GL /MD program.c nyrt.c /Fe:program.exe ^
   /link /LTCG /OPT:REF /OPT:ICF
```

### プロファイルガイド最適化（PGO）

```bash
# Step 1: プロファイル収集
zig cc -O3 -fprofile-generate program.c -o program_prof
./program_prof < typical_input.txt

# Step 2: プロファイルを使用して再コンパイル
zig cc -O3 -fprofile-use program.c -o program_opt
```

## 5. 性能測定の例

```nyash
// benchmark.nyash
static box Benchmark {
    main() {
        local start, end, result
        
        // ウォームアップ
        me.fibonacci(20)
        
        // 測定開始
        start = Time.now()
        result = me.fibonacci(40)
        end = Time.now()
        
        print("Fibonacci(40) = " + result)
        print("Time: " + (end - start) + "ms")
    }
    
    @[pure, nothrow]  // 最適化ヒント
    fibonacci(n) {
        if n <= 1 {
            return n
        }
        return me.fibonacci(n - 1) + me.fibonacci(n - 2)
    }
}
```

## 6. 最適化レベルごとの比較

| レベル | MIR最適化 | Cコンパイラ | 想定性能 | 用途 |
|--------|-----------|-------------|----------|------|
| 0 | なし | -O0 | 10% | デバッグ |
| 1 | 基本 | -O2 | 50% | 開発 |
| 2 | 全て | -O3 -flto | 70% | リリース |
| 3 | 全て+PGO | -O3 -flto -fprofile-use | 85% | 高性能 |
| 4 | 全て | LLVM -O3 | 90%+ | 特殊用途 |

*性能は理論上の最大性能を100%とした場合の目安