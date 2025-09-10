# ChatGPT5さんへ：プラグイン戻り値表示バグ詳細調査レポート

## 🎯 根本原因特定完了

### 問題の流れ
1. **プラグイン戻り値取得** ✅ 正常
   - `CounterBox.get()` → 整数2が返される
   - L1016でvmapに生のi64値として格納

2. **LLVM console.log呼び出し** ✅ 正常  
   - L1270で`nyash.console.log_handle(生のi64値)`呼び出し

3. **nyrt処理** ❌ **ここに問題**
   ```rust
   // crates/nyrt/src/lib.rs:2391-2401
   pub extern "C" fn nyash_console_log_handle(handle: i64) -> i64 {
       if let Some(obj) = handles::get(handle as u64) {  // ← 生の値2でハンドル検索→失敗
           let s = obj.to_string_box().value;
           println!("{}", s);
       } else {
           println!("{}", handle);  // ← なぜか空白表示される
       }
   }
   ```

### 疑問点
**なぜ`println!("{}", handle)`が空白になるのか？**
- handleには実際の値（2）が入っているはず
- なぜprintln!が何も出力しない？

## 🔍 必要な修正案

### 提案1: デバッグ出力追加
```rust
pub extern "C" fn nyash_console_log_handle(handle: i64) -> i64 {
    eprintln!("DEBUG: handle={}", handle);  // デバッグ出力
    if let Some(obj) = handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        println!("{}", s);
    } else {
        eprintln!("DEBUG: handle {} not found in registry", handle);
        println!("{}", handle);  // 元のコード
    }
    0
}
```

### 提案2: 生の整数値対応
```rust
pub extern "C" fn nyash_console_log_handle(handle: i64) -> i64 {
    if let Some(obj) = handles::get(handle as u64) {
        let s = obj.to_string_box().value;
        println!("{}", s);
    } else {
        // プラグイン戻り値の生の整数を直接表示
        println!("{}", handle);
        eprintln!("DEBUG: Printed raw value {}", handle);
    }
    0
}
```

## 🐛 追加問題: 文字列連結
MIRで`"result is: " + 2`が`String + Integer → Integer`と間違った型推論

お手数ですが、調査をお願いします！