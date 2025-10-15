# 実装フェーズ詳細 - Phase 1-5完全ガイド

**Status**: Implementation Guide
**Created**: 2025-10-13
**Purpose**: 全実装フェーズの詳細な計画とガイド

---

## 📊 全体タイムライン

```
Phase 1: Hakorune VM完成       2-3週間  (P0: 最優先) ← 今ココ！
Phase 2: Parser/Tokenizer     1-2週間  (P1: 高)
Phase 3: Boxes プラグイン化   2-3週間  (P1: 高) ← 修正: 移行作業のみ
Phase 4: Runtime置き換え      6-8週間  (P2: 中)
Phase 5: AOT化               4-6週間  (P2: 中)

合計: 15-22週間 (4-5.5ヶ月)
```

---

## 🚀 Phase 1: Hakorune VM完成（最優先）

### 概要
**期間**: 2-3週間（実質1週間の集中実装 + 1-2週間のバッファ）
**難易度**: Medium
**優先度**: P0 (最高優先)
**担当**: Claude + ChatGPT協調

### 目標
- **Hakorune VMの16命令完全実装**（現在15/16 = 93% → 100%）
- Rust VMからの完全独立
- 509テストすべてPASS維持

### 実装内容
1. **Week 1: MirCall実装**（詳細は hakorune_vm_completion.md 参照）
   - Day 1: Callee型の設計と基礎実装
   - Day 2: Callee型の完全実装とテスト
   - Day 3-4: MirCallハンドラー実装
   - Day 5: テストと検証
   - Day 6: トレース機能の実装
   - Day 7: ドキュメント整備とコミット

2. **Week 2-3: バッファと統合テスト**
   - 予期しない問題の修正
   - パフォーマンス測定
   - エッジケースの対応
   - ドキュメント改善

### 受け入れ条件
- ✅ MirCall実装完了（16命令100%実装）
- ✅ 509テストすべてPASS
- ✅ VM/LLVMパリティ維持
- ✅ トレース機能動作
- ✅ パフォーマンス劣化が50%以内

### 期待される効果
- ✅ Rust VM (1,556行) の完全削除
- ✅ セルフホスティングの完全実現
- ✅ デバッグ容易性の維持

### リスク
- **Medium**: パフォーマンス劣化の可能性 → Phase 2 (AOT化) で解決
- **Low**: MirCall実装の複雑性 → 既存実装を参考

### 次のステップ
- Phase 2: Parser/Tokenizerのセルフホスト化

---

## 📝 Phase 2: Parser/Tokenizerのセルフホスト化

### 概要
**期間**: 1-2週間
**難易度**: Easy (既に85%完成)
**優先度**: P1 (高)
**担当**: Claude + ChatGPT協調

### 目標
- **Parser/Tokenizer (7,637行) の完全脱Rust化**
- セルフホストコンパイラの完成（残り15%）
- Rustパーサーとの互換性維持

### 実装内容
1. **Week 1: セルフホストコンパイラの完成**
   - 残り15%の実装
     - エラーメッセージの改善
     - エッジケースの対応
     - パフォーマンス最適化
   - デュアルパス方式の実装
     - Rust + Hakorune並行実行
     - フラグ切り替え（HAKO_USE_SELFHOST_PARSER=1）

2. **Week 2: 統合テストとパリティ確認**
   - パリティテスト（Rust vs Hakorune）
     - AST一致確認
     - エラーメッセージ一致確認
   - 509テストすべてPASS維持
   - ドキュメント整備

### 受け入れ条件
- ✅ セルフホストコンパイラ完成（100%）
- ✅ Rustパーサーとの互換性確認
- ✅ 509テストすべてPASS
- ✅ パフォーマンス劣化が30%以内

### 期待される効果
- ✅ Parser/Tokenizer (7,637行) の完全削除
- ✅ セルフホスティングの完全独立

### リスク
- **Low**: パフォーマンス劣化 → AOT化で解決
- **Low**: エッジケースの互換性 → テストカバレッジ高い

### 次のステップ
- Phase 3: Boxes実装のプラグイン化

---

## 📦 Phase 3: Boxes実装のプラグイン化

### 概要
**期間**: 2-3週間（プラグインシステム完成済み、移行作業のみ）
**難易度**: Easy-Medium（システム基盤は既に完成）
**優先度**: P1 (高)
**Note**: **Phase 15.6と同一作業**（プラグインシステムは完成済み、残りのBoxを plugins/ に移行するのみ）

### 目標
- **残りのBoxを plugins/ に移行**（プラグインシステムは既に完成）
- 単一ソース原則の実現
- 静的/動的ビルド分岐

