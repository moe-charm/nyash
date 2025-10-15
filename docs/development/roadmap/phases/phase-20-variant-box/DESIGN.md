# VariantBox設計詳細

**作成日**: 2025-10-08
**設計元**: ChatGPT Pro提案 + ultrathink 4タスク分析
**状態**: 承認済み（セルフホスト完了後に実装）

---

## 🎯 設計目標

### 哲学的整合性
- ✅ **Everything-is-Box**: VariantBox自体がBox、tag/fieldsもすべてBox
- ✅ **MIR16維持**: 新命令なし、すべて既存のnewbox/boxcallに脱糖
- ✅ **決定性**: ArrayBoxベースで再現性保証（MapBoxより優位）
- ✅ **Plugin-First**: 開いたタグ拡張可能（EnumSchemaBox.extend()）

### 技術的目標
- 型安全性向上（閉じた型システム）
- パターンマッチング構文
- Option/Resultの改善

---

## 🏗️ Core設計

### VariantBox構造

```hakorune
// 判別共用体の最小表現
box VariantBox {
    tag: StringBox           // Variant識別子（Phase 25でSymbolBox化可能）
    fields: ArrayBox         // 位置引数（常に配列、統一的API）

    birth(tag_str) {
        me.tag = new StringBox(tag_str)
        me.fields = new ArrayBox()
    }

    // 判定
    is_tag(name: StringBox) -> BoolBox {
        return me.tag.equals(name)
    }

    // フィールドアクセス（Fail-Fast）
    field(i: I64Box) -> Box {
        if i < 0 || i >= me.fields.len() {
            panic("VariantBox.field: index " + i.to_s() +
                  " out of bounds (len=" + me.fields.len().to_s() + ")")
        }
        return me.fields.get(i)
    }

    // 安全版（Option返却、Phase 20.9で実装）
    try_field(i: I64Box) -> OptionBox {
        if i < 0 || i >= me.fields.len() {
            return Option.none()
        }
        return Option.some(me.fields.get(i))
    }

    // シリアライズ（決定性キャッシュ用）
    to_data() -> DataBox {
        return DataBox.of_array([
            DataBox.of_string(me.tag.to_s()),
            DataBox.of_array(me.fields.map(|f| (f as BaseValue).to_data()?))
        ])
    }

    // デバッグ
    debug() -> StringBox {
        return me.tag + "(" + me.fields.join(", ") + ")"
    }
}
```

**設計判断**:

| 選択肢 | 採用 | 理由 |
|--------|------|------|
| **タグ型** | StringBox | SymbolBox未実装、即座に開始可能 |
| **フィールド型** | ArrayBox | MapBoxより決定性高い、順序保証 |
| **Arity** | 可変（0-N） | 単一値も複数値も統一的に扱える |

---

### EnumSchemaBox（型安全性検証）

```hakorune
// enum定義のメタ情報管理
static box EnumSchemaBox {
    name: StringBox              // "Result", "Option" 等
    variants: MapBox             // "Ok" -> VariantSchema

    register_variant(tag: StringBox, field_names: ArrayBox) {
        local schema = new VariantSchema()
        schema.tag = tag
        schema.field_names = field_names
        schema.arity = field_names.len()
        me.variants.set(tag, schema)
    }

    validate(v: VariantBox) -> Result {
        local schema = me.variants.get(v.tag)
        if schema == null {
            return Result.err("Unknown variant: " + v.tag)
        }
        if v.fields.len() != schema.arity {
            return Result.err("Arity mismatch: expected " + schema.arity +
                              ", got " + v.fields.len())
        }
        return Result.ok()
    }

    // プラグイン拡張（開いたタグ対応）
    extend(tag: StringBox, arity: I64Box) {
        me.register_variant(tag, [])  // フィールド名なし版
    }
}

// Variant定義の詳細
box VariantSchema {
    tag: StringBox
    field_names: ArrayBox        // ["value"], ["lhs", "rhs"] 等
    arity: I64Box

    // 名前付きアクセス（Phase 20.9で実装）
    field_index(name: StringBox) -> I64Box {
        return me.field_names.index_of(name)
    }
}
```

