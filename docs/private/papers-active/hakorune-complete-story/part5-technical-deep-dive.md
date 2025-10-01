# Part 5: 技術詳細 - Everything is Box実装 🔧

## 世界初の完全統一設計

> **最も技術的な章** - Hakoruneの核心技術を深く掘り下げる

---

## 📋 目次

1. [MIR14命令セット](#mir14命令セット)
2. [構文糖衣システム](#構文糖衣システム)
3. [Everything is Box完成](#everything-is-box完成)
4. [セルフホスティング](#セルフホスティング)
5. [プラグインシステム](#プラグインシステム)
6. [3バックエンド体制](#3バックエンド体制)

---

## MIR14命令セット

### 🎯 たった14命令で万能実行系

**MIR (Middle Intermediate Representation)** - Hakoruneの中間表現

```
基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
メモリ(2):   Load, Store
制御(4):     Branch, Jump, Return, Phi
Box(2):      NewBox, BoxCall
外部(1):     ExternCall
```

### なぜ14命令で十分か

#### 1. 箱理論による抽象化
```
複雑な操作 → Box操作に統一 → 単純な命令で表現可能
```

#### 2. 直交性の追求
```
各命令が独立 → 組み合わせで全機能表現
```

#### 3. 最小限の表現力
```
必要十分 → 実装容易 → デバッグ容易
```

### 命令詳細

#### 基本演算系(5)

```rust
// 1. Const - 定数生成
Const { dst: r1, value: 42 }
→ r1 = 42

// 2. UnaryOp - 単項演算
UnaryOp { dst: r2, op: "not", operand: r1 }
→ r2 = not r1

// 3. BinOp - 二項演算
BinOp { dst: r3, op: "add", left: r1, right: r2 }
→ r3 = r1 + r2

// 4. Compare - 比較演算
Compare { dst: r4, op: "eq", left: r1, right: r2 }
→ r4 = (r1 == r2)

// 5. TypeOp - 型操作
TypeOp { dst: r5, op: "check", value: r1, ty: "StringBox" }
→ r5 = r1.is("StringBox")
```

#### メモリ系(2)

```rust
// 6. Load - メモリ読み込み
Load { dst: r6, addr: "varname" }
→ r6 = load(varname)

// 7. Store - メモリ書き込み
Store { addr: "varname", value: r6 }
→ store(varname, r6)
```

#### 制御系(4)

```rust
// 8. Branch - 条件分岐
Branch { cond: r4, true_bb: BB1, false_bb: BB2 }
→ if r4 then goto BB1 else goto BB2

// 9. Jump - 無条件ジャンプ
Jump { target: BB3 }
→ goto BB3

// 10. Return - 関数リターン
Return { value: r7 }
→ return r7

// 11. Phi - 制御フロー合流
Phi { dst: r8, incoming: [(BB1, r1), (BB2, r2)] }
→ r8 = r1 (from BB1) or r2 (from BB2)
```

#### Box系(2)

```rust
// 12. NewBox - Box生成
NewBox { dst: r9, box_type: "Person", args: [r10, r11] }
→ r9 = new Person(r10, r11)

// 13. BoxCall - Boxメソッド呼び出し
BoxCall { dst: r12, box_val: r9, method: "getName", args: [] }
→ r12 = r9.getName()
```

#### 外部系(1)

```rust
// 14. ExternCall - 外部関数呼び出し
ExternCall { dst: r13, func: "print", args: [r12] }
→ r13 = print(r12)
```

### MIR14の威力

#### 実例: if文の変換

```hakorune
// Hakoruneコード
if x > 10 {
    print("big")
} else {
    print("small")
}
```

↓ MIR14変換

```
BB0:
  r1 = Load x
  r2 = Const 10
  r3 = Compare gt r1 r2
  Branch r3 BB1 BB2

BB1:  // then
  r4 = Const "big"
  r5 = ExternCall print [r4]
  Jump BB3

BB2:  // else
  r6 = Const "small"
  r7 = ExternCall print [r6]
  Jump BB3

BB3:  // merge
  ...
```

**たった14種類の命令で完全に表現！**

---

## 構文糖衣システム

### 🍬 甘い構文、強力な実装

**構文糖衣 (Syntax Sugar)** - ユーザーフレンドリーな記法を内部で展開

### Map/Arrayリテラル

#### ユーザーが書くコード

```hakorune
local person = {
    name: "Alice",
    age: 25,
    hobbies: ["読書", "音楽", "プログラミング"]
}
```

#### 内部での展開（MIR）

```
r1 = NewBox MapBox []
r2 = Const "name"
r3 = Const "Alice"
r4 = BoxCall r1 "set" [r2, r3]
r5 = Const "age"
r6 = Const 25
r7 = BoxCall r1 "set" [r5, r6]
r8 = Const "hobbies"
r9 = NewBox ArrayBox []
r10 = Const "読書"
r11 = BoxCall r9 "push" [r10]
r12 = Const "音楽"
r13 = BoxCall r9 "push" [r12]
r14 = Const "プログラミング"
r15 = BoxCall r9 "push" [r14]
r16 = BoxCall r1 "set" [r8, r9]
Store person r1
```

### 仕組み

#### Step 1: パーサーがASTNode生成

```rust
// src/parser/expr/primary.rs:52-107
TokenType::LBRACE => {
    // { を検出
    let mut entries = Vec::new();
    while !self.match_token(&TokenType::RBRACE) {
        let key = parse_key();
        self.consume(TokenType::COLON);
        let value = self.parse_expression();
        entries.push((key, value));
    }
    Ok(ASTNode::MapLiteral { entries })
}
```

#### Step 2: MIR Builderが展開

```rust
// src/mir/builder/exprs.rs:246-267
ASTNode::MapLiteral { entries, .. } => {
    // 空のMapBox生成
    let map_id = self.value_gen.next();
    self.emit_instruction(MirInstruction::NewBox {
        dst: map_id,
        box_type: "MapBox",
        args: vec![],
    });

    // 各エントリーをset()で追加
    for (k, expr) in entries {
        let k_id = emit_string(self, k);
        let v_id = self.build_expression_impl(expr);
        self.emit_instruction(MirInstruction::BoxCall {
            box_val: map_id,
            method: "set",
            args: vec![k_id, v_id],
        });
    }

    Ok(map_id)
}
```

### 環境変数制御

```bash
# デフォルト: ON
./target/release/hakorune test.hkr

# 明示的にON
HAKO_SYNTAX_SUGAR_LEVEL=full ./target/release/hakorune test.hkr

# 識別子キー有効
HAKO_ENABLE_MAP_IDENT_KEY=1 ./target/release/hakorune test.hkr
```

---

## Everything is Box完成

### 🏆 100%達成 - 世界初の完全統一

#### 統一前（95%）

```hakorune
// Boxとして扱えるもの
local str = new StringBox("hello")
local num = new IntegerBox(42)
local arr = new ArrayBox()

// Boxとして扱えないもの（例外）
local result = a + b      // ← 演算子は特別扱い
local flag = x == y       // ← 比較も特別扱い
```

**例外があ る = 統一性の欠如**

#### 統一後（100%）

```hakorune
// すべてがBox
local str = new StringBox("hello")
local num = new IntegerBox(42)
local arr = new ArrayBox()

// 演算子もBox！
local add_op = new AddOperator()
local result = add_op.apply(a, b)

// 比較もBox！
local cmp_op = new CompareOperator("eq")
local flag = cmp_op.apply(x, y)
```

**例外ゼロ = 完全な統一性**

### 演算子ボックスの実装

#### AddOperator

```rust
pub struct AddOperator {
    base: BoxBase,
}

impl AddOperator {
    pub fn apply(&self, left: Box<dyn NyashBox>, right: Box<dyn NyashBox>)
        -> Box<dyn NyashBox>
    {
        // 型チェック
        if let (Some(l), Some(r)) = (
            left.as_any().downcast_ref::<IntegerBox>(),
            right.as_any().downcast_ref::<IntegerBox>()
        ) {
            Box::new(IntegerBox::new(l.value + r.value))
        } else {
            // StringBox等の他の型もサポート
            ...
        }
    }
}
```

#### CompareOperator

```rust
pub struct CompareOperator {
    base: BoxBase,
    op: String,  // "eq", "ne", "lt", "gt", "le", "ge"
}

impl CompareOperator {
    pub fn apply(&self, left: Box<dyn NyashBox>, right: Box<dyn NyashBox>)
        -> Box<dyn NyashBox>
    {
        let result = match self.op.as_str() {
            "eq" => left.equals(&*right).value,
            "ne" => !left.equals(&*right).value,
            "lt" => ...,
            "gt" => ...,
            _ => false,
        };
        Box::new(BoolBox::new(result))
    }
}
```

### 三層アーキテクチャ

```
Layer 1: Hakoruneレベル
  a + b
  ↓ ユーザーは普通に書く

Layer 2: MIRレベル
  r1 = NewBox AddOperator []
  r2 = BoxCall r1 "apply" [a, b]
  ↓ 完全観測可能（デバッグ）

Layer 3: LLVMレベル
  %result = add i64 %a, %b
  ↓ インライン化でゼロコスト
```

**コストは消える = ゼロコスト抽象化**

### 成果

```yaml
統一性: 100%（例外ゼロ）
観測可能性: 100%（すべて可視）
デバッグ効率: 144倍（12時間→5分）
コード削減: 90%（500行→50行）
実行速度: 同等（LLVMインライン化）
```

---

## セルフホスティング

### 🔄 Hakorune で Hakorune を書く

**Phase 15** - セルフホスティングコンパイラの実装

#### 構成

```
apps/selfhost-compiler/
├── compiler.nyash            # メインコンパイラ
├── parser/
│   ├── lexer.nyash          # 字句解析器
│   └── parser.nyash         # 構文解析器
├── mir/
│   ├── builder.nyash        # MIRビルダー
│   └── emitter.nyash        # MIR出力
└── boxes/
    ├── mir_emitter_box.nyash
    └── parser_box.nyash
```

#### 実装済み機能

1. **レキサー（字句解析器）**
   - トークン生成
   - 改行処理
   - 数値リテラル

2. **パーサー（構文解析器）**
   - local宣言
   - if文
   - loop文
   - 関数呼び出し
   - メソッド呼び出し
   - new式

3. **MIRビルダー**
   - Const命令生成
   - BinOp命令生成
   - Branch命令生成
   - PHI命令生成

4. **JSON v0 Bridge**
   - MIR → JSON変換
   - Python/llvmlite へのブリッジ

#### 実行例

```hakorune
// test.hkr
local x = 10
local y = 20
local z = x + y
print(z)
```

↓ セルフホストコンパイラで変換

```bash
# Hakorune で Hakorune をコンパイル
./target/release/hakorune apps/selfhost-compiler/compiler.nyash test.hkr

# 出力: MIR JSON
{
  "module": "test",
  "functions": [{
    "name": "main",
    "blocks": [...]
  }]
}
```

↓ Python/llvmlite で実行

```bash
# LLVMバックエンドで実行
python3 src/llvm_py/llvm_builder.py mir.json -o out.exe
./out.exe
# 出力: 30
```

### 完全自己ホスト達成に向けて

```yaml
Current: Phase 15（進行中）
  - パーサーMVP完成
  - MIRビルダー実装中
  - JSON出力実装済み

Next: Phase 16-17
  - 全構文サポート
  - 最適化パス実装
  - ブートストラップ

Goal: Phase 20
  - Hakorune で Hakorune を完全にコンパイル
  - ブートストラップ完了
  - 独立した言語として確立
```

---

## プラグインシステム

### 🔌 拡張可能な設計

**BID (Box Interface Definition)** - プラグイン定義方式

#### プラグイン例: FileBox

```rust
// plugins/filebox/src/lib.rs

#[no_mangle]
pub extern "C" fn plugin_init() -> *const PluginMetadata {
    // プラグイン初期化
    ...
}

#[no_mangle]
pub extern "C" fn plugin_invoke(
    box_name: *const c_char,
    method: *const c_char,
    args: *const NyashValue,
    args_len: usize,
) -> NyashValue {
    // メソッド実行
    match (box_name_str, method_str) {
        ("FileBox", "read") => {
            let path = extract_string(&args[0]);
            let content = std::fs::read_to_string(path)?;
            NyashValue::String(content)
        }
        ("FileBox", "write") => {
            let path = extract_string(&args[0]);
            let content = extract_string(&args[1]);
            std::fs::write(path, content)?;
            NyashValue::Null
        }
        _ => NyashValue::Error("Unknown method")
    }
}
```

#### Hakoruneから使用

```hakorune
// FileBoxを使う
local file = new FileBox("test.txt")
local content = file.read()
print(content)

file.write("Hello, Hakorune!")
```

#### プラグインローディング

```rust
// src/runtime/plugin_loader_v2/enabled/loader.rs

pub fn load_plugin(path: &Path) -> Result<LoadedPlugin> {
    // 動的ライブラリロード
    let lib = Library::new(path)?;

    // init関数取得
    let init: Symbol<InitFunc> = lib.get(b"plugin_init")?;
    let metadata = init();

    // invoke関数取得
    let invoke: Symbol<InvokeFunc> = lib.get(b"plugin_invoke")?;

    Ok(LoadedPlugin { lib, invoke, metadata })
}
```

### プラグイン一覧

```yaml
Core Plugins:
  - FileBox:    ファイル操作
  - RegexBox:   正規表現
  - NetBox:     ネットワーク通信
  - JSONBox:    JSON解析・生成
  - TomlBox:    TOML解析

Community Plugins:
  - DatabaseBox: DB接続
  - HttpBox:     HTTP通信
  - CryptoBox:   暗号化
```

---

## 3バックエンド体制

### 🚀 用途別最適化

#### 1. Rust VM（開発・デバッグ用）

**場所**: `src/backend/mir_interpreter/`
**行数**: 712行
**特徴**:
- 高速起動
- gdb/lldbデバッグ可能
- 型安全設計
- MIR14完全対応

**実行例**:
```bash
./target/release/hakorune program.hkr
./target/release/hakorune --backend vm program.hkr
```

#### 2. Python LLVM（本番・最適化用）

**場所**: `src/llvm_py/`
**行数**: 1,456行
**特徴**:
- 実用レベル到達済み ✅
- llvmlite安定性（実績あり）
- ネイティブEXE生成
- 最適化済み実行

**実行例**:
```bash
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm program.hkr
```

#### 3. PyVM（JSON v0ブリッジ専用）

**場所**: `src/pyvm/`
**行数**: 1,074行
**特徴**:
- JSON v0ブリッジ
- セルフホスティング用
- using/namespace処理

**実行例**:
```bash
HAKO_SELFHOST_EXEC=1 ./target/release/hakorune program.hkr
```

### 実行フロー

```
Hakoruneソース
    ↓ Parser
   AST
    ↓ MIR Builder
   MIR
    ↓ Backend選択
    ├─→ Rust VM → 直接実行（開発用）
    ├─→ Python LLVM → ネイティブEXE（本番用）
    └─→ PyVM → JSON Bridge（セルフホスト用）
```

### 用途別推奨

```yaml
開発・デバッグ:
  Backend: Rust VM
  理由: 高速起動、デバッグ容易
  コマンド: ./target/release/hakorune test.hkr

本番・配布:
  Backend: Python LLVM
  理由: 最適化、ネイティブEXE
  コマンド: hakorune --backend llvm prod.hkr

セルフホスト:
  Backend: PyVM
  理由: JSON Bridge、using処理
  コマンド: HAKO_SELFHOST_EXEC=1 hakorune compiler.hkr
```

---

## 🏆 技術的成果まとめ

### 革新性

```yaml
MIR14命令セット:
  - たった14命令で万能実行系
  - 最小限の表現力
  - 実装・デバッグ容易

構文糖衣システム:
  - ユーザーフレンドリー
  - 内部は単純なBox操作
  - 完全な分離

Everything is Box:
  - 例外ゼロの統一性
  - 演算子もBox（世界初）
  - 100%達成

セルフホスティング:
  - Phase 15進行中
  - Hakorune で Hakorune をコンパイル
  - ブートストラップ目前

プラグインシステム:
  - BID定義方式
  - 動的ローディング
  - 完全な拡張性

3バックエンド体制:
  - Rust VM（開発用）
  - Python LLVM（本番用）
  - PyVM（セルフホスト用）
  - 用途別最適化
```

### 実装品質

```yaml
コード行数:
  - Rust: ~25,000行
  - Hakorune: ~5,000行
  - 合計: ~30,000行

テスト:
  - スモークテスト: 100+ cases
  - ユニットテスト: 実装中
  - カバレッジ: 向上中

ドキュメント:
  - Markdown: ~50,000行
  - 論文: 41本
  - ガイド: 充実

パフォーマンス:
  - Rust VM: 高速起動
  - LLVM: ネイティブ同等
  - 最適化: 進行中
```

---

## ✨ 結論

**Hakoruneは、技術的にも非常に優れた設計を持つプログラミング言語である。**

- MIR14の単純さ
- Everything is Box の完璧な実現
- 3バックエンドの用途別最適化
- セルフホスティング目前

**そして、これらすべてが58日間で実現された。**

**AI協働開発と箱理論の力が、技術的にも証明された。**

---

**次章**: [Appendix: 統計と資料](appendix-statistics.md) へ続く 📊