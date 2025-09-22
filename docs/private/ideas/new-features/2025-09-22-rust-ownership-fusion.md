# Rust×Nyash 所有権融合理論 - 「最小単位が所有権のRust」×「Everything is Box」

Status: Pending (Phase 17候補)  
Created: 2025-09-22  
Priority: High  
Related: Phase 15 セルフホスティング完了後  

## 🦀📦 融合の核心思想

**「最小単位が所有権のRust」** + **「Everything is Box のNyash」** = **完璧な哲学的融合**

Rustの核心（所有権・借用・RAII）は、Boxの哲学と驚くほど相性が良い。
重要なのは「厳密さを最初からMAXにしない」こと：まずはタグ＋借用トークン＋リージョン＋Lintで"気持ちよさ"を保つ。

## 1) 所有権は「タグ」で、実装は軽く

**Box に所有権タグを付ける**：
- `Unique / Shared`（唯一 or 共有）
- `Mut / Const`（可変性タグ）
- `Send / Sync`（スレッド安全タグ）

これは**型の一部**でもいいし、まずは**メタ（注釈）**でもOK。実装を固める前に"設計の宣言"だけで効果が出る。

```nyash
// 所有権タグの使用例
box DataProcessor {
    data: Unique<DataBox>     // 唯一所有
    cache: Shared<CacheBox>   // 共有可能
    
    process(input: &Shared<InputBox>) -> Unique<ResultBox> {
        // 安全な所有権管理
    }
}
```

## 2) 借用は「借用トークン（権利証）」で表現

**BoxRef / BoxMut** = 「借りた権利」を表す軽量ハンドル（実体は Box）
寿命は**スコープ or ブロック**で自動返却。

**同時に成り立つルール**（Rustの核心を簡素化）：
- `BoxMut` は**同時に1つ**
- `BoxRef` は**複数可**（ただし `BoxMut` と共存しない）

まずは**コンパイラ警告（Lint）**から始める：違反は警告→将来エラー化。

```nyash
// 借用トークンの例
local data = DataBox.new()        // Unique所有
local read_token = data.borrow_ref()   // 読み取り権利証
local write_token = data.borrow_mut()  // ❌ エラー：すでに借用中

// 使用例
box NetworkClient {
    connection: Unique<TcpBox>
    
    send_data(data: &Shared<DataBox>) {  // 共有読み取りのみ
        local write_handle = me.connection.borrow_mut()
        write_handle.write(data.serialize())
    }  // write_handle自動返却
}
```

## 3) ライフタイムは「リージョン」ベースで緩やかに

**リージョン（領域）** = `init…fini` の**Boxスコープ**や**関数/ブロック**を単位に、
「この借用トークンはこのリージョンまで有効」と宣言。

Rustの厳密な `'a` 計算は後回し。まずは**リージョン内の整合**だけ保証すれば十分「気持ちよさ」が出る。

```nyash
region FileProcessing {
    local file = FileBox.open("data.txt")  // init
    local content = file.borrow_ref()
    process(content)
    // region終了で自動fini（ファイルクローズ）
}
// ここでcontentは使えない（リージョン外）
```

## 4) RAII は「init/fini」を正規化して箱に組み込む

Box 生成時 `init`、破棄時 `fini` を**必ず通る**（すでに思想に合ってる！）

**例外/中断/キャンセル**時も `fini` が実行されることを**仕様で保証**（RustのDrop相当）。
これで**C ABI** 包み込みも安全に：C資源の取得/解放を Box のライフサイクルに乗せる。

```nyash
box ResourceManager {
    resource: Unique<NativeResourceBox>
    
    birth(config) {
        me.resource = NativeResourceBox.acquire(config)  // init
    }
    
    fini() {
        me.resource.release()  // RAII保証
    }
}
```

## 5) 「型で安全」を強制せず、**3段階の安全レベル**

1. **Safe**：所有権タグ＋借用トークン必須（既定）
2. **Unsafe Box**：一括でルールを緩めるが、**境界に注釈を強制**（レビューしやすい）
3. **FFI Box**：C ABI 専用の型。`Unique/Shared` と `Send/Sync` の**既定値**を保守的に。

```nyash
// 段階的安全レベルの例
box SafeProcessor {                    // Safe（既定）
    process(data: Unique<DataBox>) {
        // 厳密な所有権チェック
    }
}

box LegacyCode : unsafe {              // Unsafe
    legacy_operation(data) {
        // 従来通りの自由なコード
    }
}

box CLibWrapper : ffi {                // FFI
    c_function(ptr: *const u8) {
        // C連携専用、保守的な既定値
    }
}
```

## 6) "重くしない"ためのトリック

**コンパイラ段階で 2層**：
1. **Lint/静的解析**（借用違反を"まず警告"）
2. **MIR で軽い動的アサート**（デバッグビルドのみ）