---

## 📋 @enumマクロ脱糖

### 入力構文

```hakorune
@enum Result {
    Ok(value)
    Err(error)
}
```

### 脱糖先（自動生成コード）

```hakorune
// 1. 名前空間static box
static box Result {
    // 2. Schema登録
    _schema: EnumSchemaBox

    _init_schema() {
        me._schema = new EnumSchemaBox()
        me._schema.name = "Result"
        me._schema.register_variant("Ok", ["value"])
        me._schema.register_variant("Err", ["error"])
    }

    // 3. コンストラクタ関数
    Ok(value) {
        local v = new VariantBox("Ok")
        v.fields.push(value)
        return v
    }

    Err(error) {
        local v = new VariantBox("Err")
        v.fields.push(error)
        return v
    }

    // 4. 判定ヘルパー
    is_ok(r: VariantBox) -> BoolBox {
        return r.is_tag("Ok")
    }

    is_err(r: VariantBox) -> BoolBox {
        return r.is_tag("Err")
    }

    // 5. アクセサ（型安全な値取り出し）
    as_ok(r: VariantBox) -> Box {
        if !me.is_ok(r) {
            panic("Expected Ok, got " + r.tag)
        }
        return r.field(0)
    }

    as_err(r: VariantBox) -> Box {
        if !me.is_err(r) {
            panic("Expected Err, got " + r.tag)
        }
        return r.field(0)
    }
}

// 初期化（モジュールロード時に実行）
Result._init_schema()
```

**生成コード量**: 約40-60行/enum（2 Variant × 20-30行）

---

## 🔄 @matchマクロ脱糖

### 入力構文

```hakorune
flow handle_result(r: VariantBox) {
    @match r {
        Ok(v) => {
            console.log("Success: " + v)
            return v
        }
        Err(e) => {
            console.error("Error: " + e)
            return null
        }
    }
}
```

### 脱糖先

```hakorune
flow handle_result(r: VariantBox) {
    if r.is_tag("Ok") {
        local v = r.field(0)
        console.log("Success: " + v)
        return v
    } else if r.is_tag("Err") {
        local e = r.field(0)
        console.error("Error: " + e)
        return null
    } else {
        panic("Non-exhaustive match on Result: unexpected variant " + r.tag)
    }
}
```

**脱糖ルール**:
1. パターン解析: `Ok(v)` → タグ"Ok" + 変数束縛`v = r.field(0)`
2. 分岐生成: `if ... else if ... else panic(...)`
3. 網羅性チェック: else節で実行時エラー

---

## 🎨 使用例

### Option型

```hakorune
@enum Option {
    Some(value)
    None
}

// 使用例
flow find_user(id: I64Box) -> VariantBox {
    if id > 0 {
        return Option.Some("User" + id.to_s())
    } else {
        return Option.None()
    }
}

flow main() {
    local result = find_user(42)

    @match result {
        Some(name) => print("Found: " + name)
        None       => print("Not found")
    }
}
```

### AST表現

```hakorune
@enum Expr {
    Lit(value: NumberBox)
    Var(name: StringBox)
    Add(lhs: Expr, rhs: Expr)
}

flow eval(e: Expr, env: MapBox) -> NumberBox {
    @match e {
        Lit(n)      => return n
        Var(name)   => return env.get(name)
        Add(lhs, rhs) => return eval(lhs, env) + eval(rhs, env)
    }
}
```

---

## 🆚 代替案との比較

### MapBox vs VariantBox