### 実装内容（Phase 15.6計画から）
**核心コンセプト**:
```
plugins/          ← すべてのBox実装（唯一の管理場所）
  ├── core系      ← 静的リンク候補（hako_kernel features）
  │   ├── string/
  │   ├── integer/
  │   ├── bool/
  │   ├── array/
  │   ├── map/
  │   └── null/
  └── 拡張系      ← 動的ロード
      ├── io/     (file, buffer, path)
      ├── async/  (future, task, promise)
      ├── error/  (result, exception)
      └── network/ (http, socket)

src/boxes/        ← 完全削除（段階的）
```

**実装戦略**（移行作業のみ）:
1. **Week 1: 基盤系プラグイン化**
   - FutureBox (150行)
   - ResultBox (100行)
   - NullBox (50行)
   - ExceptionBox (80行)
   - TypeBox (200行)
   - MethodBox (100行)
   - ChannelBox (100行)
   - 重複登録ガードは既に実装済み ✅

2. **Week 2: IO/ネットワーク系**
   - FileBox (200行)
   - BufferBox (100行)
   - PathBox (100行)
   - HTTPBox (150行)
   - SocketBox (150行)

3. **Week 3: 統合テスト + src/boxes/ 削除**
   - plugin-on スモーク実行
   - Stage-2 HostHandle完全化（既に完成）✅
   - 段階的削除（実行基盤から順次）
   - 最終確認
   - ドキュメント更新

### 重複登録ガードの実装
**場所**: `src/runtime/provider_box/registration_guard.rs`

**実装内容**:
```rust
// 静的→動的の順序で登録
pub fn register_box(name: &str, source: ProviderSource) -> Result<()> {
    let mut registry = PROVIDER_REGISTRY.lock().unwrap();

    // 既に登録されている場合
    if let Some(existing) = registry.get(name) {
        match (existing.source, source) {
            // 静的→静的: エラー
            (ProviderSource::Static, ProviderSource::Static) => {
                return Err(format!("Duplicate static registration: {}", name));
            }
            // 静的→動的: スキップ（静的が優先）
            (ProviderSource::Static, ProviderSource::Plugin) => {
                eprintln!("[WARN] Skipping plugin registration for {} (static exists)", name);
                return Ok(());
            }
            // 動的→静的: エラー（このケースは通常発生しない）
            (ProviderSource::Plugin, ProviderSource::Static) => {
                return Err(format!("Cannot register static {} after plugin", name));
            }
            // 動的→動的: 警告（最初の登録を維持）
            (ProviderSource::Plugin, ProviderSource::Plugin) => {
                eprintln!("[WARN] Skipping duplicate plugin registration: {}", name);
                return Ok(());
            }
        }
    }

    // 新規登録
    registry.insert(name.to_string(), Provider { name, source });
    Ok(())
}
```

### bootstrap featureのデフォルト化
**場所**: `Cargo.toml`

```toml
[features]
default = ["bootstrap"]  # デフォルトで有効化
bootstrap = []           # テスト維持用
```

### 受け入れ条件
- ✅ すべてのBox実装プラグイン化完了
- ✅ 重複登録ガード動作確認
- ✅ plugin-on スモーク PASS
- ✅ 509テストすべてPASS
- ✅ src/boxes/ 完全削除

### 期待される効果
- ✅ Boxes実装 (12,752行) の完全削除
- ✅ プラグインシステムの完全確立
- ✅ 単一ソース原則の実現

### リスク
- **Medium**: 重複登録ガードの実装
- **Medium**: 既存コードとの互換性維持
- **Low**: パフォーマンス劣化 → 静的リンクで解決

### 次のステップ
- Phase 4: Runtime部分の段階的置き換え

---

## 🔧 Phase 4: Runtime部分の段階的置き換え

### 概要
**期間**: 6-8週間
**難易度**: Hard
**優先度**: P2 (中)
**Note**: Runtime確定後にPhase 5 (AOT化) を実施

### 目標
- **Runtime (9,311行) の大部分を脱Rust化**
- GC (335行 → 200行) の最小化
- 最小限のC ABI層の確立

### 実装内容
1. **Week 1-2: 型システムの置き換え**
   - TypeBox → プラグイン化
   - 型解決 → Hakorune実装
   - SSOT (Single Source of Truth) 完全実現
   - type_registry.rs → Hakorune実装

2. **Week 3-4: モジュールシステムの置き換え**
   - ModuleRegistry → Hakorune実装
   - Using解決 → セルフホストコンパイラに統合
   - modules_registry.rs → Hakorune実装

