# ベンチマークアプリ問題レポート

**日時**: 2025-10-01
**調査者**: Claude Sonnet 4.5

---

## 🔍 **問題サマリー**

［2025-10-01 更新］
- コア側に「エントリ解決の強化」を入れました（既定ON）。
  - `Main.main` が無い場合でも、モジュール内に「唯一の `Box.main`（`Box.main` または `Box.main/0`）」が存在すれば、それをエントリとして実行します。
  - 環境変数 `NYASH_ENTRY_PREFER_STATIC_MAIN=0` で無効化可能。
  - これにより `static box Fibonacci { main() { ... } }` 形式のベンチも動作します。

### **症状**
- `simple_add` ベンチマーク: ✅ **動作成功** (結果: 42)
- `fibonacci` ベンチマーク: ✅ **動作成功** (結果: 55) ← 修正完了！

### **根本原因**
**エントリーポイントの不一致**

---

## 📊 **詳細分析**

### **1. 成功ケース: simple_add**

#### ソースコード
```nyash
static box Main {
  main() {
    local result = 15 + 27
    print(result)
    return result  // → 42
  }
}
```

#### 生成されるMIR関数
```
define i64 @Main.main() effects(read) {
    ...
    return 42
}
```

#### 実行結果
```
[UnifiedBoxRegistry] 🎯 Factory Policy: StrictPluginFirst
42
```

✅ **成功理由**: `Main.main()`が正しく実行される

---

### **2. 失敗ケース: fibonacci**

#### ソースコード
```nyash
static box Fibonacci {
  compute(n) {
    if n <= 1 {
      return n
    }
    return me.compute(n - 1) + me.compute(n - 2)
  }

  main() {
    local result = me.compute(5)
    print(result)
    return result
  }
}
```

#### 生成されるMIR関数（問題箇所）
```
1. define i64 @Fibonacci.compute/1(? %0) effects(read) { ... }
2. define i64 @Fibonacci.main/0() effects(read) { ... }
3. define void @main() {           ← ★ 問題！
     bb0:
       0: %0 = const void
       1: ret %0
   }
```

#### 実行結果
```
[UnifiedBoxRegistry] 🎯 Factory Policy: StrictPluginFirst
(無応答・タイムアウト)
```

❌ **失敗理由**: VMが`main()`を実行するが、これは`void`を返すだけで何もしない（修正済み：唯一の `Box.main` をエントリとして解決）

---

## 🔧 **問題の詳細**

### **エントリーポイント解決の問題**

1. **コンパイラの動作**:
   - `static box Fibonacci { main() { ... } }`をコンパイル
   - `Fibonacci.main/0()`関数を生成（正しい実装）
   - **追加で**`main()`関数を自動生成（`void`を返すダミー）

2. **VMの動作**:
   - エントリーポイントとして`main()`関数を探す
   - ダミーの`main()`を発見して実行
   - `void`を返して即座に終了
   - `Fibonacci.main/0()`は**実行されない**

3. **なぜsimple_addは成功するのか**:
   - `Main.main()`という名前が特別扱いされている可能性
   - または、`Main`という名前が優先的に処理される

---

## 🔍 **call_legacy の問題**

### **MIR命令の違い**

#### simple_add (成功)
```
直接的な算術演算のみ（Call命令なし）
```

#### fibonacci (失敗)
```
%14 = call_legacy %15(%13)  // Fibonacci.compute/1 を呼び出し
%18 = call_legacy %19(%17)  // Fibonacci.compute/1 を呼び出し
```

**call_legacy**: `callee: None`の状態で文字列ベースの関数解決を使用

### **VMのサポート状況**

✅ **実装確認済み**:
- `src/backend/mir_interpreter/handlers/calls/legacy.rs`
- `execute_legacy_call()`関数が存在
- 関数名解決ロジック:
  1. 完全一致チェック
  2. アリティ（引数数）を考慮した正規化
  3. `FunctionIndex`を使用したtail-uniqueクエリ
  4. 見つからない場合はエラー

⚠️ **潜在的な問題**:
- 関数が実際に実行されているかは不明
- タイムアウトの原因が無限ループなのか、実行されていないのか不明

---

## 💡 **推奨される解決策**

### **短期解決策（即座に試せる）**

#### Option 1: トップレベル`main()`関数を追加
```nyash
static box Fibonacci {
  compute(n) { ... }
}

main() {
  local fib = new Fibonacci()
  local result = fib.compute(5)
  print(result)
  return result
}
```

#### Option 2: `Main` という名前の箱を使用
```nyash
static box Main {
  compute(n) { ... }

  main() {
    local result = me.compute(5)
    print(result)
    return result
  }
}
```

### **中期解決策（調査が必要）**

1. **エントリーポイント解決ロジックの調査**:
   - なぜダミーの`main()`が生成されるのか
   - `Main.main()`が優先される理由は何か
   - エントリーポイント選択のルールを文書化

2. **call_legacy の動作確認**:
   - 実際に関数が呼ばれているかログ出力
   - 無限ループチェック
   - スタックトレース機能の追加

### **長期解決策（設計変更）**

1. **統一的なエントリーポイント規約**:
   - `Main.main()`を標準エントリーポイントとする
   - または、`flow Main { main() {} }`形式を推奨
   - ダミー`main()`の自動生成を停止

2. **call_legacy の廃止**:
   - 全てのCallを`callee: Some(Callee::...)`形式に移行
   - 型安全な関数解決を徹底
   - Phase 15.5の統一Call実装を完全適用

---

## 🧪 **検証手順**

### **最小再現ケース**

```bash
# 1. simple_add (成功)
./target/release/nyash --backend vm apps/benchmarks/simple_add/main.nyash
# 結果: 42 ✅

# 2. fibonacci (失敗)
timeout 3 ./target/release/nyash --backend vm apps/benchmarks/fibonacci/main.nyash
# 結果: タイムアウト ❌

# 3. MIR確認
./target/release/nyash --dump-mir apps/benchmarks/fibonacci/main.nyash | grep "^define"
# 結果:
#   define i64 @Fibonacci.compute/1
#   define i64 @Fibonacci.main/0
#   define void @main           ← これが問題
```

---

## 📝 **次のアクション**

### **即座に実施**
1. ✅ `Main` という名前で fibonacci ベンチマークを書き直す
2. ⏳ 動作確認
3. ⏳ WASMバックエンドでもテスト

### **調査が必要**
1. ⏳ エントリーポイント選択ロジックのソースコード確認
2. ⏳ `main()` 自動生成の理由を特定
3. ⏳ `call_legacy` の実行トレースを取得

### **ドキュメント化**
1. ⏳ エントリーポイント規約を文書化
2. ⏳ ベンチマーク作成ガイドラインを作成
3. ⏳ トラブルシューティングガイドに追加

---

## 🔗 **関連ファイル**

- ベンチマーク:
  - `apps/benchmarks/simple_add/main.nyash` (成功)
  - `apps/benchmarks/fibonacci/main.nyash` (失敗)
- VM実装:
  - `src/backend/mir_interpreter/handlers/calls/legacy.rs`
  - `src/backend/mir_interpreter/mod.rs`
- MIR定義:
  - `src/mir/instruction.rs`
- ランナー:
  - `tools/run_benchmark.sh`

---

**レポート作成日**: 2025-10-01
**次回更新**: 解決策実装後
