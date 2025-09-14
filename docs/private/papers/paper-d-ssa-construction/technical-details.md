# SSA構築の技術詳細

## 1. Nyash特有のSSA課題

### 1.1 Box型システムとSSA
```nyash
// Nyashコード
local str = "hello"
local num = 42
local result = str + num  // 動的な型
```

```llvm
; LLVM IRでの課題
%str = call i64 @nyash_string_new(i8* @.str.hello)  ; handle
%num = i64 42
%result = ?  ; concat_si? concat_ii? 実行時まで不明
```

### 1.2 PHI型の決定問題
```llvm
; 複雑な合流での型推論
bb1:
  %val1 = i64 123          ; integer handle
  br label %merge
bb2:
  %val2 = i8* @string      ; string pointer
  %handle = ptrtoint i8* %val2 to i64
  br label %merge
merge:
  %phi = phi ??? [ %val1, %bb1 ], [ %handle, %bb2 ]
  ; i64? i8*? 文脈依存で決定が必要
```

## 2. BuilderCursor設計の詳細

### 2.1 問題：位置管理の複雑さ
```rust
// 悪い例：グローバルなbuilder状態
builder.position_at_end(bb1);
emit_instructions();
builder.position_at_end(bb2);  // 位置が変わる！
// bb1の続きを書きたいが...？
```

### 2.2 解決：BuilderCursor
```rust
pub struct BuilderCursor<'ctx, 'b> {
    builder: &'b Builder<'ctx>,
    closed_by_bid: HashMap<BasicBlockId, bool>,
    cur_bid: Option<BasicBlockId>,
    cur_llbb: Option<BasicBlock<'ctx>>,
}

impl BuilderCursor {
    pub fn with_block<R>(&mut self, bid, bb, f: impl FnOnce(&mut Self) -> R) -> R {
        // 状態を保存
        let prev = (self.cur_bid, self.cur_llbb);
        self.at_end(bid, bb);
        let result = f(self);
        // 状態を復元
        (self.cur_bid, self.cur_llbb) = prev;
        result
    }
}
```

### 2.3 終端管理
```rust
pub fn emit_term(&mut self, bid: BasicBlockId, f: impl FnOnce(&Builder)) {
    self.assert_open(bid);  // 閉じたブロックへの挿入を防止
    f(self.builder);
    self.closed_by_bid.insert(bid, true);  // 明示的に閉じる
}
```

## 3. Sealed SSAの実装

### 3.1 従来のアプローチ（問題あり）
```rust
// emit_jump/branchで即座にPHI配線
if let Some(phis) = phis_by_block.get(target) {
    for (dst, phi, inputs) in phis {
        // predからの値をその場で配線
        let val = vmap.get(vid)?;  // でも値がまだない場合も...
        phi.add_incoming(&[(val, pred_bb)]);
    }
}
```

### 3.2 Sealed SSAアプローチ
```rust
// ブロック終了時にスナップショット
let mut block_end_values: HashMap<BlockId, HashMap<ValueId, Value>> = HashMap::new();

// 各ブロック降下後
let snapshot = vmap.iter()
    .filter(|(vid, _)| defined_in_block.contains(vid))
    .map(|(k, v)| (*k, *v))
    .collect();
block_end_values.insert(bid, snapshot);

// seal時にスナップショットから配線
fn seal_block(...) {
    let val = block_end_values[&pred_bid].get(&vid)
        .or_else(|| /* フォールバック */);
}
```

### 3.3 PHI正規化の課題
```rust
// 理想：pred数 = incoming数
assert_eq!(phi.count_incoming(), preds.get(&bb).len());

// 現実：MIR PHIとCFG predsの不一致
// - MIRは静的に決定
// - CFGは動的に変化（最適化、終端追加など）
```

## 4. 型変換の統一戦略

### 4.1 基本方針
```llvm
; すべてのBox値はi64 handleとして統一
; 必要な箇所でのみptr変換

; 原則
%handle = i64 ...
%ptr = inttoptr i64 %handle to i8*  ; 必要時のみ

; PHIも原則i64
%phi = phi i64 [...], [...]
```

### 4.2 文字列処理の特殊性
```llvm
; 文字列リテラル
%str_ptr = getelementptr [6 x i8], [6 x i8]* @.str.hello, i32 0, i32 0
%handle = call i64 @nyash_string_new(i8* %str_ptr)

; 文字列操作（handleベース）
%len = call i64 @nyash.string.len_h(i64 %handle)
%sub = call i64 @nyash.string.substring_hii(i64 %handle, i64 %start, i64 %end)
```

