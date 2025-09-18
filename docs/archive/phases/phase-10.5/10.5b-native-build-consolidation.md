# 10.5b – ネイティブビルド基盤の固め（AOT/EXE）

Python統合を本格化する前に、配布可能なネイティブ実行ファイル（EXE）の足回りを先に完成させる。JITは実行エンジンから外し、EXE生成専用のコンパイラとして運用する。

## 🎯 目的
- VM=実行、JIT=EXE（AOT）の二系統を明確化（フォールバックなし/Fail-Fast）
- CLIF→.o→`libnyrt`リンク→EXEのパイプラインを実効化
- プラグイン解決をクロスプラットフォームに（.so/.dll/.dylib、自動lib剥がし、検索パス）
- Windowsを含む実用的な配布体験を整備

## 🧩 範囲
- JIT分離・Strict運用（Fail-Fast/No-fallback）
- AOTパイプライン: `--compile-native` と `tools/build_aot.{sh,ps1}`
- プラグインローダの拡張: 拡張子変換/`lib`剥がし、`plugin_paths`+`NYASH_PLUGIN_PATHS`
- Windowsリンク: clang優先（`nyrt.lib`/`libnyrt.a`両対応）、bash+cc fallback
- 観測/EXE出力の統一: `Result: <val>`、終了コード=<val>

## ✅ 成果（DoD）
- `cargo build --release --features cranelift-jit` の後、
  - Linux: `./tools/build_aot.sh examples/aot_min_string_len.nyash -o app && ./app`
  - Windows: `powershell -ExecutionPolicy Bypass -File tools\build_aot.ps1 -Input examples\aot_min_string_len.nyash -Out app.exe && .\app.exe`
- プラグインは `.so` 記述でも各OSで自動解決（.dll/.dylib へ変換、lib剥がし）
- `tools/smoke_aot_vs_vm.sh` で VM/EXE の `Result:` 行比較が可能（差異は警告表示）

## 🔧 実装メモ
- `src/runtime/plugin_loader_v2.rs` に `resolve_library_path()` を追加:
  - OS別拡張子、Windowsの`lib`剥がし、`plugin_paths`探索
- `src/config/nyash_toml_v2.rs` に `NYASH_PLUGIN_PATHS` を追加（`;`/`:`区切り）
- `AotConfigBox` に `set_plugin_paths()` 追加（env同期）
- `crates/nyrt` の EXE出力統一（`Result:`/exit code）
- Windows: `tools/build_aot.ps1`（clang→bash fallback）、Linux: `tools/build_aot.sh`

## 📌 次（10.5c 以降）
- PyRuntimeBox/PyObjectBox（RO優先）
- Python ABIルータを `libnyrt` に同梱（type_id→invokeディスパッチ）
- 配布用パッケージ整備（nyash.toml/プラグイン配置ガイドの最終化）

