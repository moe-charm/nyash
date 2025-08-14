# Phase 8.5: MIR 25命令完全仕様実装（ChatGPT5 + AI大会議決定版）

## 🎯 Issue概要

**最終決定**: AI大会議（Gemini+Codex）+ ChatGPT5先生によるMIR 25命令完全仕様の実装

**仕様確定**: ChatGPT5先生が「化け物に伸びる余白」と「実装の現実」のちょうど真ん中として設計した、**Nyashのコア価値（所有森＋weak＋Bus＋効果注釈）を無理なくIR化**する完璧な25命令セット

## 📋 確定版: MIR 25命令完全仕様

### **Tier-0: 普遍コア（8命令）**
```mir
Const       // 定数値生成（pure）
BinOp       // 二項演算（pure）
Compare     // 比較演算（pure）
Branch      // 条件分岐（control）
Jump        // 無条件ジャンプ（control）
Phi         // SSA phi関数（pure）
Call        // 外部関数呼び出し（context依存）
Return      // 関数戻り（control）
```

**効果**: 将来のJIT/AOT/WASMすべてで必須の基盤

### **Tier-1: Nyashセマンティクス（12命令）**
```mir
NewBox        // 強所有のBox生成（所有森のノード）
BoxFieldLoad  // Boxのフィールド読み（pure）
BoxFieldStore // Boxのフィールド書き（mut）
BoxCall       // Boxのメソッド呼び出し（context依存）
Safepoint     // 分割finiや割込み許可ポイント（io）
RefGet        // 参照（強/弱を問わず）を値として取得（pure）
RefSet        // 参照の差し替え（所有規則検証付き）（mut）
WeakNew       // weak ハンドル生成（非所有リンク作成）（pure）
WeakLoad      // weak から生存チェック付きで強参照取得（失効時null）（pure）
WeakCheck     // weak の生存確認（bool）（pure）
Send          // Bus送信（io）
Recv          // Bus受信（io）
```

**革命的価値**: **所有森＋weak＋Bus** が言語一次市民として表現可能

### **Tier-2: 実装補助・最適化友好（5命令）**
```mir
TailCall      // 末尾呼び出し（スタック節約）（control）
Adopt         // 所有移管: this が子を強所有に取り込む（mut）
Release       // 強所有を解除（weak化 or null化）（mut）
MemCopy       // 小さなメモリ移動（構造体/配列最適化フック）（mut）
AtomicFence   // 並行時の順序保証（Actor/Port境界で使用）（io）
```

**位置づけ**: 言語仕様の裏方。無くても表現可能だが、**性能・安全検査・移植性**が安定

## 🔧 効果（Effect）システム

### 効果分類と最適化ルール
```rust
pub enum Effect {
    Pure,     // 再順序化OK、CSE/LICM可能
    Mut,      // 同一Box/同一Fieldで依存保持
    Io,       // 再順序化禁止、副作用あり
    Control,  // 制御フロー変更
}
```

### 命令別効果定義
- **pure**: Const, BinOp, Compare, Phi, RefGet, WeakNew, WeakLoad, WeakCheck
- **mut**: BoxFieldStore, RefSet, Adopt, Release, MemCopy
- **io**: Send, Recv, Safepoint, AtomicFence
- **control**: Branch, Jump, Return, TailCall
- **context依存**: Call, BoxCall（呼び先効果に従属）

## 🔍 検証（Verifier）要件

### 所有森検証ルール
```rust
// 1. 強参照のin-degree制約
fn verify_ownership_forest(mir: &MirModule) -> Result<(), VerifyError> {
    for instruction in mir.instructions() {
        match instruction {
            NewBox { dst, .. } => verify_strong_indegree_one(dst)?,
            Adopt { parent, child, .. } => verify_ownership_transfer(parent, child)?,
            Release { ref_val, .. } => verify_release_safety(ref_val)?,
            RefSet { target, new_ref, .. } => verify_refset_safety(target, new_ref)?,
            _ => {}
        }
    }
}

// 2. 強循環禁止検証
fn verify_no_strong_cycles(mir: &MirModule) -> Result<(), VerifyError> {
    // 強エッジのみ辿ってDAG（森）であることを確認
}

// 3. weak参照の決定的挙動
fn verify_weak_determinism(mir: &MirModule) -> Result<(), VerifyError> {
    // WeakLoad/WeakCheckの失効時はnull/falseを返す（例外禁止）
}
```

