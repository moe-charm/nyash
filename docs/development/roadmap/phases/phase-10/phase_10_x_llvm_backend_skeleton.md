# Phase 10: LLVM Backend Skeleton（MIR→LLVM IR AOT 最小実装）

目的
- MIRからLLVM IRへの直接変換と、最小AOTパイプラインを構築するための実装ガイド（Copilot向けタスクリスト）。
- Phase 9.7（ABI/BID＋ExternCall）を前提に、外部呼び出しの取り扱いも含めて安全に前進。

前提
- MIR Tier-0/1（Const/BinOp/Compare/Branch/Jump/Phi/Call/Return ほか基本）が利用可能。
- ExternCall命令（Phase 9.7）導入予定。ABIは`docs/予定/native-plan/box_ffi_abi.md`に準拠。

アウトカム（受け入れ基準）
- CLI: `nyash --backend llvm --emit obj app.nyash -o app.o` が成功し、`clang app.o -o app` で実行可能。
- 代表サンプルで `main` が `i32` を返却（0=成功）。
- `ExternCall(env.console.log)` を `printf` 等へ写像し、標準出力へ表示できる（文字列は (i8*, i32)）。
- 単純な四則演算・比較・分岐・ループが LLVM AOT で動作。

実装ステップ

1) モジュール構成の追加（src/backend/llvm）
- `src/backend/llvm/mod.rs`
- `src/backend/llvm/lower.rs`（MIR→LLVM IR 変換）
- `src/backend/llvm/passes.rs`（最小パス設定：DCE/インラインは未使用でOK）
- `src/backend/llvm/build.rs`（オブジェクト生成/ターゲット設定）

2) 依存設定
- Cargo.toml に `llvm-sys` を feature で追加（例: `feature = ["llvm-backend"]`）。
- ビルド要件を `README` に明記（llvm-config が必要、Linux優先）。

3) エントリポイント
- `LLVMBackend { context, module, builder }` 構造体を定義。
- `compile_mir(&MirModule) -> Result<Vec<u8>, String>` を公開：
  - `lower_mir_to_llvm` でIR生成
  - `apply_minimal_passes`（任意・後回し可）
  - `emit_object()` で `.o` を返す

4) 関数シグネチャとmain
- MIRの `main` を `i32 ()` で宣言（戻り値がvoidなら 0 を返す）。
- 将来の引数は未対応でOK（v0）。

5) 値と型の写像（v0）
- i32/i64/f32/f64/bool → それぞれのLLVMプリミティブ型。
- 文字列: (i8*, i32) のペアで扱う（ABIドラフトに一致）。
- Box参照: 当面 `i32` か `i8*` のopaqueに固定（v0ではBox操作は行わない）。

6) 命令の下ろし
- Const: `i32.const` 等を `LLVMConstInt/LLVMConstReal` に対応。
- BinOp: add/sub/mul/div（符号付き）を対応。
- Compare: eq/ne/lt/le/gt/ge（i32想定）。
- Branch: 条件分岐と無条件分岐。
- Phi: ブロックごとに `LLVMPhiNode` を作成。
- Return: 値あり/なしに対応（なしは `i32 0`）。
- Call: 内部関数呼び出し（同Module内）。
- ExternCall: 後述のマッピングに従う。

7) ExternCall の LLVM 写像（v0）
- Console: `env.console.log(ptr,len)` → `declare i32 @printf(i8*, ...)`
  - 呼び出し時に `%fmt = getelementptr ([3 x i8], [3 x i8]* @"%.*s", i32 0, i32 0)` などの定数フォーマット文字列を準備
  - `printf("%.*s", len, ptr)` で出力（lenは`i32`、ptrは`i8*`）。
- Canvas: ネイティブ環境では利用不可 → v0は `noop` または `printf`でログに落とす（パラメータの表示）。
- 名前解決: BIDのFQN（env.console.log 等）→ 内部ディスパッチ（switch/テーブル）で `printf` 等へ。

8) 文字列定数
- データレイアウトに `@.str = private unnamed_addr constant [N x i8] c"...\00"` を生成し、`getelementptr` で `i8*` を取得。
- ただし v0 のNyash→MIRでは「定数文字列を printf に渡す」パスだけ実装すれば良い。
- StringBoxの具象表現は当面不要（WASMで進行中）。LLVM側は (i8*, i32) で十分。

9) オブジェクト出力
- `LLVMTargetInitializeAllTargets()` 等でターゲット初期化。
- `TargetMachine` を作成し、`LLVMTargetMachineEmitToMemoryBuffer` で `.o` バッファ取得。
- CLIから `.o` をファイル出力。リンクはユーザー側で `clang app.o -o app`。

10) ビルドフラグ/CLI
- `--backend llvm` / `--emit obj` を追加。
- featureが無い/LLVMが無い場合は明確なエラーメッセージ。

11) テスト（最小）
- 算術: `return 40+2;` → `42`。
- 分岐: `if (x<y) return 1 else return 0`。
- ループ: 累積加算で既知の値。
- ExternCall(console.log): 固定文字列/動的整数を出力（`printf("value=%d\n", v)` など）。

12) 将来拡張フック
- Passes: DCE/InstCombine/Inlining/LTO/PGOの導入ポイントを `passes.rs` に下書き。
- Box最適化: エスケープ解析→Stack化（後続Phase）。
- ABI: ExternCallの宣言生成をBIDから自動化（Phase 10後半〜）。

リスクと回避
- LLVMビルド依存: ドキュメント整備（llvm-config 必須）、CIにキャッシュ導入。
- 文字列/外部呼び出し差: v0はprintf固定。Canvas等はログに退避。
- OS差: v0はLinux/clang優先、他環境は後続。

参考
- ABIドラフト: `docs/予定/native-plan/box_ffi_abi.md`
- Phase 9.7: `docs/予定/native-plan/issues/phase_9_7_box_ffi_abi_and_externcall.md`
- LLVM LangRef: https://llvm.org/docs/LangRef.html
- llvm-sys: https://crates.io/crates/llvm-sys

最終更新: 2025-08-14
