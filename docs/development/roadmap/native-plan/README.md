# 🚀 Nyash ネイティブビルド計画（Native Plan）

## 🎯 目的
開発者向けに「ビルド計画・段階的タスク・設計上の要点」を集約。  
利用者向けの具体的なビルド手順は guides/ 以下の各ガイドを参照。

## 📋 重要リンク
- **現在のタスク**: [development/current/CURRENT_TASK.md](../../current/CURRENT_TASK.md)
- **コア概念（速習）**: [reference/architecture/nyash_core_concepts.md](../../reference/architecture/nyash_core_concepts.md)
- **🤖 AI大会議記録**: [../ai_conference_native_compilation_20250814.md](../ai_conference_native_compilation_20250814.md)
- **🗺️ ネイティブコンパイル戦略**: [../native-compilation-roadmap.md](../native-compilation-roadmap.md)
- **フェーズ課題一覧**: [issues/](issues/)
- **🤖 Copilot協調**: [copilot_issues.txt](copilot_issues.txt)

## 🌟 **最新戦略 (2025-08-14 AI大会議策定)**

### ⚡ ネイティブコンパイル革命計画
**現状**: WASM 13.5倍実行高速化達成 → **目標**: 500-1000倍総合高速化

#### **Phase A: AOT WASM** (最優先 - 2-3週間)
```bash
nyash --compile-native program.nyash -o program.exe
```
- **技術**: wasmtime compile でネイティブ化
- **効果**: 13.5倍 → 100倍 (7倍追加向上)

#### **Phase B: Cranelift Direct** (中期 - 2-3ヶ月)  
```
Pipeline: MIR → Cranelift IR → ネイティブバイナリ
```
- **技術**: エスケープ解析・ボックス化解除
- **効果**: 100倍 → 200倍

#### **Phase C: LLVM Ultimate** (長期 - 6ヶ月+)
```
Pipeline: MIR → LLVM IR → 最適化ネイティブ
```
- **技術**: LTO・PGO・高度最適化
- **効果**: 200倍 → 500-1000倍

### 🔑 成功の鍵 (3AI一致見解)
1. **MIR最適化**: ボックス化解除がバックエンド差より圧倒的効果
2. **エスケープ解析**: スタック割り当て・型特殊化
3. **段階的検証**: 各Phaseでベンチマーク駆動開発

要点サマリ（統合）
- ビルド方針
  - デフォルトは CLI 最小構成（`cargo build --bin nyash`）。GUI/Examples は feature で任意有効化。
  - Windows ネイティブ: MSVC または WSL + cargo-xwin によるクロスコンパイルを推奨。
- MIR/VM の段階的導入
  - Phase 5.2: `static box Main` → MIR への lowering 経路を実装済み。
 - Phase 6: 参照/弱参照の最小命令（RefNew/RefGet/RefSet, WeakNew/WeakLoad, BarrierRead/Write=no-op）。
  - 例外/Async は薄く導入、先に snapshot/verify の安定化を優先。
- 弱参照の意味論（実装で壊れにくく）
  - WeakLoad は Option<Ref> を返す（生存時 Some、消滅時 None）。PURE 扱い（必要に応じ READS_HEAP）。
  - `fini()` 後の使用禁止・weak 自動 null・cascading 順序（weak はスキップ）を不変として扱う。
- Safepoint と Barrier
  - 関数入口・ループ先頭・呼出直後に safepoint。Barrier は最初は no-op 命令として実装可。
- テスト戦略
  - 黄金テスト：ソース→MIR ダンプのスナップショットで後退検出。
  - VM/JIT 一致：同入力で VM と JIT の結果一致（将来の AOT でも同様）。
  - 弱参照の確率テスト：alloc→weak→drop→collect→weak_load の順序/タイミングを多様化。

進行フェーズ（抜粋）
- Phase 0: CLI 最小ビルド安定化（Linux/Windows）。
- Phase 5.2: static Main lowering（実装済み）。
- Phase 6: 参照/弱参照（Barrier は no-op で開始）。
- Phase 7: nowait/await（スレッドベース、FutureBox 連携）。

Phase 8: MIR→WASM（Rustランタイムに非依存のWASM生成）

目的
- MIR から素の WebAssembly を生成し、ブラウザ/wasmtime（WASI）でサンドボックス実行する。
- Rust は「コンパイラ本体」のみ。実行時は純WASM＋ホスト import（env.print など）。

範囲（最小）
- ABI/インポート・エクスポート:
  - exports: `main`, `memory`
  - imports: `env.print(i32)`（文字列は一旦 i32 値/デバッグ用途でOK。将来は文字列ABIを定義）
- メモリ/ヒープ:
  - 線形メモリに簡易ヒープ（bump/フリーリスト）。
  - Box は固定レイアウト（フィールド→オフセット表）。
- 命令カバレッジ（段階導入）:
  - 算術/比較/分岐/loop/return/print
  - RefNew/RefSet/RefGet（Phase 6 と整合）
  - Weak/Barrier は下地のみ（WeakLoad は当面 Some 相当でOK、Barrier は no-op）

段階的マイルストーン（PoC）
1) PoC1: 算術/分岐/return のみのWASM出力（wasmtime/ブラウザで実行）
2) PoC2: オブジェクト最小実装（RefNew/RefSet/RefGet）で `print(o.x)` が動作
3) PoC3: Weak/Barrier の下地（WeakLoad は常に有効、Barrier はダミー命令）
4) PoC4: CLI 統合（`--backend wasm` で wasm 生成・実行。ブラウザ用は別JSローダ）

受け入れ基準
- wasmtime 実行で戻り値/標準出力が期待通り（PoC1–2）。
- Ref 系がメモリ上で正しく動作（PoC2）。
- Weak/Barrier のダミー実装を含むWASMが生成され、実行に支障がない（PoC3）。
- CLI オプションで wasm バックエンドが選択でき、未実装部分は明瞭にエラーメッセージで誘導（PoC4）。

テスト方針
- 生成WASMを wasmtime で実行し、戻り値/print の内容を検証。
- ブラウザ用はヘッドレス環境（node + WebAssembly API）で同等の確認ができるスクリプトを用意。

対象外（Phase 8）
- 本格的GC/Weak無効化、fini/Pin/Unpin、JIT/AOT、複雑な文字列ABI。

アーカイブ（長文メモ・相談ログ）
- docs/予定/native-plan/archive/chatgptネイティブビルド大作戦.txt
- docs/予定/native-plan/archive/追記相談.txt
  - 上記2ファイルの要点は本 README に統合済み。詳細経緯はアーカイブを参照。

備考
- デバッグ補助: `--debug-fuel` でパーサーの燃料制御。`--dump-mir`/`--verify` で MIR の可視化・検証。
- 一部の開発用ログ出力（/mnt/c/...）は存在しない環境では黙って無視されます（問題なし）。
