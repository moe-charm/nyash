# Phase 10.1 実装ステップガイド

## 🎯 実装の鉄則：必ずこの順序で！

ChatGPT5さんの指摘通り、緻密な計画と順序が成功の鍵にゃ。

## 📊 実装ステップ

### Step 1: ArrayBoxのプラグイン化（最小実装）

#### 1.1 プロジェクト作成
```bash
cd plugins/
cargo new nyash-array-plugin --lib
cd nyash-array-plugin
```

#### 1.2 最小限のC FFI実装
```rust
// src/lib.rs
#[repr(C)]
pub struct NyBox {
    data: *mut u8,
    typeid: u64,
    flags: u32,
    gen: u32,
}

#[no_mangle]
pub extern "C" fn nyplug_array_abi_version() -> i32 { 1 }

#[no_mangle]
pub extern "C" fn nyplug_array_new() -> NyBox {
    // 簡略実装：Vec<i64>のみサポート
    let vec = Box::new(Vec::<i64>::new());
    NyBox {
        data: Box::into_raw(vec) as *mut u8,
        typeid: 3, // ArrayBox
        flags: 0,
        gen: 1,
    }
}

#[no_mangle]
pub extern "C" fn nyplug_array_len(arr: NyBox) -> u64 {
    unsafe {
        let vec = &*(arr.data as *const Vec<i64>);
        vec.len() as u64
    }
}
```

#### 1.3 ビルド設定
```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "staticlib"]  # 動的・静的両対応
```

### Step 2: VM動作確認

#### 2.1 プラグインローダーとの統合
```rust
// src/runtime/plugin_loader_v2.rsに追加
fn load_builtin_plugins(&mut self) {
    // 既存のFileBox等に加えて
    self.register_plugin("nyash-array-plugin", 3); // ArrayBox type_id = 3
}
```

#### 2.2 テストプログラム
```nyash
// test_array_plugin.nyash
local arr
arr = new ArrayBox()  // プラグイン版を呼ぶ
print(arr.length())   // 0が出力されれば成功
```

#### 2.3 VM実行
```bash
./target/release/nyash --backend vm test_array_plugin.nyash
```

### Step 3: JIT動作確認

#### 3.1 LowerCoreの修正
```rust
// src/jit/lower/core.rs
match box_type {
    "ArrayBox" => {
        // HostCallからPluginInvokeに切り替え
        b.emit_plugin_invoke(3, method_id, args);
    }
    // 他のBoxは従来通り
}
```

#### 3.2 JIT実行テスト
```bash
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 ./target/release/nyash --backend vm test_array_plugin.nyash
```

### Step 4: 段階的移行

#### 4.1 移行優先順位
1. **ArrayBox** - 最も使用頻度が高い
2. **StringBox** - 基本的なデータ型
3. **IntegerBox/BoolBox** - プリミティブ型
4. **MapBox** - コレクション型
5. **その他** - 順次移行

#### 4.2 互換性維持
```rust
// フラグで切り替え可能に
if env::var("NYASH_USE_PLUGIN_BUILTINS").is_ok() {
    // プラグイン版を使用
} else {
    // 従来のビルトイン版
}
```

### Step 5: パフォーマンス測定

#### 5.1 ベンチマーク作成
```nyash
// bench_array_ops.nyash
local arr = new ArrayBox()
local start = Timer.now()
loop(i in 0..1000000) {
    arr.push(i)
}
local elapsed = Timer.now() - start
print("Time: " + elapsed)
```

#### 5.2 比較測定
```bash
# 従来版
./target/release/nyash --benchmark bench_array_ops.nyash

# プラグイン版
NYASH_USE_PLUGIN_BUILTINS=1 ./target/release/nyash --benchmark bench_array_ops.nyash
```

## 🎯 成功基準

### Phase 1（1週間）
- [ ] ArrayBoxプラグインが動作
- [ ] VM経由で基本操作（new, length, push, get）が可能
- [ ] パフォーマンス劣化が10%以内

### Phase 2（2週間）
- [ ] JIT経由でも動作
- [ ] 5つ以上のビルトインBoxがプラグイン化
- [ ] 既存テストがすべてパス

### Phase 3（1ヶ月）
- [ ] すべての主要ビルトインBoxがプラグイン化
- [ ] 静的リンクでの最小exe生成
- [ ] Linux/macOSで動作確認

## ⚠️ 注意事項

1. **TLVエンコーディング**: 既存のプラグインシステムに合わせる
2. **エラー処理**: panicではなくエラーコードを返す
3. **メモリ管理**: Box化されたデータのライフサイクルに注意

## 💡 デバッグ時のヒント

```bash
# プラグインロード確認
NYASH_DEBUG_PLUGIN=1 ./target/release/nyash test.nyash

# JIT呼び出し確認
NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm test.nyash
```

---

*"手順を守れば大丈夫" - 一歩ずつ確実に進めるにゃ！*