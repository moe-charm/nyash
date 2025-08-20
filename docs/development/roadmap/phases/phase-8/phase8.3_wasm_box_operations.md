# Phase 8.3: WASM Box Operations - オブジェクト操作のWASM実装

## Summary
Phase 8.2 PoC1で基本演算のMIR→WASM変換が完成。次はNyashの核心である「Everything is Box」哲学をWASMで実現する。メモリ管理とBox操作（RefNew/RefGet/RefSet）を実装し、オブジェクト指向プログラミングをWASMで動作させる。

## Current State
- ✅ Phase 8.1: WASM基盤完成（メモリ管理・ランタイム・WAT生成）
- ✅ Phase 8.2 PoC1: 基本演算完成（算術・比較・制御フロー・print）
- ✅ Phase 8.2 PoC2: CLI統合完成（`--compile-wasm`オプション + Safepoint対応）
- ✅ Phase 8.2 PoC3: ブラウザ実行確認（Nyash→WASM→Browser完全パイプライン）
- ✅ Phase 8.2 Docs: 実行バックエンド完全ドキュメント作成（execution-backends.md）
- 🚧 Phase 8.3: Box操作実装（本Issue）

## Technical Requirements

### 1. メモリレイアウト拡張
```wat
;; Box layout in WASM linear memory:
;; [type_id:i32][ref_count:i32][field_count:i32][field0:i32][field1:i32]...
;;
;; Example: StringBox
;; [0x1001][1][2][ptr_to_string][string_length]
```

### 2. メモリアロケータ改良
現在のbump allocatorを拡張：
- `malloc(size) -> ptr` - メモリ確保
- `free(ptr)` - メモリ解放（Phase 8.3では未実装、将来対応）
- アライメント考慮（4バイト境界）

### 3. MIR→WASM変換実装
```rust
// Phase 6で実装済みのMIR命令
MirInstruction::RefNew { dst, box_val }      // 新規Box作成
MirInstruction::RefGet { dst, reference, field }  // フィールド読み取り
MirInstruction::RefSet { reference, field, value } // フィールド書き込み
MirInstruction::NewBox { dst, box_type, args }    // Box生成
```

## Implementation Tasks

### Task 1: メモリ管理強化 🔧
- [ ] Box用メモリレイアウト定義（src/backend/wasm/memory.rs）
- [ ] malloc関数のWASM実装（アライメント対応）
- [ ] Box型ID管理システム（StringBox=0x1001等）

### Task 2: RefNew実装 📦
- [ ] `MirInstruction::RefNew` → WASM変換
- [ ] メモリ確保 + 初期化コード生成
- [ ] 参照カウント初期値設定（将来のGC対応準備）
- [ ] **実装例参考**:
  ```rust
  // src/backend/wasm/codegen.rs の MirInstruction::RefNew 処理
  MirInstruction::RefNew { dst, box_val } => {
      // 1. メモリサイズ計算 (header + fields)
      // 2. malloc呼び出し
      // 3. type_id設定
      // 4. ref_count=1設定  
      // 5. dst変数に格納
  }
  ```

### Task 3: RefGet/RefSet実装 🔍
- [ ] フィールドオフセット計算
- [ ] `MirInstruction::RefGet` → `i32.load` 変換
- [ ] `MirInstruction::RefSet` → `i32.store` 変換
- [ ] 型安全性チェック（デバッグビルドのみ）

### Task 4: NewBox実装 🎁
- [ ] Box型名→型ID解決
- [ ] コンストラクタ呼び出しシーケンス生成
- [ ] 初期化引数の処理

### Task 5: テスト実装 ✅
- [ ] `test_wasm_poc2_box_operations.rs` 作成
- [ ] 基本的なBox操作テスト
  ```nyash
  // テスト対象のNyashコード相当
  box DataBox { init { value } }
  local obj = new DataBox()
  obj.value = 42
  print(obj.value)  // 42が出力される
  ```
- [ ] **Copilot実装支援用：詳細テストケース**
  - [ ] RefNew単体テスト（Box作成のみ）
  - [ ] RefSet単体テスト（フィールド書き込み）
  - [ ] RefGet単体テスト（フィールド読み取り）
  - [ ] 複合操作テスト（作成→書き込み→読み取り）
  - [ ] エラーハンドリングテスト（不正アクセス等）
  - [ ] メモリレイアウト検証テスト（アライメント確認）

## Success Criteria
- [ ] RefNew/RefGet/RefSetがWASMで正常動作
- [ ] 簡単なオブジェクト操作がend-to-endで実行可能
- [ ] メモリレイアウトが明確にドキュメント化
- [ ] 既存のPoC1テストが引き続きPASS
- [ ] **Copilot品質保証**:
  - [ ] 全テストケースがCI環境でPASS
  - [ ] `cargo check` でビルドエラーなし
  - [ ] `--compile-wasm` オプションで正常なWAT出力
  - [ ] ブラウザでの実行確認（`wasm_demo/` 環境）
  - [ ] 既存Phase 8.2テストとの互換性維持

## Technical Notes

### 現在の実装基盤（2025-08-14時点）
- ✅ **WASM CLI**: `./target/release/nyash --compile-wasm program.nyash` で動作
- ✅ **ブラウザテスト**: `wasm_demo/` ディレクトリに実行環境完備
- ✅ **Safepoint対応**: `src/backend/wasm/codegen.rs:line XX` で実装済み
- ✅ **実行ドキュメント**: `docs/execution-backends.md` で使用方法詳細化

### AST→MIR制約への対応
現在AST→MIRは基本構文のみ対応（ユーザー定義Box未対応）。本Phaseでは：
- MIR直接構築によるテストを優先
- AST→MIR拡張は並行して別タスクで実施

### Copilot実装ガイダンス
Phase 8.3実装時の推奨アプローチ：
1. **段階的実装**: RefNew → RefGet → RefSet の順序で個別実装
2. **テスト駆動**: 各MIR命令に対応する単体テストを先に作成
3. **既存パターン活用**: `src/backend/wasm/codegen.rs` の既存実装を参考
4. **メモリ安全性**: アライメント・境界チェックを必ず実装
5. **デバッグ支援**: WAT出力にコメント追加で可読性向上

### 将来の拡張準備
- 参照カウントフィールドを含むが、Phase 8.3では使用しない
- GC実装は将来のPhaseで対応
- 文字列等の可変長データは次Phase以降

## Dependencies
- wasmtime 18.0.4
- wabt 0.10.0
- 既存のMIR Phase 6実装

## Estimate
- 実装期間: 2-3日
- 複雑度: 中（メモリ管理が主な課題）
- リスク: WASMメモリ管理の複雑性

---
Created: 2025-08-13
Target: Phase 8.3 PoC2
Priority: High