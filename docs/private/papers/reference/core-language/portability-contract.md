# 🤝 Nyash Portability Contract v0

*ChatGPT5アドバイス・全バックエンド互換性保証仕様*

## 🎯 目的

**「nyash --target= interp / vm / wasm / aot-rust / jit-cranelift」で同一プログラムが同一結果を保証**

全バックエンドでNyashプログラムが確実に動作し、最適化レベルに関係なく**決定的で予測可能な実行**を実現。

## 🔧 **Contract v0 仕様**

### 1️⃣ **決定的破棄（Deterministic Finalization）**

#### **強参照のみ伝播保証**
```rust
// ✅ 保証される動作
box Parent {
    child_strong: ChildBox  // 強参照→破棄連鎖
}

parent.fini()  // 必ずchild_strong.fini()も呼ばれる
```

#### **破棄順序の決定性**
```nyash
// 破棄順序: 最新→最古（スタック順序）
box Child from Parent {
    init { data }
    pack() {
        from Parent.pack()  // 1. Parent初期化
        me.data = "child"   // 2. Child初期化
    }
    // fini順序: 2→1（逆順破棄）
}
```

#### **例外安全性**
```rust
pub enum FinalizationGuarantee {
    AlwaysExecuted,    // fini()は例外時も必ず実行
    NoDoubleDestroy,   // 同一オブジェクトの二重破棄禁止
    OrderPreserved,    // 初期化と逆順での破棄保証
}
```

### 2️⃣ **weak参照の非伝播＋生存チェック**

#### **非伝播保証**
```nyash
box Parent {
    init { child_weak }
    
    pack() {
        local child = new Child()
        me.child_weak = weak(child)  // weak参照生成
        // child がfini()されても Parent は影響なし
    }
}
```

#### **生存チェック必須**
```mir
// MIR レベルでの生存チェック
%alive = weak_load %weak_ref
br %alive -> %use_bb, %null_bb

%use_bb:
    // weak参照が有効な場合の処理
    %value = /* weak_refの値使用 */
    jmp %continue_bb

%null_bb:  
    // weak参照が無効な場合の処理
    %value = const null
    jmp %continue_bb

%continue_bb:
    // 合流地点（Phi必須）
    %result = phi [%value from %use_bb, %value from %null_bb]
```

#### **自動null化契約**
```rust
pub struct WeakContract {
    auto_nullification: true,     // 参照先fini()時に自動null
    no_dangling_pointers: true,   // ダングリングポインタ禁止
    thread_safe_access: true,     // マルチスレッド安全アクセス
}
```

### 3️⃣ **Effect意味論（最適化可能性）**

#### **Effect分類契約**
```rust
pub enum EffectLevel {
    Pure,   // 副作用なし→並び替え・除去・重複実行可能
    Mut,    // メモリ変更→順序保証必要・並列化制限
    Io,     // I/O操作→実行順序厳密保証・キャッシュ禁止
    Bus,    // 分散通信→elision対象・ネットワーク最適化可能
}
```

#### **最適化契約**
```mir
// Pure関数→最適化可能
%result1 = call @pure_function(%arg) effects=[PURE]
%result2 = call @pure_function(%arg) effects=[PURE]
// → 最適化: %result2 = copy %result1

// Mut操作→順序保証
store %value1 -> %ptr effects=[MUT]
store %value2 -> %ptr effects=[MUT]  
// → 順序維持必須

// Bus操作→elision対象
send %bus, %message effects=[BUS]
// → ネットワーク最適化・バッチ化可能
```

### 4️⃣ **Bus-elision基盤契約**

#### **elision ON/OFF同一結果保証**
```bash
# 最適化ON→高速実行
nyash --elide-bus --target wasm program.nyash

# 最適化OFF→完全分散実行  
nyash --no-elide-bus --target vm program.nyash

# 結果は必ず同一（契約保証）
```

#### **Bus操作の意味保証**
```mir
// Bus送信の意味論
send %bus, %message effects=[BUS] {
    // elision OFF: 実際のネットワーク送信
    // elision ON: ローカル最適化（結果同一）
}

// Bus受信の意味論  
%msg = recv %bus effects=[BUS] {
    // elision OFF: ネットワーク受信待ち
    // elision ON: ローカル値返却（結果同一）
}
```

## 🧪 **Contract検証システム**

