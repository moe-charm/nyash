# CURRENT TASK (Phase 10.7 workbench + 10.5c 継続)

直近スナップショット（2025-08-30 更新）

Current State

- Plugin-First/Handle-First/TLVはAOT/VMで安定（10.5e完了状態を継続）
- 10.6計画（Thread-Safety/Scheduler）と10.7計画（トランスパイルAll-or-Nothing）を確定
- Nyash-onlyパイプライン（tools/pyc）を開始（Parser/CompilerはNyashで実装方針）
- include式の最小実装を追加（式でBoxを返す／1ファイル=1static box）
  - インタプリタ: include式は実行時評価
  - VM/AOT: MIRビルダーが取り込み先を同一MIRに連結（MIR命令は増やさない）
  - nyash.tomlの[include.roots]でルート解決（拡張子省略、index.nyash対応）
- tools/pycをモジュール分割
  - tools/pyc/pyc.nyash（エントリ: includeでPyIR/PythonParserNy/PyCompilerを取り込み）
  - tools/pyc/PyIR.nyash, PythonParserNy.nyash, PyCompiler.nyash（Nyash-only実装）

How To Run（Nyash-only）

- VM: `NYASH_PY_CODE=$'def main():\n  return 42' ./target/release/nyash --backend vm tools/pyc/pyc.nyash`
  - 出力: Parser JSON → IR（return 42）→ 生成Nyashソース（現状は骨組み）
- include動作サンプル: `./target/release/nyash --backend vm examples/include_main.nyash`（Math.add(1,2)=3）

進捗（2025-08-30 夜）

- include: 循環検出を追加（インタプリタ/VM収集器ともにロード中スタックで経路出力）。examples/cycle_a/b で検証
- tools/pyc: 最小IR（return定数）→Nyash生成を通し、出力をprintまで接続
- 文字列基盤: VMにString統一ブリッジを着手（内部StringBoxとプラグインStringBoxの比較互換、内部Stringメソッドのフォールバック）
- 追加プラグイン（小粒・基底）
  - RegexBox（compile/isMatch/find/replaceAll/split）: examples/regex_min.nyash
  - EncodingBox（utf8/base64/hex）: examples/encoding_min.nyash
  - TOMLBox（parse/get/toJson）: examples/toml_min.nyash
  - PathBox（join/dirname/basename/extname/isAbs/normalize）: examples/path_min.nyash

Next Steps（優先順・更新）

1. String統一ブリッジの完了
   - VM: 内部String受けのフォールバックを全パスで拾う（length/isEmpty/charCodeAt/concat/+）
   - Interpreter: 同等のフォールバック/正規化（比較・結合・代表メソッド）
   - 混在比較/結合の回帰ケース追加（内部/プラグイン/プリミティブ混在）
2. tools/pyc: IR→Nyashの反映強化（return/If/Assignを安定化、Strictスイッチ連動）
3. Strictスイッチ: tools/pyc（unsupported_nodes非空でErr、envでON/OFF）
4. CLI隠しフラグ `--pyc`/`--pyc-native`（Parser→Compiler→AOTの一本化導線）
5. 最小回帰（VM/AOTの差分記録）とdocs追補（include/exportとpyc、Regex/Encoding/TOML/PathのAPI概要）

Env Keys（pyc）

- NYASH_PY_CODE: Pythonソース文字列（Nyash-onlyパイプライン/Parser用）
- NYASH_PY_IR: IR(JSON)直接注入（Rust雛形Compilerの確認用・オプション）

目的: Handle-First + by-name を軸に、Python統合（PyRuntimeBox/PyObjectBox）を汎用・安全に実装する。最適化は後段。さらに10.7のNyash-onlyトランスパイルC2（pyc）を最小構成で立ち上げる。

ステータス（2025-08-30 更新）

- フェーズ: 10.5c 汎用Handle/TLV実装の拡張（Python統合開始）
- 方針: 「綺麗に作って動かす」= ハードコーディング排除・Handle/TLV統一・最適化は後回し

10.5b 完了項目（橋渡し済み）

- by-name シム（getattr/call）を実装（JIT/AOT）し、Lowerer から a0 を `nyash.handle.of` で確実にハンドル化して呼び出し
- 引数 a1/a2 はハンドル優先／なければレガシー参照から TLV 構築（String/Integer はプリミティブ化）
- 汎用 birth シムを追加
  - `nyash.box.birth_h(type_id:i64)->i64`（JIT/AOT）
  - `nyash.box.birth_i64(type_id:i64, argc:i64, a1:i64, a2:i64)->i64`（JIT/AOT）
  - Lowerer: NewBox（引数無し）は birth_h に統一。引数ありは安全なケース（Integer const／引数が既にハンドル）だけ birth_i64 に段階導入
- AOT: examples/aot_py_math_sqrt_min.nyash で Strict でも .o 生成を確認（target/aot_objects/main.o）
- ログ
  - AOT: NYASH_CLI_VERBOSE=1 で birth_h の可視化
  - JIT: events で by-name/birth の観測（必要十分の最小限）

10.5c 着手項目（進行中）
- Lowerer: PluginInvoke（type_id/method_id & by-name）の Handle-First 配線を統一（a0を常にnyash.handle.of）
- JIT/AOT: birth（_h/_i64）と by-name シムでTLV生成を汎用化（String/Integerはプリミティブ化、他はHandle）
- Strict時のJIT実行停止（コンパイル専用）でVM=仕様の原則を徹底

非対応（後回し・最適化）

- StringBox 専用の known_string/再利用最適化
- 汎用的な定数プール／birth の可変長 TLV 一括最適化

次の作業（10.5c 続き）

1) FFI仕様の短文化（a0/a1/a2=Handle優先→TLV、レガシー抑止フラグ、戻りTLVのdecodeポリシー）
2) birth引数の一般化メモ（可変長TLV、例外時ハンドリング）
3) Python統合の最小チェーン（import→getattr→call）のAOT/VM双方での実装確認サンプル追加
4) ドキュメント更新（10.5c README/INDEX、FFIガイド）

合意済みルール

- まず汎用・安全に動かす（最適化は内部に隠し、後段）
- StringBox 等の個別特化は入れない。Handle/TLV で統一し、Box 追加を阻害しない
- Strict/Fail‑Fast を維持（fallback で隠さない）
