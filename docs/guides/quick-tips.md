# 🚀 Hakorune Quick Tips - 1ページ実用ガイド

**目的**: 「こう書きたい時はこれを使う」を1ページにまとめた実用ガイド

---

## 🎯 **よくある場面と解決策**

### **1. 値のルックアップ（検索）**

#### ❌ **やりがちな書き方（冗長）**
```nyash
hex_digit(ch) {
    if ch == "0" { return 0 }
    if ch == "1" { return 1 }
    if ch == "2" { return 2 }
    // ... 16行続く
}
```

#### ✅ **推奨: match式**（シンプル）
```nyash
hex_digit(ch) {
    return match ch {
        "0" => 0, "1" => 1, "2" => 2, "3" => 3,
        "4" => 4, "5" => 5, "6" => 6, "7" => 7,
        "8" => 8, "9" => 9, "a" => 10, "b" => 11,
        "c" => 12, "d" => 13, "e" => 14, "f" => 15,
        _ => -1  // デフォルト値
    }
}
```

#### ✅ **推奨: indexOf**（連続文字）
```nyash
hex_digit(ch) {
    return "0123456789abcdef".indexOf(ch)
}
```

#### ✅ **推奨: MapBox**（複雑な値）
```nyash
static box HexMap {
    map: MapBox

    birth() {
        me.map = new MapBox()
        me.map.set("zero", 0)
        me.map.set("one", 1)
        // ...
    }

    get(key) {
        return me.map.get(key)
    }
}
```

---

### **2. 配列/コレクションの操作**

#### ❌ **やりがちな書き方（手動ループ）**
```nyash
doubled = new ArrayBox()
loop(i = 0; i < arr.len(); i = i + 1) {
    doubled.push(arr.get(i) * 2)
}
```

#### ✅ **推奨: map/filter/reduce**
```nyash
// 変換
doubled = arr.map(fn(x) { x * 2 })

// フィルタ
evens = arr.filter(fn(x) { x % 2 == 0 })

// 集約
sum = arr.reduce(0, fn(acc, x) { acc + x })

// 組み合わせ
result = [1, 2, 3, 4, 5]
    .map(fn(x) { x * 2 })
    .filter(fn(x) { x > 5 })
```

---

### **3. エラー処理**

#### ❌ **やりがちな書き方（try-catchのつもり）**
```nyash
// Hakoruneでは非推奨
try {
    data = readFile(path)
} catch(e) {
    handleError(e)
}
```

#### ✅ **推奨: ? 演算子**（Result伝播）
```nyash
// エラー時は自動で早期return
flow Main.main() {
    local data = readFile(path)?  // エラーならここでreturn
    local parsed = parseJson(data)?
    processData(parsed)
}
```

#### ✅ **推奨: postfix catch**（その場で処理）
```nyash
// エラーをその場でキャッチ
data = readFile(path) catch(e) {
    print("Failed to read: " + e.message)
    return defaultData()
}
```

#### ✅ **推奨: cleanup**（必ず実行）
```nyash
// 成功・失敗問わず実行
file.open(path)
    catch(e) { handleError(e) }
    cleanup { file.close() }
```

---

### **4. 高階関数・Lambda式**

#### ✅ **Lambda式の基本**
```nyash
// 基本形
local add = fn(x, y) { return x + y }

// 単一式（returnは省略可）
local double = fn(x) { x * 2 }

// 高階関数での使用
array.map(fn(x) { x * x })
array.sort(fn(a, b) { a - b })
```

#### ✅ **実用例: カスタムソート**
```nyash
users.sort(fn(a, b) {
    if a.age < b.age { return -1 }
    if a.age > b.age { return 1 }
    return 0
})
```

---

### **5. パターンマッチング（match式）**

#### ✅ **基本的なmatch**
```nyash
status_code = match response.status {
    200 => "OK"
    404 => "Not Found"
    500 => "Server Error"
    _ => "Unknown"
}
```

#### ✅ **match式は値を返す**
```nyash
// 式として使用
local message = match level {
    "info" => "Information"
    "warn" => "Warning"
    "error" => "Error!"
    _ => "Unknown"
}

// そのまま関数の戻り値に
getStatus() {
    return match me.state {
        "active" => 1
        "paused" => 0
        _ => -1
    }
}
```

---

## 🔍 **使い分けガイド**

### **ルックアップ（値の検索）**
```
シンプルなルックアップ    → match式
連続文字（0-9, a-z等）    → indexOf
複雑な値/動的             → MapBox
```

### **配列操作**
```
変換（各要素を変える）    → map()
絞り込み                  → filter()
集約（1つにまとめる）     → reduce()
```

### **エラー処理**
```
伝播（上に投げる）        → ? 演算子
その場で処理              → postfix catch
必ず実行                  → cleanup
```

---

## 🚨 **よくある間違い**

### **❌ 間違い1: if連鎖でルックアップ**
```nyash
// ❌ 冗長
if x == "a" { return 1 }
if x == "b" { return 2 }
if x == "c" { return 3 }

// ✅ match式を使う
return match x {
    "a" => 1, "b" => 2, "c" => 3,
    _ => 0
}
```

### **❌ 間違い2: 手動ループで変換**
```nyash
// ❌ 冗長
result = new ArrayBox()
loop(i = 0; i < arr.len(); i = i + 1) {
    result.push(arr.get(i) * 2)
}

// ✅ map()を使う
result = arr.map(fn(x) { x * 2 })
```

### **❌ 間違い3: try-catch風のエラー処理**
```nyash
// ❌ Hakoruneでは非推奨
try {
    risky()
} catch(e) {
    handle(e)
}

// ✅ ? 演算子またはpostfix catchを使う
risky()? または risky() catch(e) { handle(e) }
```

---

## 📚 **さらに詳しく**

- **完全リファレンス**: [LANGUAGE_REFERENCE_2025.md](../reference/language/LANGUAGE_REFERENCE_2025.md)
- **Quick Reference**: [quick-reference.md](../reference/language/quick-reference.md)
- **言語進化ロードマップ**: [language-evolution](../development/roadmap/language-evolution/)
- **発見性問題分析**: [discoverability-analysis.md](../development/roadmap/language-evolution/discoverability-analysis.md)

---

## 🎊 **まとめ**

Hakoruneは**強力な糖衣構文**を持っています：
- **match式** - if連鎖の代わり
- **Lambda式** - 高階関数で活用
- **? 演算子** - エラー伝播を簡潔に
- **postfix catch/cleanup** - エラー処理を直感的に

**これらを使えば、コードが劇的にシンプルになります！** 🚀

---

**作成日**: 2025-10-02
**作成者**: Claude Sonnet 4.5
**関連**: [Discoverability問題分析](../development/roadmap/language-evolution/discoverability-analysis.md)
