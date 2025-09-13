# Nyash Environment Variables (管理棟ガイド)

本ドキュメントは Nyash の環境変数を用途別に整理し、最小限の運用セットを提示します。`nyash.toml` の `[env]` で上書き可能（起動時に適用）。

- 例: `nyash.toml`
```
[env]
NYASH_JIT_THRESHOLD = "1"
NYASH_CLI_VERBOSE = "1"
NYASH_DISABLE_PLUGINS = "1"
```

起動時に `nyash` は `[env]` の値を `std::env` に適用します（src/config/env.rs）。

## コア運用セット（最小）
- NYASH_CLI_VERBOSE: CLI の詳細ログ（"1" で有効）
- NYASH_DISABLE_PLUGINS: 外部プラグインを無効化（CI/再現性向上）

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
- LLVM_SYS_180_PREFIX: LLVM 18 のパス指定
- NYASH_LLVM_VINVOKE_RET_SMOKE, NYASH_LLVM_ARRAY_RET_SMOKE: CI 用スモークトグル
- NYASH_LLVM_OBJ_OUT: Rust LLVM 経路で生成する `.o` の出力パス（Runner/スクリプトが尊重）
- NYASH_AOT_OBJECT_OUT: AOT パイプラインで使用する `.o` 出力ディレクトリ/パス
- NYASH_LLVM_USE_HARNESS: "1" で llvmlite ハーネス経路を有効化（MIR(JSON)→Python→.ll→llc→.o）

## 管理方針（提案）
- コード側: `src/config/env.rs` を単一の集約窓口に（JIT は `jit::config` に委譲）。
- ドキュメント側: 本ファイルを単一索引にし、用途別に追加。
- 設定ファイル: `nyash.toml` の `[env]` で標準化（ブランチ/CI での一括制御）。
- 将来: `nyash env print/set` の CLI サブコマンドを追加し、実行前に `.env`/toml 反映と検証を行う。

## MIR Cleanup (Phase 11.8) 用トグル（段階導入）
- NYASH_MIR_ARRAY_BOXCALL: ArrayGet/Set → BoxCall 変換を有効化
- NYASH_MIR_REF_BOXCALL: RefGet/Set → BoxCall 変換を有効化
- NYASH_MIR_CORE13: Core‑13 セットの一括有効（将来拡張）
- NYASH_MIR_CORE13_PURE: Core‑13 純化モード（"1" で有効）。最終MIRは13命令のみ許可され、Load/Store などは `env.local.get/set`、`new` は `env.box.new` 経由へ強制正規化。禁制命令が残存するとコンパイルエラーで早期失敗。