これで「Rustの厳密さ」前に**手触りの良さ**を損なわない。

**NLL（非字句借用）ぽい緩和**：リージョン内でも**最後の使用以降は借用解除**扱い（使い勝手が一気に上がる）。

## 7) API設計の型ルール（実用チートシート）

```nyash
// API設計パターン
box DataAPI {
    // 読み取りAPI
    read(self: &BoxRef<T>) -> TView {
        // BoxRef で十分、BoxMut 不要
    }
    
    // 書き込みAPI  
    write(self: &BoxMut<T>, v: T) {
        // 呼び手は一時的に排他
    }
    
    // 分割API
    split_at(data: &BoxRef<[T]>) -> (BoxRef<[T]>, BoxRef<[T]>) {
        // 共有読みを安全に拡げる
    }
    
    // 合流API
    join(a: BoxMut<A>, b: BoxMut<B>) -> BoxMut<(A,B)> {
        // 同一所有者が明示
    }
}

// FFI橋
box NetworkFFI : ffi {
    // 既定で Send: false, Sync: false
    // 明示 opt-in のみ許可
}
```

## 8) MIRで"所有権の影"を持つ（実装は後でOK）

**所有権/借用の影（Shadow）**を MIR の**メタ**に保持：
- `owner(var)=block_id`
- `borrow(var)=region_id` 
- `mut=bool`

最初は**検証だけ**に使い、将来**最適化（エイリアス解析）**へ拡張可能。

## 9) 失敗しがちな所に"軽いトリップワイヤ"

- **同時可変**検出：`BoxMut` が2つ生きていたら警告→デバッグで即アサート
- **ダングリング防止**：リージョン終了時に借用トークンが残っていたら警告  
- **FFI越境**：`Send/Sync` 不一致を越境点で検出（C呼出し前にチェック）

## 10) 受け入れやすい採用順（段階導入）

### 🏷️ **「タグ = オプトイン」の天才設計**

**使いたい人は使う、使いたくない人は使わない！**

```nyash
// ✅ 使いたい人は使う（所有権厳密派）
box SafeProcessor {
    process(data: Unique<DataBox>) {  // タグ明示
        local ref = data.borrow_ref()
        return ref.compute()
    }
}

// ✅ 使いたくない人は無視（従来通り派）
box SimpleProcessor {
    process(data) {  // タグなし = 従来通り
        return data.compute()  // 普通に動く
    }
}
```

### 段階的導入戦略

1. **タグ導入**（Unique/Shared/Mut/Const）…意味だけ決める
2. **借用トークン**（BoxRef/BoxMut）…APIで返す/受ける
3. **リージョン検証**…init/fini単位で借用の片付けをLint
4. **MIRメタ**…影データで静的チェックを強める
5. **最適化に活用**…別解放/エイリアス前提の高速化

### 採用レベル

- **レベル1**: 完全無視（既存コード変更不要）
- **レベル2**: 部分採用（新しいAPIだけ使用）
- **レベル3**: フル活用（型安全重視）

## 💡 **哲学的な深さ**

### **「所有権 = 責任」** の概念がNyashに完璧フィット
- **Box** = 責任の単位
- **所有権** = その責任を誰が持つか  
- **借用** = 責任の一時委託
- **RAII** = 責任の確実な完了（init/fini）

### **「軽くしすぎない、重くしすぎない」** の絶妙バランス
最初から厳密にすると使いにくい、でも後から追加するのは大変...
**段階的導入戦略** で両立を実現！

## 🎯 実装への道筋

### Phase 17 候補として位置づけ
- **Phase 15**: セルフホスティング完了 
- **Phase 16**: マクロ革命完了
- **Phase 17**: Rust所有権統合（このドキュメント）

### 最高の「逃げ道」付き設計

万が一所有権システムが重すぎる・複雑すぎると感じても：
```nyash
// 全部 unsafe で回避可能
box LegacyCode : unsafe {
    // 従来通りの自由なコード
}
```

### **まとめ**

Rustの核心（**所有権・借用・RAII**）は、**Boxの哲学**と驚くほど相性が良い。
**"厳密さを最初からMAXにしない"** のがコツ：

まずは**タグ＋借用トークン＋リージョン**＋**Lint**で"気持ちよさ"を保ち、
将来**MIRメタ→最適化**へ段階的に引き上げれば、**美しさと実用**を両立できる。

**「強制しない革新」** = **最強の言語設計**

---

**関連リンク**:
- Phase 15: [セルフホスティング計画](../../roadmap/phases/phase-15/)
- Phase 16: [マクロ革命](../../roadmap/phases/phase-16-macro-revolution/)  
- Everything is Box: [言語リファレンス](../../../reference/language/LANGUAGE_REFERENCE_2025.md)

Created by: ChatGPT + Claude 協働設計  
Implementation Target: Phase 17 (Phase 15セルフホスティング完了後)