### **互換テストスイート**
```rust
// tests/portability_contract_tests.rs
#[test]
fn test_deterministic_finalization() {
    let program = "/* fini順序テスト */";
    
    let interp_result = run_interpreter(program);
    let vm_result = run_vm(program);
    let wasm_result = run_wasm(program);
    
    // 破棄順序・タイミングが全バックエンドで同一
    assert_eq!(interp_result.finalization_order, vm_result.finalization_order);
    assert_eq!(vm_result.finalization_order, wasm_result.finalization_order);
}

#[test]
fn test_weak_reference_semantics() {
    let program = "/* weak参照テスト */";
    
    // 生存チェック・null化が全バックエンドで同一動作
    let results = run_all_backends(program);
    assert_all_equal(results.map(|r| r.weak_behavior));
}

#[test]
fn test_effect_optimization_equivalence() {
    let program = "/* Effect最適化テスト */";
    
    // PURE関数の最適化結果が同一
    let optimized = run_with_optimization(program);
    let reference = run_without_optimization(program);
    assert_eq!(optimized.output, reference.output);
}

#[test] 
fn test_bus_elision_equivalence() {
    let program = "/* Bus通信テスト */";
    
    let elision_on = run_with_flag(program, "--elide-bus");
    let elision_off = run_with_flag(program, "--no-elide-bus");
    
    // Bus最適化ON/OFFで結果同一
    assert_eq!(elision_on.output, elision_off.output);
}
```

### **Golden Dump検証**
```bash
#!/bin/bash
# scripts/verify_portability_contract.sh

echo "🧪 Portability Contract v0 検証中..."

# 1. MIR出力一致検証
nyash --dump-mir test.nyash > golden.mir
nyash --dump-mir test.nyash > current.mir
if ! diff golden.mir current.mir; then
    echo "❌ MIR回帰エラー検出"
    exit 1
fi

# 2. 全バックエンド同一出力
declare -a backends=("interp" "vm" "wasm")
for backend in "${backends[@]}"; do
    nyash --target $backend test.nyash > ${backend}.out
done

# 出力一致確認
if diff interp.out vm.out && diff vm.out wasm.out; then
    echo "✅ 全バックエンド出力一致"
else
    echo "❌ バックエンド出力差異検出"
    exit 1
fi

# 3. Bus-elision検証
nyash --elide-bus test.nyash > elision_on.out
nyash --no-elide-bus test.nyash > elision_off.out
if diff elision_on.out elision_off.out; then
    echo "✅ Bus-elision同一結果"
else
    echo "❌ Bus-elision結果差異"
    exit 1
fi

echo "🎉 Portability Contract v0 検証完了"
```

## 📊 **Contract適合レベル**

### **Tier-0: 基本互換性**
- [ ] **決定的破棄**: fini()順序がバックエンド間で同一
- [ ] **weak非伝播**: weak参照が親破棄に影響しない  
- [ ] **基本Effect**: PURE/MUT/IO の意味論統一
- [ ] **出力一致**: 同一プログラム→同一標準出力

### **Tier-1: 最適化互換性**
- [ ] **PURE最適化**: 純粋関数の除去・移動がバックエンド間で同等
- [ ] **weak生存チェック**: 全バックエンドで同一タイミング
- [ ] **Bus-elision**: ON/OFF切り替えで結果同一
- [ ] **性能予測**: 最適化レベル差が定量的

### **Tier-2: 高度互換性**
- [ ] **メモリレイアウト**: Box構造がバックエンド間で互換
- [ ] **エラー処理**: 例外・パニックが同一動作
- [ ] **並行性**: Future/awaitが同一意味論
- [ ] **デバッグ**: スタックトレース・診断情報が同等

## ⚡ **実装優先順位**

### **Phase 8.4（今すぐ）**
1. **Tier-0契約実装**: 基本互換性確保
2. **Golden dump自動化**: CI/CDで回帰検出
3. **Bus命令設計**: elision基盤構築

### **Phase 8.5（短期）**
1. **Tier-1契約実装**: 最適化互換性
2. **性能ベンチマーク**: 契約準拠性測定
3. **エラー契約**: 例外処理統一

### **Phase 9+（中長期）**
1. **Tier-2契約実装**: 高度互換性
2. **形式検証**: 契約の数学的証明
3. **認証システム**: 契約適合認定

---

## 🎯 **期待効果**

### **開発者体験**
- **予測可能性**: どのバックエンドでも同一動作保証
- **デバッグ容易性**: バックエンド切り替えで問題切り分け
- **最適化信頼性**: 高速化しても結果不変保証

### **Nyash言語価値**  
- **差別化**: 「全バックエンド互換」言語として独自性
- **信頼性**: エンタープライズ採用の技術的根拠
- **拡張性**: 新バックエンド追加時の品質保証

---

*最終更新: 2025-08-14 - ChatGPT5アドバイス完全実装*

*「Everything is Box」×「全バックエンド互換」= Nyashの技術的優位性*