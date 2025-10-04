# Rune System Design (Phase 23+)

**作成日**: 2025-10-03
**ステータス**: 提案段階（Phase 15完了後に検討）
**関連Issue**: Phase 15セルフホスティング完了後の型システム拡張

---

## 🎯 概要

**Rune-First設計**：継承ではなく、能力（Capability）ベースの型システムを導入する提案。

**Rune（ルーン）**: 箱に刻まれる能力の印。不変の契約として複数組み合わせ可能。

### なぜRuneか？

ChatGPTとの議論で出た「BaseValueBox（暗黙の基底クラス）」案は、設計として優れているが、**継承ではなくRuneとして実装すべき**という結論に至った。

**語源的シナジー**: Hakorune = Hako（箱）+ Rune（ルーン）→ 箱に能力を刻む

**理由**:
- Everything is Box 哲学との整合性
- 複数Runeの組み合わせ可能（composition over inheritance）
- 後から拡張可能（箱に新しいRuneを刻める）
- Phase 20-25の型パス（型推論・型検査）への橋渡し

---

## 📚 他言語の事例研究

### 1. Rust `Any` trait - 型消去＋ダウンキャスト

```rust
trait Any: 'static {
    fn type_id(&self) -> TypeId;
}

// ダウンキャスト
let v: &dyn Any = ...;
if let Some(s) = v.downcast_ref::<String>() { ... }
```

**学び**:
- ✅ 継承なしで型判別
- ✅ traitで能力表現
- ✅ Optionで安全ダウンキャスト

**→ Hakoruneが目指すべきモデル**

### 2. Swift Protocol - 構造的型付け

```swift
protocol CustomStringConvertible {
    var description: String { get }
}

protocol Equatable {
    static func == (lhs: Self, rhs: Self) -> Bool
}

// 複数のProtocolを組み合わせ
struct Point: Equatable, CustomStringConvertible { ... }
```

**学び**:
- ✅ composition over inheritance
- ✅ 複数Rune組み合わせ可能
- ✅ 型安全

### 3. Elixir Protocol - 多態性の分離

```elixir
defprotocol String.Chars do
  def to_string(value)
end

defimpl String.Chars, for: Integer do
  def to_string(int), do: Integer.to_string(int)
end
```

**学び**:
- ✅ 型と実装を分離
- ✅ 後から拡張可能
- ✅ 名前衝突なし

### 4. Haskell Type Class - 制約としての型

```haskell
class Eq a where
    (==) :: a -> a -> Bool

class Show a where
    show :: a -> String

-- 複数制約
foo :: (Eq a, Show a) => a -> String
```

**学び**:
- ✅ 型レベルの制約
- ✅ composable
- ✅ 推論可能

---

## 💡 Hakorune的解決策

### Rune構文（提案）

```hako
// Rune定義（能力の刻印）
@rune ValueLike {
    // メタ情報
    type_id(): SymbolBox
    box_id(): BoxId

    // 基本操作
    equals(other: Box): BoolBox
    hash(): U64Box
    debug(): StringBox

    // Capability
    caps(): CapabilitySetBox
    is_serializable(): BoolBox
    is_deterministic(): BoolBox

    // 安全ダウンキャスト
    try_as_string(): OptionBox<StringBox>
    try_as_number(): OptionBox<NumberBox>

    // DataBox変換
    to_data(): ResultBox<DataBox, ErrorBox>
}

// 実装（@deriveで自動生成）
@derive(ValueLike)
static box StringBox {
    // 既存実装...

    // カスタマイズが必要なら上書き
    override debug(): StringBox {
        return "\"" + me.value + "\""
    }
}
```

### 脱糖後（MIR14維持）

```hako
// @derive(ValueLike) は以下に展開
static box StringBox {
    // ... 既存実装

    // 自動生成メソッド
    type_id(): SymbolBox {
        return :string
    }

    equals(other: Box): BoolBox {
        if let Some(s) = other.try_as_string() {
            return me.value == s.value
        }
        return false
    }

    try_as_string(): OptionBox<StringBox> {
        return OptionBox.some(me)
    }

    try_as_number(): OptionBox<NumberBox> {
        return OptionBox.none()
    }
}
```

**MIR14で表現可能**:
- `type_id()` → Const + Return
- `equals()` → Compare + Branch + Call
- `try_as_*()` → Branch + NewBox (Option)

**命令は増えない！**

---

## 🚀 実装案（3段階アプローチ）

### Phase A: プレリュード関数（即効性）

**実装コスト**: 50-100行、1日
**タイミング**: Phase 15完了後すぐ

```hako
// core/prelude/box_protocol.hako

// 型判別
flow box_type_id(b: Box): SymbolBox { ... }
flow box_kind(b: Box): SymbolBox { ... }

// 安全ダウンキャスト
flow try_as_string(b: Box): OptionBox<StringBox> {
    if box_type_id(b) == :string {
        return OptionBox.some(b as StringBox)
    }
    return OptionBox.none()
}

// Capability判定
flow is_serializable(b: Box): BoolBox { ... }
flow is_deterministic(b: Box): BoolBox { ... }

// 等値・ハッシュ
flow box_equals(a: Box, b: Box): BoolBox { ... }
flow box_hash(b: Box): U64Box { ... }
```

**メリット**:
- ✅ すぐ使える
- ✅ Rust VM変更不要
- ✅ セルフホスト実装の素材になる

**デメリット**:
- ⚠️ メソッド構文が使えない（`b.type_id()`ではなく`box_type_id(b)`）

### Phase B: Rune構文（中期）