### 安全性検証項目
- [ ] **所有森**: `strong in-degree ≤ 1`（NewBox/Adopt/Release/RefSetで常時検査）
- [ ] **強循環禁止**: 強エッジのみ辿ってDAG（森）であること
- [ ] **weak/強相互**: 双方向とも強 → エラー（片側はWeakNew経由で弱化）
- [ ] **RefSetの安全**: 強→強の差し替え時は旧所有元からのReleaseが伴うこと
- [ ] **WeakLoad/WeakCheck**: 失効時はnull/falseを返す（例外禁止、決定的挙動）
- [ ] **TailCall**: 末尾位置のみ可（Return直前）
- [ ] **Send/Recv**: at-least-once契約を満たすか、契約を明示

## 🚀 実装範囲・優先度

### Phase 8.5A: コア命令実装（最優先）
- [ ] **Tier-0完全実装**: 8命令の基盤確立
- [ ] **Tier-1 Box操作**: NewBox, BoxFieldLoad/Store, BoxCall
- [ ] **Tier-1 weak参照**: WeakNew, WeakLoad, WeakCheck
- [ ] **効果システム**: Effect注釈とVerifier基盤

### Phase 8.5B: 高度機能（重要）
- [ ] **所有移管**: Adopt, Release命令実装
- [ ] **最適化**: TailCall, MemCopy実装
- [ ] **並行制御**: AtomicFence実装
- [ ] **Bus操作**: Send, Recv統合

### Phase 8.5C: 検証・最適化（完成度）
- [ ] **Verifier完全実装**: 所有森・strong循環・安全性検証
- [ ] **バックエンド対応**: Interpreter/VM/WASM全対応
- [ ] **最適化パス**: pure再順序化・mut依存保持・io順序保証

## 🧪 代表的ロワリング実装例

### 1. look参照のロワリング
```nyash
// Nyashソース
local weak_ref = look parent.child

// MIRロワリング
%0 = WeakNew %parent_child_ref
%1 = WeakLoad %0         // 読み取り時に生存チェック
```

### 2. borrow{}ブロックのロワリング
```nyash
// Nyashソース
borrow parent.field {
    use_field(parent.field)
}

// MIRロワリング
%0 = WeakNew %parent_field   // ブロック先頭
%1 = WeakLoad %0
%2 = Call @use_field, %1
// ブロック末尾でハンドル破棄（MIR上はNop、型で書換禁止）
```

### 3. Bus最適化（Elision）
```nyash
// Nyashソース
send(data, local_receiver)
local result = recv(local_receiver)

// MIR最適化前
%0 = Send %data, %local_receiver
%1 = Recv %local_receiver

// MIR最適化後（同一スレッド/アリーナの場合）
%0 = BoxFieldLoad %local_receiver, "buffer"
%1 = BoxFieldStore %local_receiver, "buffer", %data
// Send/Recv → 直接アクセスに縮退
```

## 🎯 バックエンド別実装指針

### Interpreter実装
```rust
// 25命令を素直に実装（正しさの基準）
match instruction {
    MirInstruction::NewBox { dst, box_type } => {
        let box_val = create_box(box_type);
        self.set_value(dst, box_val);
    },
    MirInstruction::WeakCheck { dst, weak_ref } => {
        let is_alive = self.check_weak_alive(weak_ref);
        self.set_value(dst, Value::Bool(is_alive));
    },
    MirInstruction::TailCall { func, args } => {
        self.prepare_tail_call(func, args);
        return TailCallResult::Jump;
    },
    // ... 他23命令
}
```

### VM実装
```rust
// Register-VM + direct-threading
// Send/Recvはローカル判定時にインライン化
impl VM {
    fn execute_send(&mut self, data: RegId, target: RegId) {
        if self.is_local_target(target) {
            // ローカル最適化: 直接バッファ書き込み
            self.local_buffer_write(target, data);
        } else {
            // 通常のBus送信
            self.bus_send(data, target);
        }
    }
}
```

### WASM実装
```rust
// Send/Recvはhost import、MemCopyはmemory.copyに対応
fn compile_mem_copy(&mut self, dst: WasmAddr, src: WasmAddr, size: u32) {
    self.emit_wasm_instruction(&WasmInstruction::MemoryCopy {
        dst_offset: dst,
        src_offset: src,
        size,
    });
}

fn compile_send(&mut self, data: ValueId, target: ValueId) {
    // host importとして実装
    self.emit_call_import("env.bus_send", &[data, target]);
}
```

