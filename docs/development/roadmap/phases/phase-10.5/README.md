# Phase 10.5 – ネイティブ基盤固め + Python ネイティブ統合
*(旧10.1の一部を後段フェーズに再編。まずネイティブ/AOT基盤を固め、その上でPythonを統合する方針に整理)*

本フェーズでは方針を明確化する：実行はVMが唯一の基準系、JITは「EXE/AOT生成専用のコンパイラ」として分離運用する。

アーキテクチャの整理（決定）
- 開発/デバッグ: MIR → VM（完全実行）
- 本番/配布:     MIR → JIT（CLIF）→ OBJ → EXE（完全コンパイル）

ポイント
- フォールバック不要/禁止: JITが未対応ならコンパイルエラー。VMへは落とさない。
- 役割分担の明確化: VM=仕様/挙動の唯一の基準、JIT=ネイティブ生成器。
- プラグイン整合: VM/EXEとも同一のBID/FFIプラグインを利用（Everything is Plugin）。

## 📂 サブフェーズ構成（10.5s → 10.5e）

先行タスク（最優先）
- 10.5s JIT Strict/分離の確定（Fail-Fast / ノーフォールバック） [DONE]
  - 目的: 「VM=実行・JIT=コンパイル」の二系統で混在を排除し、検証を単純化
  - 仕様:
    - JITは実行経路から外し、`--compile-native`（AOT）でのみ使用
    - Lowerer/Engine: unsupported>0 または fallback判定>0 でコンパイル中止（Fail-Fast）
    - 実行: VMのみ。フォールバックという概念自体を削除
  - DoD:
    - CLIに `--compile-native` を追加し、OBJ/EXE生成が一発で通る
    - VM実行は常にVMのみ（JITディスパッチ既定OFF）。

### 10.5a（Python）設計・ABI整合（1–2日）
- ルート選択: 
  - Embedding: NyashプロセスにCPythonを埋め込み、PyObject*をハンドル管理
  - Extending: Python拡張モジュール（nyashrt）を提供し、PythonからNyashを呼ぶ
- ABI方針:
  - ハンドル: TLV tag=8（type_id+instance_id）。Pythonオブジェクトは `PyObjectBox` として格納
  - 変換: Nyash ⇄ Python で Bool/I64/String/Bytes/Handle を相互変換
  - GIL: birth/invoke/decRef中はGIL確保。AOTでも同等

### 10.5b ネイティブビルド基盤の固め（AOT/EXE）（2–4日）
- 目的: Python統合の前に、AOT/EXE配布体験・クロスプラットフォーム実行の足回りを先に完成させる
- 範囲:
  - VMとJITの分離（JIT=EXE専用）とStrict運用の徹底
  - AOTパイプラインの実働（CLIF→.o→libnyrtリンク→EXE）
  - プラグイン解決のクロスプラットフォーム化（.so/.dll/.dylib、自動lib剥がし、検索パス）
  - Windowsビルド/リンク（clang優先、MSYS2/WSL fallback）
  - EXE出力の統一（`Result: <val>`）とスモークテスト
- DoD:
  - Linux/Windowsで `--compile-native` が通り、`plugins/` のDLL/so/dylibを自動解決
  - `tools/build_aot.{sh,ps1}` で配布しやすいEXEが生成される
  - `tools/smoke_aot_vs_vm.sh` でVM/EXEの出力照合が可能

### 10.5c PyRuntimeBox / PyObjectBox 実装（3–5日）
- `PyRuntimeBox`（シングルトン）: `eval(code) -> Handle` / `import(name) -> Handle`
- `PyObjectBox`: `getattr(name) -> Handle` / `call(args...) -> Handle` / `str() -> String`
- 参照管理: `Py_INCREF`/`Py_DECREF` をBoxライフサイクル（fini）に接続
- プラグイン化: `nyash-python-plugin`（cdylib/staticlib）で `nyplug_python_invoke` を提供（将来の静的同梱に対応）

### 10.5c 境界の双方向化（3–5日）
- Nyash→Python: BoxCall→plugin_invokeでCPython C-APIに橋渡し
- Python→Nyash: `nyashrt`（CPython拡張）で `nyash.call(func, args)` を提供
- エラーハンドリング: 例外は文字列化（tag=6）でNyashに返却、またはResult化

### 10.5d JIT/AOT 統合（3–5日）
- AOTパイプライン固定: Lower→CLIF→OBJ出力→`ny_main`+`libnyrt.a`リンク→EXE
- CLI: `nyash --compile-native file.nyash -o app` を追加（失敗は非ゼロ終了）
- libnyrt: `nyash.python.*` 等のシムを提供し、未解決シンボル解決
- ディスパッチ: type_id→`nyplug_*_invoke` の静的/動的ルート（第一段は動的優先）

### 10.5e サンプル/テスト/ドキュメント（1週間）
- サンプル: `py.eval("'hello' * 3").str()`、`numpy`の軽量ケース（import/shape参照などRO中心）
- テスト: GILの再入・参照カウントリーク検知・例外伝搬・多プラットフォーム
- ドキュメント: 使用例、制約（GIL/スレッド）、AOT時のリンク・ランタイム要件

## 🎯 DoD（定義）
- NyashからPythonコードを評価し、PyObjectをHandleで往復できる
- 代表的なプロパティ取得/呼び出し（RO）がJIT/VMで動作
- AOTリンク後のEXEで `py.eval()` 代表例が起動できる（動的ロード前提）
 - 10.5s Strict: VM=仕様/JIT=高速実装の原則に基づき、フォールバック無しで fail-fast が機能

## ⌛ 目安
| サブフェーズ | 目安 |
|---|---|
| 10.5a 設計 | 1–2日 |
| 10.5b 実装 | 3–5日 |
| 10.5c 双方向 | 3–5日 |
| 10.5d JIT/AOT | 3–5日 |
| 10.5e 仕上げ | 1週間 |

## ⚠️ リスクと対策
- GILデッドロック: 入口/出口で厳格に確保/解放。ネスト呼び出しの方針を文書化
- 参照カウント漏れ: BoxライフサイクルでDECREFを必ず実施、リークテストを追加
- リンク/配布: Linux/macOS優先。WindowsのPythonリンクは後段で対応
- 性能: RO先行でJITに寄せ、ミューテーションはポリシー制御

---

次は 10.5a（設計・ABI整合）から着手。Everything is Plugin / libnyrt シムの成功パターンをPythonにも適用し、最小リスクで“Pythonネイティブ”を実現する。
