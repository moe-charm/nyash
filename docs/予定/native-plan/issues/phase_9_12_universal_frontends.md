# Phase 9.12: Universal Frontends（各言語→NyIR 落とし込み PoC）

目的（What/Why）
- 「All Languages → NyIR」を実証するため、代表的な言語サブセットのフロントエンドPoCを作る。
- 最適化は脇に置き、意味保存とエッジケースの把握を最優先にする。

対象（Initial set）
- Cサブセット（例外なし/CASあり）
- JavaScript/TypeScriptサブセット（辞書/例外/非同期の最小）
- Pythonサブセット（辞書/例外/awaitの最小）
- JVMサブセット（bytecode 経由：例外/スレッド）

成果物（Deliverables）
- `lang2nyir-<lang>` ツール（AST/IR→NyIR）
- Golden NyIR（各サンプルの `.nyir`）
- 変換ガイド（言語機能→NyIR/Ext/標準Box の対応表）

スコープ（Scope）
1) C-subset → NyIR
   - if/loop/call/return、構造体の最小投影、CAS（AtomicExt）
2) JS/TS-subset → NyIR
   - 例外（Try/Throw）、Promise/await（Await近似）、辞書/配列→標準Box
3) Python-subset → NyIR
   - 例外・awaitの最小、辞書/リスト→標準Box
4) JVM-subset → NyIR
   - 例外/スレッド/同期の最小投影（Ext準拠）

受け入れ基準（Acceptance）
- 各言語サンプルが NyIR に落ち、interp/vm/wasm/llvm のいずれかで実行可能
- Golden NyIR を用いた Diff 一致が取れる

参照（References）
- NyIR 仕様/Ext: `docs/nyir/spec.md`
- ビジョン: `docs/nyir/vision_universal_exchange.md`
- ABI/BID: `docs/予定/native-plan/box_ffi_abi.md`

