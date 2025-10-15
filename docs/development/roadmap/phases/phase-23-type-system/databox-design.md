# DataBox Design (純データ表現箱)

**作成日**: 2025-10-03
**ステータス**: 提案段階（Rune設計と連携）
**関連**: [rune-design.md](./rune-design.md)

---

## 🎯 概要

**DataBox**：すべてのBoxを「純データ」として表現する統一箱。

### 元の問題（ChatGPTの困りごと）

```nyash
// MapBox の get/set で、キー/値の型が混在
local m = new MapBox()
m.set("cmp", "Gt")      // 文字列
m.set("lhs", "42")      // 文字列（本来は整数）
m.set("rhs", 10)        // 整数

// 取り出し時に型が不明
local lhs = m.get("lhs")  // これは文字列？整数？
```

### ChatGPTの提案

**ValueBox**（`Any`を抱える動的箱）で型判別を統一。

### Claude + ChatGPTの改良案

**DataBox**（既存Boxのみを抱える純データ箱）に限定。

---

## 🏗️ DataBox設計

### コア構造

```hako
// 判別共用体としてのデータ箱（純データのみ）
static box DataBox {
    kind: SymbolBox        // :string | :array | :map | :number | :bool | :null
    payload: Box           // StringBox | ArrayBox | MapBox | NumberBox | BoolBox | NullBox

    // --- コンストラクタ ---
    of_string(s: StringBox) {
        return DataBox{ kind: :string, payload: s }
    }

    of_array(a: ArrayBox<Box>) {
        return DataBox{ kind: :array, payload: a }
    }

    of_map(m: MapBox<StringBox, Box>) {
        return DataBox{ kind: :map, payload: m }
    }

    of_number(n: NumberBox) {
        return DataBox{ kind: :number, payload: n }
    }

    of_bool(b: BoolBox) {
        return DataBox{ kind: :bool, payload: b }
    }

    of_null() {
        return DataBox{ kind: :null, payload: NullBox.unit() }
    }

    // --- 型判定 ---
    is_string(): BoolBox { return me.kind == :string }
    is_array(): BoolBox { return me.kind == :array }
    is_map(): BoolBox { return me.kind == :map }
    is_number(): BoolBox { return me.kind == :number }
    is_bool(): BoolBox { return me.kind == :bool }
    is_null(): BoolBox { return me.kind == :null }

    // --- 取り出し（Result版、安全） ---
    as_string(): ResultBox<StringBox, ErrorBox> {
        if me.kind == :string {
            return Ok(me.payload as StringBox)
        }
        return Err(TypeErrorBox.mismatch(:string, me.kind))
    }

    as_array(): ResultBox<ArrayBox<Box>, ErrorBox> {
        if me.kind == :array {
            return Ok(me.payload as ArrayBox<Box>)
        }
        return Err(TypeErrorBox.mismatch(:array, me.kind))
    }

    // ... 他の型も同様

    // --- 取り出し（Option版、軽量） ---
    try_string(): OptionBox<StringBox> {
        if me.kind == :string {
            return OptionBox.some(me.payload as StringBox)
        }
        return OptionBox.none()
    }

    // ... 他の型も同様

    // --- デバッグ ---
    kind_name(): StringBox {
        return me.kind.to_string()
    }

    debug(): StringBox {
        // 安全な短縮表示（巨大データは省略）
        // ...
    }
}
```

---

## 🎯 `Any` vs `Box` の違い

### ❌ `Any` を使わない理由

```hako
// NG: Any版
static box ValueBox {
    data_type: StringBox
    data: Any  // ← これがダメ
}
```

**問題**:
- **Deterministic モード**と相性が悪い（`Any`は関数/ハンドル等も入り得る）
- シリアライズ不可／リプレイ不能が紛れ込む
- トレースと検証に弱い
- Phase 20-25の型パスに橋をかけにくい

### ✅ `Box` に限定する理由

```hako
// OK: Box版
static box DataBox {
    kind: SymbolBox
    payload: Box  // ← 既存のBoxのみ
}
```

**利点**:
- **Everything is Box** を徹底
- 観測・検証・序列化・差分が全部「箱API」で揃う
- 決定性実行の保証
- Plugin-First と自然整合

---

## 🎨 @match マクロ（砂糖）

`is_*/as_*` の連打を避けるための構文糖衣。

```hako
@test
flow demo(value: DataBox) {
    @match value {
        .string(s) => print("len=" + s.len()),
        .array(a)  => print("items=" + a.len()),
        .map(m)    => print("keys=" + m.keys().len()),
        _          => print("other: " + value.kind_name()),
    }
}
```