3. **Week 5-6: GC最小化**
   - GCアルゴリズム → Hakorune実装
   - Rust側 → 最小限のC ABIのみ提供
   - パフォーマンス重視の部分は残す（335行 → 200行）

   **GC API設計**:
   ```rust
   // Rust側（最小限のC ABI）
   #[no_mangle]
   pub extern "C" fn gc_alloc(size: usize) -> *mut u8 { /* ... */ }

   #[no_mangle]
   pub extern "C" fn gc_collect() { /* ... */ }

   #[no_mangle]
   pub extern "C" fn gc_mark(ptr: *mut u8) { /* ... */ }
   ```

   ```hakorune
   // Hakorune側
   static box GcBox {
     alloc(size) {
       return Extern("gc.alloc", size)
     }

     collect() {
       return Extern("gc.collect")
     }

     mark(ptr) {
       return Extern("gc.mark", ptr)
     }
   }
   ```

4. **Week 7-8: その他Runtime**
   - Scheduler → Hakorune実装
   - MessageBus → Hakorune実装
   - C ABI経由でRustと連携

### 受け入れ条件
- ✅ 型システム脱Rust化完了
- ✅ モジュールシステム脱Rust化完了
- ✅ GC最小化完了（335行 → 200行）
- ✅ 509テストすべてPASS
- ✅ パフォーマンス劣化が20%以内

### 期待される効果
- ✅ Runtime (9,311行) の大部分を脱Rust化
- ✅ GC (335行 → 200行) の最小化
- ✅ 最小限のC ABI層の確立

### リスク
- **High**: パフォーマンス劣化の可能性
- **High**: メモリ安全性の確保
- **Medium**: GC実装の複雑性

### 次のステップ
- Phase 5: Hakorune VM AOT化（パフォーマンス最適化）

---

## ⚡ Phase 5: Hakorune VM AOT化（パフォーマンス最適化）

### 概要
**期間**: 4-6週間
**難易度**: Medium
**優先度**: P2 (中)
**Note**: Phase 4 (Runtime置き換え) 完了後に実施（Runtime確定が必要）

### 目標
- **Hakorune VM全体をLLVMでAOT化**
- Rust VMと同等以上の速度を実現
- パフォーマンス問題の完全解決

### 実装内容
1. **Week 1-3: Hakorune VM → LLVM IR変換**
   - Hakorune VM全体（4,998行）をLLVMでコンパイル
   - AOT化ツールチェーンの構築
   - 最適化パスの適用

   **AOT化の仕組み**:
   ```
   Hakorune VM (.hako)
      ↓
   Hakoruneコンパイラ (selfhost)
      ↓
   MIR JSON
      ↓
   LLVM IR生成 (llvm_py/)
      ↓
   LLVM最適化 (opt -O3)
      ↓
   ネイティブコード (.o)
      ↓
   リンク (ld)
      ↓
   実行ファイル (hakorune_vm_aot)
   ```

2. **Week 4-5: パフォーマンス最適化**
   - ベンチマーク実行
     - Rust VM vs Hakorune VM (インタープリタ)
     - Rust VM vs Hakorune VM (AOT)
   - ホットパスの特定と最適化
   - インライン展開
   - ループ最適化
   - デッドコード削除

   **ベンチマーク例**:
   ```bash
   # Rust VM
   time ./target/release/hako --backend vm benchmark.hako

   # Hakorune VM (インタープリタ)
   time ./target/release/hako --backend nyvm benchmark.hako

   # Hakorune VM (AOT)
   time ./hakorune_vm_aot benchmark.hako
   ```

3. **Week 6: 配布パッケージング**
   - AOT化されたバイナリの配布
   - インストーラー作成
   - ドキュメント整備

### 受け入れ条件
- ✅ Hakorune VM AOT化完了
- ✅ パフォーマンス: Rust VMと同等以上（100-120%）
- ✅ 509テストすべてPASS
- ✅ 配布パッケージ作成完了

### 期待される効果
- ✅ パフォーマンス問題の完全解決
- ✅ Rust VMと同等以上の速度
- ✅ 配布の簡素化（単一バイナリ）

### リスク
- **Medium**: AOT化の複雑性
- **Low**: LLVM最適化の調整

### 次のステップ
- すべてのPhase完了！完全脱Rust達成

---

## 📊 全Phase完了後の構成

### 最終的なRust依存
```
Rust依存 (約15,000行)
├── C ABI層 (~500行) - プラグインシステムに必須
├── GC実装 (~200行) - パフォーマンス重視
├── LLVM Backend (~5,000行) - LLVM IR生成
├── WASM Backend (~3,000行) - WASM生成
├── Plugin Loader (~1,500行) - 動的ロード
└── その他Runtime (~4,800行) - スケジューラ等

完全削除 (約84,000行 = 85%)
├── Rust VM (1,556行) → Hakorune VM
├── Parser/Tokenizer (7,637行) → セルフホストコンパイラ
├── Boxes実装 (12,752行) → プラグイン化
└── その他 (62,000行) → Hakorune実装
```