| 要素 | MapBox | VariantBox | 判定 |
|------|--------|-----------|------|
| 決定性 | ❌ 順序不定 | ✅ 配列順序保証 | VariantBox勝利 |
| 可読性 | ⚠️ `{tag: "Ok", value: 42}` | ✅ `Ok(42)` | VariantBox勝利 |
| 性能 | ❌ ハッシュ計算 | ✅ 配列アクセス | VariantBox勝利 |
| 既存実装 | ✅ 実装済み | ❌ 新規実装 | MapBox勝利 |

**結論**: VariantBoxが技術的に優位

### Box継承 vs VariantBox

| 要素 | Box継承 | VariantBox | 判定 |
|------|---------|-----------|------|
| 型安全性 | ❌ 開いた型 | ✅ 閉じた型 | VariantBox勝利 |
| 網羅性チェック | ❌ 不可能 | ✅ 可能 | VariantBox勝利 |
| 既存機能 | ✅ マクロ不要 | ❌ マクロ必要 | Box継承勝利 |

**結論**: VariantBoxが型安全性で優位

### MIR拡張 vs マクロ脱糖

| 要素 | MIR拡張 | マクロ脱糖 | 判定 |
|------|---------|-----------|------|
| 性能 | ✅ 最速 | ⚠️ やや遅い | MIR拡張勝利 |
| MIR複雑化 | ❌ 16→19-20命令 | ✅ 16命令維持 | マクロ勝利 |
| 実装コスト | ❌ 3バックエンド改修 | ✅ マクロ層のみ | マクロ勝利 |

**結論**: マクロ脱糖が哲学整合・実装コストで優位

---

## 📊 段階的導入戦略

### MVP版（Phase 20.6）: 手動構築

```hakorune
// マクロなし、手動でVariantBox構築
local some = new VariantBox("Some")
some.fields.push(42)

if some.is_tag("Some") {
    print(some.field(0))  // → 42
}
```

**利点**: 即座に実装可能、概念検証

### v1.0版（Phase 20.7）: @enumマクロ

```hakorune
@enum Option {
    Some(value)
    None
}

local some = Option.Some(42)
if Option.is_some(some) {
    print(Option.as_some(some))  // → 42
}
```

**利点**: コンストラクタ自動生成、可読性向上

### v2.0版（Phase 20.8）: @matchマクロ

```hakorune
@match some {
    Some(v) => print(v)
    None    => print("empty")
}
```

**利点**: パターンマッチング、網羅性チェック

---

## 🚀 将来拡張

### Phase 25: SymbolBoxタグ化

```hakorune
// StringBox → SymbolBox
box VariantBox {
    tag: SymbolBox  // :Some, :None （インターン済み）
    ...
}
```

**利点**: パフォーマンス向上、メモリ削減

### Phase 25: 静的網羅性チェック

```hakorune
// コンパイル時エラー
@match result {
    Ok(v) => ...
    // エラー: Err caseが不足
}
```

**実装**: 型パス（Phase 25）で実現

### Phase 30: GADT拡張

```hakorune
@enum Expr {
    Lit(n: NumberBox) -> Expr
    Add(lhs: Expr, rhs: Expr) -> Expr
}
```

**利点**: 型安全性さらに向上

---

## 🎓 設計原則まとめ

### 採用された設計
1. **VariantBox（tag: StringBox, fields: ArrayBox）**
2. **@enum/@matchマクロ脱糖**
3. **段階的導入（MVP→v1.0→v2.0）**
4. **StringBoxタグ（SymbolBoxは後回し）**

### 棄却された設計
1. ❌ MapBoxベース（決定性低い）
2. ❌ MIR命令拡張（MIR16維持に反する）
3. ❌ 一括導入（リスク高い）

### 成功の鍵
- 既存マクロ基盤の活用（pattern.rs、engine.rs）
- セルフホスト完了後の実装（Phase 15.7の後）
- 80/20ルール適用（MVP版で80%の価値）

---

**承認**: Phase 15.7完了後に実装開始
**優先度**: Phase 20内でP2（推奨、但しP1マクロ基盤の後）
