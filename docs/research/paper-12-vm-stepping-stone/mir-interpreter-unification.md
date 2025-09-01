# MIR統一解釈層の設計と実装（2025-09-01追記）

## 背景：VMがMIR解釈の参照実装だった

### 元々の意図
```
VM = MIR解釈層（唯一の真実）
  ↓
他のバックエンド（JIT/AOT/LLVM）もVMの解釈を参照
```

### 実際に起きた問題
```
VM → 独自のMIR解釈実装
JIT → 独自のMIR解釈実装（VMと微妙に違う）
LLVM → また独自のMIR解釈実装
Cranelift → さらに独自実装の危機...

結果：同じMIR解釈を4回も実装！
```

## Semanticsトレイト設計（ChatGPT5との共同設計）

### 統一インターフェース
```rust
trait Semantics {
    type Val;  // 値の表現（VMValue, LLVM Value, etc）
    
    // MIR命令の意味論を定義
    fn const_i64(&mut self, v: i64) -> Self::Val;
    fn binop_add(&mut self, lhs: Self::Val, rhs: Self::Val) -> Self::Val;
    fn box_call(&mut self, type_id: u16, method_id: u16, args: Vec<Self::Val>) -> Self::Val;
    // ... 他の命令
}
```

### 各バックエンドの実装
```rust
// VM: 実際に計算
impl Semantics for VmSemantics {
    type Val = VMValue;
    fn binop_add(&mut self, lhs: VMValue, rhs: VMValue) -> VMValue {
        // 実際に加算を実行
    }
}

// LLVM: IR生成
impl Semantics for LlvmSemantics {
    type Val = inkwell::values::BasicValue;
    fn binop_add(&mut self, lhs: BasicValue, rhs: BasicValue) -> BasicValue {
        // LLVM IRのadd命令を生成
    }
}
```

## フォールバック削除の苦労

### フォールバックによる複雑性
```rust
// Before: フォールバック地獄
match jit_compile(mir) {
    Ok(code) => run_jit(code),
    Err(_) => {
        // JIT失敗→VMフォールバック
        // でもJITとVMで動作が違う...
        vm_execute(mir)  
    }
}
```

### 削除作業の大変さ
- フォールバック用の条件分岐が至る所に散在
- エラーハンドリングが複雑に絡み合う
- テストもフォールバック前提で書かれている
- 「念のため」のコードを全て削除する必要

### 削除後のシンプルさ
```rust
// After: 潔い失敗
match backend {
    Backend::VM => vm.execute(mir),
    Backend::JIT => jit.compile_and_run(mir),  // 失敗したら失敗！
}
```

## 教訓

1. **最初から統一設計が重要**
   - VMをMIR解釈の参照実装として明確に位置づける
   - 他のバックエンドはVMの動作を忠実に再現

2. **フォールバックは悪**
   - 一時的な安心感と引き換えに長期的な複雑性を生む
   - 失敗は失敗として扱う方がシンプル

3. **AI協調開発での伝達の重要性**
   - 「VMをMIR解釈層にする」という意図が伝わらなかった
   - 結果として重複実装が発生
   - 明確な設計文書の重要性

## 追記：LLVM地獄からの解放（2025-09-01）

### LLVM統合で直面した現実
本日、LLVM統合作業中に以下の問題が発生：
- llvm-sys v180.0.0のバージョン要求地獄
- WSL→Windows環境変数伝播の失敗
- 文字化けによるバッチファイル実行不能
- vcpkgでのLLVMビルドが数時間かかりCPU 100%

### Craneliftという救世主
```bash
# LLVM: 環境構築だけで1日以上
# Cranelift: 5分で完了
cargo build --release --features cranelift-jit
```

### 戦略的転換の決断
LLVMの問題点（巨大・ビルド困難）を認識し、既存のCranelift実装に戻ることを決定。これは後退ではなく、実用性を重視した前進である。

### 統一設計の新たな意味
- MIR解釈の統一だけでなく「開発体験の統一」も重要
- 外部依存（LLVM）による複雑性は制御不能
- Rust製軽量バックエンド（Cranelift）により、真の統一が実現
- 「大きいものが良い」という固定観念からの解放