## 5. デバッグとトレース

### 5.1 環境変数による制御
```bash
NYASH_CLI_VERBOSE=1          # 基本ログ
NYASH_LLVM_TRACE_PHI=1       # PHI配線の詳細
NYASH_LLVM_PHI_SEALED=1      # Sealed SSAモード
NYASH_ENABLE_LOOPFORM=1      # LoopForm実験
```

### 5.2 診断出力の例
```
[PHI:new] fn=Main_esc_json_1 bb=30 dst=30 ty=i64 inputs=(23->7),(27->30)
[PHI] sealed add pred_bb=27 val=30 ty=i64 (snapshot)
[PHI] sealed add (synth) pred_bb=23 zero-ty=i64
[LLVM] terminator present for bb=27
[LoopForm] detect while-pattern: header=15 body=16 other=17
```

## 6. 未解決の技術課題

### 6.1 完全なDominance保証
- 現状：hoistingとentry block配置で部分対応
- 課題：ループ内での循環参照
- 将来：LoopFormでの構造化解決

### 6.2 最適PHI配置
- 現状：MIR指定の場所に素直に配置
- 課題：冗長なPHIの削減
- 将来：PHI最小化アルゴリズム

### 6.3 例外安全性
- 現状：ゼロ値合成でクラッシュ回避
- 課題：意味的正確性の保証
- 将来：Box型システムでのnull安全性

## 7. 箱理論による革命的簡略化

### 7.1 実装アーキテクチャ
```python
class BoxBasedSSA:
    def __init__(self):
        self.boxes = {}      # block_id -> {var: value}
        self.current_box = {}
        self.deferred_phis = []  # 後処理用
```

### 7.2 PHI処理の簡略化
```python
# 従来：複雑なdominance計算とキャッシュ
def resolve_phi_complex(self, phi_info):
    # 300行のResolver処理...
    # dominance確認、型変換、キャッシュ管理

# 箱理論：単純な値選択
def resolve_phi_simple(self, var, predecessors):
    for pred_id, _ in predecessors:
        if self.came_from(pred_id):
            return self.boxes[pred_id].get(var, 0)
    return 0
```

### 7.3 alloca/load/store方式への転換
```python
# SSA形式を諦めて、メモリベースの実装
def emit_variable_access(self, var):
    if var not in self.allocas:
        # 変数用のメモリ確保
        self.allocas[var] = self.builder.alloca(self.i64, name=var)
    
    # 読み込み
    def load_var():
        return self.builder.load(self.allocas[var])
    
    # 書き込み
    def store_var(value):
        self.builder.store(value, self.allocas[var])
```

### 7.4 型システムの単純化
```python
# すべてをi64として扱う
def to_i64(self, value):
    if is_pointer(value):
        # ポインタ→ハンドル変換
        return self.call_from_i8_string(value)
    elif is_integer(value):
        return value
    else:
        return 0  # デフォルト

# 必要時のみポインタ変換
def to_ptr_if_needed(self, value, context):
    if context == "console_log":
        return self.call_to_i8p_h(value)
    return value
```

### 7.5 パフォーマンス特性
```
従来のSSA実装:
- コンパイル時間: 遅い（PHI配線で50分）
- 実行時性能: 最適
- メモリ使用: 少ない

箱理論実装:
- コンパイル時間: 高速（5分以内）
- 実行時性能: やや遅い（alloca/load/storeのオーバーヘッド）
- メモリ使用: やや多い（変数ごとにalloca）

トレードオフ: "動かないより100倍マシ"
```

### 7.6 実装の段階的移行
```python
# Phase 1: 最小動作確認（現在）
- allocaベースで全変数管理
- PHI完全スキップ
- 動作優先

# Phase 2: 部分的最適化（将来）
- 読み取り専用変数はSSA
- ループ変数のみalloca
- 段階的性能改善

# Phase 3: 完全最適化（長期）
- 箱理論の知見を活かしたSSA再実装
- 100行のシンプルさを維持
```

---

*これらの技術詳細は、論文の Technical Section の基礎となる。箱理論により、理論的な美しさより実装の実用性を優先した新しいアプローチを示している。*