**実装コスト**: 200-300行、1週間
**タイミング**: Phase 20-25（型システム拡張期）

```hako
@rune ValueLike {
    type_id(): SymbolBox
    equals(other: Box): BoolBox
    // ...
}

// Runeを実装することを宣言
implements ValueLike for StringBox
implements ValueLike for NumberBox
```

**脱糖**:
- `implements`はコンパイル時チェックのみ
- 実装はプレリュード関数を呼ぶ

### Phase C: @deriveマクロ（長期）

**実装コスト**: 500-700行、2週間
**タイミング**: Phase 25+（マクロシステム完成後）

```hako
@derive(ValueLike, Debug, Hash, Serialize)
static box CustomBox { ... }
```

完全版の実装。

---

## 📊 継承 vs Rune の比較

| 観点 | 継承（BaseValueBox） | Rune（ValueLike） |
|------|---------------------|---------------------|
| **哲学整合性** | ⚠️ is-a関係（階層的） | ✅ can-do関係（水平的） |
| **拡張性** | ❌ 単一継承の制約 | ✅ 複数Rune組み合わせ |
| **後方互換性** | ⚠️ 基底変更が全体に影響 | ✅ Rune追加は影響なし |
| **実装の自由度** | ❌ 実装の強制 | ✅ デフォルト実装＋上書き可 |
| **Phase 20-25連携** | ⚠️ 型パスとの整合が難しい | ✅ 型制約として自然に統合 |
| **Everything is Box** | ⚠️ 階層構造との矛盾 | ✅ Box の能力として自然 |

---

## 🎯 Rust VM との整合性

### NyashValue との対応

```rust
// Rust VM側（Phase 15の実装）
impl NyashValue {
    fn type_id(&self) -> Symbol { ... }
    fn equals(&self, other: &Self) -> bool { ... }
    fn hash(&self) -> u64 { ... }
}

// Hakorune側（Phase 23+の実装）
@rune ValueLike {
    type_id(): SymbolBox
    equals(other: Box): BoolBox
    hash(): U64Box
}
```

**メソッド名を揃えるだけで整合！継承不要！**

FFI境界で自動変換:
```
Rust NyashValue → Hakorune Rune実装
```

---

## 🏗️ Phase 15との関係

### ⚠️ 重要な制約

**Phase 15の原則**:
- ✅ Rust VM = 捨てる前提
- ✅ .hako実装 = 本体
- ❌ Rust VMを複雑にしたら二重実装の地獄

**Rune設計の実装タイミング**:
1. **Phase 15中**: 何もしない（セルフホスティングに集中）
2. **Phase 15完了後**: プレリュード関数（Phase A）を実装
3. **Phase 20-25**: Rune構文（Phase B）を実装
4. **Phase 25+**: @deriveマクロ（Phase C）を実装

---

## 📝 DataBox / MapBox v2 との連携

### DataBox統合

```hako
@rune ValueLike {
    // ...
    to_data(): ResultBox<DataBox, ErrorBox>
}

// すべてのBoxが純データ表現を返せる
static box StringBox {
    to_data(): ResultBox<DataBox, ErrorBox> {
        return Ok(DataBox.of_string(me))
    }
}
```

### MapBox v2統合

```hako
@derive(ValueLike)
static box MapBox {
    // root_id/provenance/caps/hash を基底から継承
    // COW/Shared/Foreignでもroot_id()は不変
}
```

---

## 🎊 Hakorune哲学との整合性チェック

### Everything is Box ✅
- Rune は Box の**能力**を表現
- 継承階層ではなく**水平的な能力の組み合わせ**

### Plugin-First ✅
- プレリュード関数は`extern_call`経由
- プラグインもRune実装可能

### Deterministic ✅
- `is_deterministic()`で判定可能
- Capability として明示

### De-sugaring Contract ✅
- Runeは既存のMIR14に脱糖
- 命令は増えない

### Fail-Fast ✅
- ダウンキャストは`Option/Result`
- panicではなく明示的失敗

---

## 🔬 研究課題

### 1. SchemaBox / ProofBox 連携

```hako
// 実行時の形状検証
let schema: SchemaBox = schema_from_string("{ instructions: array, ... }");
let proof: ProofBox = schema.check(v)?; // v: DataBox

// 以後、この v は schema 準拠として扱える（静的パスに渡せる）
use_verified(v, proof);
```

### 2. 型推論との連携

```hako
// Rune制約
flow process<T: ValueLike>(v: T) {
    print(v.type_id())  // コンパイル時チェック
}
```

### 3. 複数Rune組み合わせ

```hako
@rune Serializable { to_data(): DataBox }
@rune Comparable { compare(other: Box): IntegerBox }
@rune Hashable { hash(): U64Box }

// 複数組み合わせ
implements Serializable, Comparable, Hashable for CustomBox
```

---

## 📚 参考文献

- [Rust Any trait](https://doc.rust-lang.org/std/any/trait.Any.html)
- [Swift Protocols](https://docs.swift.org/swift-book/LanguageGuide/Protocols.html)
- [Elixir Protocols](https://elixir-lang.org/getting-started/protocols.html)
- [Haskell Type Classes](https://www.haskell.org/tutorial/classes.html)

---

## 🎯 次のアクション

1. Phase 15完了を待つ
2. プレリュード関数（Phase A）の仕様を詳細化
3. Rune構文（Phase B）のパーサー設計
4. @deriveマクロ（Phase C）の脱糖ルール策定

---

**最終更新**: 2025-10-03
**レビュー**: ChatGPT o1 + Claude Sonnet 4.5
**承認待ち**: Phase 15完了後に再検討