### JIT実装（将来）
```rust
// TailCall最適化、WeakLoadは世代タグでO(1)生存チェック
impl JITCompiler {
    fn compile_weak_load(&mut self, dst: RegId, weak_ref: RegId) -> JITCode {
        // 世代タグによる高速生存チェック
        let generation_check = self.emit_generation_check(weak_ref);
        let load_value = self.emit_conditional_load(weak_ref, generation_check);
        self.emit_store_register(dst, load_value)
    }
}
```

## 🧪 テスト戦略

### 1. Golden MIR テスト
```bash
# 各サンプルのMIRダンプが全バックエンドで一致
./target/release/nyash --dump-mir test_golden_mir.nyash > golden.mir
./target/release/nyash --backend vm --dump-mir test_golden_mir.nyash > vm.mir
./target/release/nyash --backend wasm --dump-mir test_golden_mir.nyash > wasm.mir
diff golden.mir vm.mir && diff vm.mir wasm.mir
```

### 2. 行動一致テスト
```bash
# 同入力→同出力（weak失効時のnull/false含む）
./target/release/nyash --backend interpreter test_behavior.nyash > interp.out
./target/release/nyash --backend vm test_behavior.nyash > vm.out  
./target/release/nyash --backend wasm test_behavior.nyash > wasm.out
diff interp.out vm.out && diff vm.out wasm.out
```

### 3. 性能スモークテスト
```bash
# 5種の代表ケースで性能継続検証
./target/release/nyash --benchmark add_loop.nyash
./target/release/nyash --benchmark map_getset.nyash
./target/release/nyash --benchmark alloc_free.nyash
./target/release/nyash --benchmark bus_local.nyash
./target/release/nyash --benchmark bus_actor.nyash

# 期待値: VMがinterp以上、WASMがVM以上
```

## ✅ 成功基準

### 必須基準（Phase 8.5完成）
- [ ] **25命令完全実装**: 全バックエンドで25命令サポート
- [ ] **効果システム動作**: pure/mut/io/control効果の正確な実装
- [ ] **Verifier動作**: 所有森・strong循環・安全性検証の動作確認
- [ ] **Golden MIRテスト**: 全テストケースでMIR一致
- [ ] **行動一致テスト**: 全バックエンドで出力一致
- [ ] **性能要件**: VM≥Interpreter、WASM≥VM

### 理想基準（長期価値）
- [ ] **最適化効果**: pure再順序化・CSE/LICM・Bus elision動作確認
- [ ] **所有森活用**: Adopt/Release/RefSetによる安全で効率的なメモリ管理
- [ ] **weak参照活用**: WeakCheck/WeakLoadによる軽量で安全な弱参照
- [ ] **JIT準備**: TailCall/MemCopyによる将来JIT最適化基盤

## 🤖 Copilot向け実装ガイド

### 実装順序推奨
1. **Tier-0基盤**: 8命令の確実な実装
2. **Box操作**: NewBox, BoxFieldLoad/Store（Everything is Box核心）
3. **weak参照**: WeakNew, WeakLoad, WeakCheck（循環参照対策）
4. **効果システム**: Effect注釈とVerifier統合
5. **高度機能**: Adopt/Release, TailCall等
6. **テスト**: Golden MIR・行動一致・性能検証

### 重要な設計原則
- **Everything is Box**: BoxFieldLoad/Storeで明確にBox中心設計
- **所有森**: strong in-degree ≤ 1を常時保証
- **決定的挙動**: WeakLoad/WeakCheckの失効時動作を一貫化
- **効果注釈**: 最適化パスの基盤となる正確な効果分類

### デバッグ支援
```bash
# MIR命令別実行トレース
./target/release/nyash --trace-mir-execution test.nyash

# 所有森検証
./target/release/nyash --verify-ownership-forest test.nyash

# 効果システム確認
./target/release/nyash --dump-mir-effects test.nyash
```

## 📊 期待される効果

### 技術的効果
- **所有森＋weak＋Bus**のIRレベル実現
- JIT/AOT最適化の強固な基盤確立
- バックエンド間の実装一貫性向上

### 開発効率向上
- 意味明確なMIRによるデバッグ性向上
- 最適化パス開発の大幅な容易化
- 長期保守コストの劇的削減

### パフォーマンス向上
- Bus elisionによる通信最適化
- pure命令の積極的再順序化
- TailCall/MemCopyによる実行効率化

---

**優先度**: Critical（Phase 8.4完了直後）
**担当**: Copilot + Claude協調実装  
**仕様策定**: ChatGPT5 + AI大会議（Gemini+Codex）完全一致決定
**最終目標**: Nyashコア価値の完璧なIR化実現