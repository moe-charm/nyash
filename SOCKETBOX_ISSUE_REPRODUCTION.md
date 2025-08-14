# 🚨 Issue #76: SocketBoxデッドロック再現手順

## 📋 **即座実行可能テスト**

### **1. 最小デッドロック再現**
```bash
# 実行コマンド (10秒でタイムアウト)
timeout 10s ./target/release/nyash test_socket_deadlock_minimal.nyash

# 期待結果: タイムアウトでデッドロック確認
# 出力例:
# [Console LOG] ✅ SocketBox作成成功
# [Console LOG] bind()実行開始...
# (ここで無限ブロック)
```

### **2. 他のBox正常動作確認**
```bash
# 実行コマンド
./target/release/nyash test_other_boxes_working.nyash

# 期待結果: 正常完了
# 出力例:
# [Console LOG] ✅ ArrayBox正常: size=1
# [Console LOG] ✅ MapBox正常: value=test_value  
# [Console LOG] 🎉 他のBox全て正常動作: 4件成功
```

### **3. SocketBox全メソッドテスト**
```bash
# 実行コマンド (30秒でタイムアウト)
timeout 30s ./target/release/nyash test_socket_methods_comprehensive.nyash

# 期待結果: 最初のtoString()でデッドロック
# 出力例:
# [Console LOG] Test 1: toString()実行...
# (ここで無限ブロック)
```

## 🔧 **ビルド手順**
```bash
# フルリビルド
cargo clean
cargo build --release -j32

# 確認
ls -la target/release/nyash
```

## 📊 **デバッグログ確認ポイント**

### **正常Box vs SocketBoxの出力差異**
```bash
# 正常Box例（ArrayBox）:
✅ ARRAY_METHOD: push() called
✅ ArrayBox push completed

# SocketBox（問題）:
🔥 SOCKETBOX CLONE DEBUG: Arc addresses match = true
# ここで停止 - 🔥 SOCKET_METHOD: bind() called が出力されない
```

### **問題箇所ピンポイント**
```rust
// src/interpreter/expressions.rs:462-464
// この downcast_ref または obj_value 取得でデッドロック
if let Some(socket_box) = obj_value.as_any().downcast_ref::<SocketBox>() {
    let result = self.execute_socket_method(socket_box, method, arguments)?;
```

## 📝 **実行後報告フォーマット**

```
## テスト実行結果

### Test 1: 最小デッドロック再現
- 実行時間: XX秒（タイムアウト）
- 最後の出力: "bind()実行開始..."  
- 結果: ✅ デッドロック再現確認 / ❌ 正常動作

### Test 2: 他のBox正常動作
- 実行時間: XX秒
- 成功Box数: X件
- 結果: ✅ 正常動作 / ❌ 異常あり

### Test 3: Socket全メソッド
- デッドロック発生メソッド: toString/isServer/bind/close
- 結果: ✅ 全メソッドでデッドロック / ❌ 一部のみ問題

## 修正内容
- 変更ファイル: src/xxx.rs
- 修正内容: 具体的変更点
- 根本原因: 原因の詳細説明

## 修正後テスト結果
- Test 1: ✅ 正常完了
- Test 2: ✅ 正常完了  
- Test 3: ✅ 全メソッド正常動作
```

---
**⚠️ 重要**: 必ず上記3テスト全ての実行結果を報告してください。部分的修正は不可です。