**脱糖先**:
```hako
// if/else に展開（MIR14は増えない）
if let Some(s) = value.try_string() {
    print("len=" + s.len())
} else if let Some(a) = value.try_array() {
    print("items=" + a.len())
} else if let Some(m) = value.try_map() {
    print("keys=" + m.keys().len())
} else {
    print("other: " + value.kind_name())
}
```

---

## 📍 使いどころの原則（乱用防止）

### ✅ 境界で使う

- プラグイン境界（IPC/FFI/WASM）
- 永続化/キャッシュ（JSON/CBOR へ安定シリアライズ）
- Python/TS interop（PyRuntimeBox, JS 側への受け渡し）

### ❌ 内部では使わない

- 内部のホットパス（SSA/Lower/Opt）は素の`ArrayBox/MapBox`を使う

### 短命化パターン（推奨）

```hako
// Builder → DataBox（境界）
let v: DataBox = builder.emit_as_data();

// すぐに取り出して型付きで処理（内部）
let blocks = v.as_array()?;
LocalSSABox.process(blocks);
```

**境界で使う／中は型付きで走る**

---

## 🔄 JSON との関係

### DataBox ≠ JsonBox

- **JsonBox**: フォーマット（文字列）中心
- **DataBox**: 構造そのもの（Boxの森）

### JSON変換

```hako
static box DataBox {
    // ...

    to_json(): StringBox {
        // DataBox → JSON文字列
        // OrderedMapで決定性保証
    }

    from_json(json: StringBox): ResultBox<DataBox, ErrorBox> {
        // JSON文字列 → DataBox
        // パースエラーは ErrorBox
    }
}
```

---

## 🌉 Phase 20-25への橋（SchemaBox / ProofBox）

型 opt-in 期に備えて、**実行時検証の「証拠」**を残す。

```hako
// 実行時の形状検証
let schema: SchemaBox = schema_from_string("{ instructions: array, ... }");
let proof: ProofBox = schema.check(v)?; // v: DataBox

// 以後、この v は schema 準拠として扱える（静的パスに渡せる）
use_verified(v, proof);
```

**現段階では**:
- メタ情報（ProofBox）のみ付与
- 後の静的検証パスで`ProofBox`を使って型を高める
- でもMIRは増やさない

---

## 🔧 MirJsonBuilderMin との統合例

### Dual API設計

```nyash
static box MirJsonBuilderMin {
    // 既存互換（文字列）
    emit_to_string(st): StringBox { ... }

    // 新API（構造化）
    emit_as_data(st): DataBox {
        return DataBox.of_array(me.get_blocks(st) as ArrayBox<Box>)
    }

    // 高速経路（内部専用）
    get_blocks(st): ArrayBox<Box> {
        return st.get("blocks")
    }
}
```

**使い分け**:
- **外向け**（保存/外部連携）: `emit_as_data()` or `emit_to_string()`
- **内向け**（SSA/最適化）: `get_blocks()` で直接型付き

---

## 📊 実装コスト見積もり

### Phase A: 軽量ValueBox（文字列タグ版）
- **実装**: 50行
- **時間**: 1日
- **内容**: StringBox タグ、panic版、正規化ヘルパー

### Phase B: SymbolBox導入
- **実装**: 150行（SymbolBox 100行 + ValueBox統合 50行）
- **時間**: 1週間
- **内容**: interning table、整数比較

### Phase C: DataBox完全版（Result/@match）
- **実装**: 500行（DataBox 300行 + @match 200行）
- **時間**: 2週間
- **内容**: Result型、ErrorBox、@matchマクロ

---

## 🎊 Hakorune哲学チェック

### Everything is Box ✅
- payloadは必ずBox
- 「箱だけが箱に入る」保証

### Plugin-First ✅
- 決定性実行と整合
- プラグインもDataBoxで受け渡し可能

### Deterministic ✅
- `Any`排除で決定性保証
- OrderedMap で安定シリアライズ

### De-sugaring Contract ✅
- @matchはif/elseに脱糖
- MIR14据え置き

### Fail-Fast ✅
- Result/Optionで明示的失敗
- panicは最小限

---

## 🚀 実装順序（Phase 15との関係）

1. **Phase 15中**: 何もしない（セルフホスティング集中）
2. **Phase 15完了後**: 軽量ValueBox（Phase A）
3. **Phase 20-25**: DataBox完全版（Phase C）
4. **Phase 25+**: @matchマクロ（Phase C完成）

---

## 📚 参考資料

- [rune-design.md](./rune-design.md) - Rune設計全般
- Phase 15.5 MIR統一化議論（型混在問題の発端）
- ChatGPT o1議論ログ（2025-10-03）

---

**最終更新**: 2025-10-03
**レビュー**: ChatGPT o1 + Claude Sonnet 4.5
**承認待ち**: Phase 15完了後に再検討