### 新しいHakorune実装
```
Hakorune実装 (約30,000行)
├── Hakorune VM (5,000行) - 16命令完全実装 (AOT化)
├── セルフホストコンパイラ (10,000行) - Parser/MIR Builder
├── プラグイン (10,000行) - すべてのBox実装
└── Runtime (5,000行) - 型/モジュール/GC API
```

### 最終的な構成比
```
総行数: 約45,000行 (現在: 99,406行)
├── Rust: 15,000行 (33%)
└── Hakorune: 30,000行 (67%)

削減率: 55% (99,406行 → 45,000行)
```

---

## 🎯 推奨実施順序

### Option A: 順次実行（推奨）
```
Phase 1 (2-3週間)
  ↓
Phase 2 (1-2週間)
  ↓
Phase 3 (2-3週間) ← 修正: 移行作業のみ
  ↓
Phase 4 (6-8週間)
  ↓
Phase 5 (4-6週間)

合計: 15-22週間 (4-5.5ヶ月)
```

### Option B: 並行実行（リスキー）
```
Phase 1 (2-3週間)
  ↓
Phase 2 + Phase 3 並行 (2-3週間) ← 修正
  ↓
Phase 4 (6-8週間)
  ↓
Phase 5 (4-6週間)

合計: 12-17週間 (3-4.5ヶ月)
```

**推奨**: Option A（順次実行）
- 理由: 各Phaseの結果を確認しながら進められる
- リスク管理が容易

---

## 📈 進捗管理

### 全Phase進捗管理表
| Phase | 状態 | 進捗 | 期間 | 完了予定 |
|-------|------|------|------|----------|
| **Phase 1** | 🔥進行予定 | 0% | 2-3週間 | Week 3 |
| **Phase 2** | 📝計画 | 0% | 1-2週間 | Week 5 |
| **Phase 3** | 📝計画 | 0% | 2-3週間 | Week 8 |
| **Phase 4** | 📝計画 | 0% | 6-8週間 | Week 16 |
| **Phase 5** | 📝計画 | 0% | 4-6週間 | Week 22 |

### マイルストーン
- **Week 3**: Phase 1完了 - Hakorune VM 16命令完全実装
- **Week 5**: Phase 2完了 - Parser/Tokenizer脱Rust化
- **Week 8**: Phase 3完了 - Boxes プラグイン化（移行作業のみ）
- **Week 16**: Phase 4完了 - Runtime置き換え
- **Week 22**: Phase 5完了 - AOT化、完全脱Rust達成！

---

## ✅ 完了宣言テンプレート

### Phase 1完了時
```
🎉 Phase 15.75 Phase 1完了！Hakorune VM 16命令完全実装達成！

実装内容:
- MirCall実装完了
- 16命令完全実装（100%）
- 509テストすべてPASS
- Rust VMからの完全独立

次のステップ:
- Phase 2: Parser/Tokenizerのセルフホスト化（1-2週間）
```

### Phase 2完了時
```
🎉 Phase 15.75 Phase 2完了！Parser/Tokenizer完全脱Rust化達成！

実装内容:
- セルフホストコンパイラ完成（100%）
- Parser/Tokenizer (7,637行) 削除
- Rustパーサーとの互換性維持

次のステップ:
- Phase 3: Boxes実装のプラグイン化（2-3週間、移行作業のみ）
```

### Phase 3完了時
```
🎉 Phase 15.75 Phase 3完了！Boxes実装プラグイン化達成！

実装内容:
- すべてのBox実装プラグイン化
- Boxes実装 (12,752行) 削除
- 単一ソース原則の実現

次のステップ:
- Phase 4: Runtime部分の段階的置き換え（6-8週間）
```

### Phase 4完了時
```
🎉 Phase 15.75 Phase 4完了！Runtime置き換え達成！

実装内容:
- 型システム脱Rust化完了
- モジュールシステム脱Rust化完了
- GC最小化完了（335行 → 200行）
- Runtime大部分をHakorune化

次のステップ:
- Phase 5: Hakorune VM AOT化（4-6週間）
```

### Phase 5完了時（全Phase完了）
```
🎊 Phase 15.75 全Phase完了！完全脱Rust達成！🎊

総削減: 99,406行 → 45,000行 (55%削減)
├── Rust VM (1,556行) → 削除
├── Parser/Tokenizer (7,637行) → 削除
├── Boxes実装 (12,752行) → 削除
└── Runtime大部分 (58,000行) → 削除

最終構成:
├── Rust: 15,000行 (33%)
└── Hakorune: 30,000行 (67%)

成果:
✅ Rust依存を85%削減
✅ セルフホスティングの完全実現
✅ プラグインシステムの完全確立
✅ パフォーマンス維持（AOT化で100-120%）

Hakoruneは真のセルフホスティング言語になりました！
```

---

**最終更新**: 2025-10-13
**作成者**: Claude (comprehensive phase planning)
**次のアクション**: Phase 1開始 - Hakorune VM MirCall実装
