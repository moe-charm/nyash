# Sync<T>自動ロック設計 - ChatGPT5 Proの革命的提案

作成日: 2025-08-27

## 🎯 核心：「多重箱」問題の完全解決

**従来の問題**：
```nyash
// MutexBox<Map>という「箱の中に箱」構造が重い
init { MutexBox<Map<str,Bytes>> table }
with table.lock as m {  // 面倒なガード構文
    m.put(k, v)
}
```

**新提案**：
```nyash
init { Sync<Map<str,Bytes>> table }  // 一体型Box！
table.put(k, v)                      // これだけ！自動ロック
```

## 📦 Sync<T>の設計思想

### 1. **ロックが埋め込まれたBox**
- `Sync<T>`自体が単一のBox（多重箱ではない）
- 内部にRwLock相当の同期機構を内蔵
- APIからは通常のBoxと同じに見える

### 2. **メソッドプロキシによる自動ロック**
```nyash
// ユーザーが書くコード
table.put(k, v)

// 内部で起きること
1. write効果を検出 → 排他ロック取得
2. 内部のMap.put(k,v)を呼び出し
3. ロック解放
// すべて単一メソッド呼び出し内で完結！
```

### 3. **効果注釈との統合**
| メソッド効果 | ロック種別 | 例 |
|-------------|-----------|-----|
| `pure` | なし | 定数取得 |
| `read` | 共有ロック | get, len, contains |
| `write` | 排他ロック | put, remove, clear |
| `io` | 排他ロック | save, load |

## 🚀 具体的なAPI設計

### 基本使用（99%のケース）
```nyash
// 宣言
init { Sync<Map<str, int>> scores }
init { Sync<Vec<Player>> players }

// 書き込み（自動排他ロック）
scores.put("Alice", 100)
players.push(new Player("Bob"))

// 読み取り（自動共有ロック）
local score = scores.get("Alice")
local count = players.len()

// すべて普通のメソッド呼び出し！
```

### ガード構文が必要な1%のケース
```nyash
// 参照を返したい時のみ
read players as p {
    local leader = p[0]  // 参照を保持
    render(leader.name)
}

// 複数操作をまとめたい時
with scores.write as s {
    s.put("Alice", s.get("Alice") + 10)
    s.put("Bob", s.get("Bob") + 5)
    s.normalize()  // 3操作を1ロックで
}
```

## 🛡️ 安全性保証

### コンパイル時検出
1. **S1**: 参照返却の検出
   ```nyash
   // ❌ エラー：参照は自動ロックでは返せない
   local ref = table.get_ref(k)
   // 💡 提案：read table as t { local ref = t.get_ref(k) }
   ```

2. **S2**: ネストロックの防止
   ```nyash
   // ❌ エラー：read中にwriteは不可
   read table as t {
       table.put(k, v)  // デッドロック防止
   }
   ```

### 実行時最適化
- 単一操作：ロック取得→操作→即解放（最小オーバーヘッド）
- 複数操作：`with`でまとめて効率化
- デッドロック回避：複数Syncの静的順序付け

## 🔧 実装戦略

### Phase 1: 基本実装
```rust
// Sync<T>の内部構造
pub struct SyncBox<T: NyashBox> {
    inner: Arc<RwLock<T>>,
    base: BoxBase,
}

// 自動ロックプロキシ
impl<T> SyncBox<T> {
    pub fn put(&self, k: Key, v: Value) -> Result<()> {
        let mut guard = self.inner.write()?;
        guard.put(k, v)
        // guardのdropで自動アンロック
    }
}
```

### Phase 2: 効果注釈統合
```nyash
// メソッドに効果アノテーション
@effect(read)
method get(key) { ... }

@effect(write)  
method put(key, value) { ... }
```

### Phase 3: MIR/VM統合
```
// MIR展開例
table.put(k, v) →
  LockAcquire(table, write)
  BoxCall(table.inner, "put", [k, v])
  LockRelease(table)
```

## 💡 革新性のポイント

1. **多重箱問題の根本解決**
   - 「BoxをBoxで包む」ではなく「ロック内蔵Box」

2. **使いやすさの極致**
   - 99%は通常メソッド呼び出し
   - 1%の特殊ケースのみガード構文

3. **Everything is Box哲学との完全調和**
   - Sync<T>も単なるBox
   - 特別扱いなし、統一的操作

4. **効果システムとの自然な統合**
   - 既存の効果注釈がそのままロック戦略に

## 🎯 結論

ChatGPT5 Proの`Sync<T>`設計は：
- **簡単** - `table.put()`で済む
- **安全** - 自動ロック/アンロック
- **高速** - 必要最小限のロック
- **美しい** - Everything is Box哲学を保持

これは単なる「同期プリミティブの追加」ではなく、**並行プログラミングの新パラダイム**：

> 「同期を意識させない同期」

まさに、Nyashが目指す「初心者にも優しく、上級者にも強力」を体現した設計です。

---

*「ロックは見えないところに。使うのは普通のメソッド」- 究極のユーザビリティ*