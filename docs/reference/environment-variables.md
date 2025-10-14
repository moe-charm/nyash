# Nyash Environment Variables (歴史的互換性リファレンス)

> **⚠️ 重要**: このドキュメントは歴史的変数と互換性情報を保持しています。
>
> **最新の推奨変数は [docs/guides/env-variables.md](../guides/env-variables.md) を参照してください。**
>
> Phase 15 ENV統合により、`guides/env-variables.md` が簡潔版となりました。
> 本ファイルは互換性確認・移行ガイドとして残されています。

Quiet / JSON-only
- `NYASH_JSON_ONLY=1`: child/acceptance runs print JSON payloads only to stdout.
- `NYASH_QUIET=1`: suppress non-essential logs (stderr) across subsystems.
  - Runner/registry/plugin init/dev verifiers honor quiet by default.
> Historical variables like `NYASH_ENABLE_USING`, `NYASH_USING_AST`, `NYASH_DISABLE_PLUGINS`, `NYASH_PLUGIN_ONLY` remain as compatibility aliases.
> Mappings:
> - `NYASH_ENABLE_USING` → `NYASH_USING=1`
> - `NYASH_USING_AST=1`  → `NYASH_USING_STRATEGY=prelude`
> - `NYASH_DISABLE_PLUGINS=1` → `NYASH_PLUGIN_POLICY=off`
> - `NYASH_PLUGIN_ONLY=1` → `NYASH_PLUGIN_POLICY=force`
> Prefer the new variables in new docs/scripts; legacy mentions below are kept for reference.

本ドキュメントは Nyash の環境変数を用途別に整理し、最小限の運用セットを提示します。`nyash.toml` の `[env]` で上書き可能（起動時に適用）。

- 例: `nyash.toml`
```
[env]
NYASH_JIT_THRESHOLD = "1"
NYASH_CLI_VERBOSE = "1"
NYASH_PLUGIN_POLICY = "off"   # compat: NYASH_DISABLE_PLUGINS = "1"
```

起動時に `nyash` は `[env]` の値を `std::env` に適用します（src/config/env.rs）。最新の推奨セットは [docs/guides/env-variables.md](../guides/env-variables.md) を参照してください。

## コア運用セット（最小）
- NYASH_CLI_VERBOSE: CLI の詳細ログ（"1" で有効）
- NYASH_PLUGIN_POLICY: プラグインロード方針（`auto|off|force`）
  - 互換: `NYASH_DISABLE_PLUGINS=1`（off 相当）

## JIT（共通）
- NYASH_JIT_THRESHOLD: JIT 降下開始の閾値（整数）
- NYASH_JIT_EXEC: JIT 実行（"1" で有効）
- NYASH_JIT_HOSTCALL: ホストコール経路の有効化
- NYASH_JIT_PHI_MIN: PHI(min) 合流の最適化ヒント
- NYASH_JIT_NATIVE_F64: f64 のネイティブ ABI 利用（実験的）
- NYASH_JIT_NATIVE_BOOL: bool のネイティブ ABI 利用（実験的）
- NYASH_JIT_ABI_B1: B1 返り値 ABI を要求（実験的）
- NYASH_JIT_RET_B1: bool 返り値ヒント（実験的）

## JIT トレース/ダンプ
- NYASH_JIT_DUMP: JIT IR/CFG ダンプ（"1" で有効）
- NYASH_JIT_DOT: DOT 出力先ファイル指定でダンプ暗黙有効
- NYASH_JIT_TRACE_BLOCKS: ブロック入場ログ
- NYASH_JIT_TRACE_BR: 条件分岐ログ
- NYASH_JIT_TRACE_SEL: select のログ
- NYASH_JIT_TRACE_RET: return 経路のログ
- NYASH_JIT_EVENTS_COMPILE: コンパイルイベント JSONL を出力
- NYASH_JIT_EVENTS_PATH: イベント出力パス（既定: events.jsonl）

## Async/Runtime
- NYASH_AWAIT_MAX_MS: await の最大待機ミリ秒（既定 5000）
- （今後）タスク/スケジューラ関連の変数は `runtime.*` 名で集約予定

## LLVM/AOT
- NYASH_LLVM_FEATURE: LLVM機能選択（"llvm"(default) または "llvm-inkwell-legacy"）
- LLVM_SYS_180_PREFIX: LLVM 18 のパス指定（llvm-inkwell-legacy使用時のみ必要）
- NYASH_LLVM_VINVOKE_RET_SMOKE, NYASH_LLVM_ARRAY_RET_SMOKE: CI 用スモークトグル
- NYASH_LLVM_OBJ_OUT: LLVM経路で生成する `.o` の出力パス（Runner/スクリプトが尊重）
- NYASH_AOT_OBJECT_OUT: AOT パイプラインで使用する `.o` 出力ディレクトリ/パス
- NYASH_LLVM_USE_HARNESS: "1" で llvmlite ハーネス経路を有効化（MIR(JSON)→Python→.ll→llc→.o）
 - NYASH_NY_LLVM_COMPILER: ハーネス用 ny-llvmc のフルパス（未設定時は `target/release/ny-llvmc` を自動推定）
 - NYASH_EMIT_EXE_NYRT: emit‑exe 時の nyrt ライブラリの探索ディレクトリ（例: `target/release`）

### FFI / extern_c（Phase 15.76）
- HAKO_FFI_ALLOW_LIST: 追加許可するシンボルをカンマ区切りで指定（例: `llvm_compile_mir_to_object`）
- HAKO_FFI_ALLOW_ALL: 1 ですべて許可（開発専用。CI/配布では禁止）
- HAKO_FFI_LIB_PATHS: バックエンドlib探索パス（`:`区切り; 例: `$(pwd)/target/release`）
  - 既定探索: `./target/release`, `$NYASH_ROOT/target/release`, `.`

### LLVM Feature 詳細
- **llvm** (デフォルト): llvmlite Python ハーネス使用、LLVM_SYS_180_PREFIX不要
- **llvm-inkwell-legacy**: Rust inkwell bindings使用、LLVM_SYS_180_PREFIX必要

## 管理方針（提案）
- コード側: `src/config/env.rs` を単一の集約窓口に（JIT は `jit::config` に委譲）。
- ドキュメント側: 本ファイルを単一索引にし、用途別に追加。
- 設定ファイル: `nyash.toml` の `[env]` で標準化（ブランチ/CI での一括制御）。
- 将来: `nyash env print/set` の CLI サブコマンドを追加し、実行前に `.env`/toml 反映と検証を行う。

## 実行出力（整形 / ノイズ抑制）
- NYASH_NYRT_SILENT_RESULT: 1 でランタイムの末尾出力を抑制し、`Result: <n>` のみに整形（比較用途）

## MIR Cleanup (Phase 11.8) 用トグル（段階導入）
- NYASH_MIR_ARRAY_BOXCALL: ArrayGet/Set → BoxCall 変換を有効化
- NYASH_MIR_REF_BOXCALL: RefGet/Set → BoxCall 変換を有効化
- NYASH_MIR_CORE13: Core‑13 セットの一括有効（将来拡張）
- NYASH_MIR_CORE13_PURE: [Deprecated / No‑Op] Core‑13 純化モードは撤廃され、このフラグは無視されます（`NYASH_CLI_VERBOSE=1` 時に非推奨メッセージのみ出力）。通常の Core‑13 は既定ONのままです。
