# Phase 10.5 – Python ネイティブ統合（Embedding & FFI）/ JIT Strict 化の前倒し
*(旧10.1の一部を後段フェーズに再編。Everything is Plugin/AOTの基盤上で実現)*

NyashとPythonを双方向に“ネイティブ”接続する前に、JITの開発・検証効率を最大化するため、VM=仕様/JIT=高速実装 という原則に沿った「JIT Strict モード」を前倒し導入し、フォールバック起因の複雑性を排除する。

## 📂 サブフェーズ構成（10.5a → 10.5e）

先行タスク（最優先）
- 10.5s JIT Strict モード導入（Fail-Fast / ノーフォールバック）
  - 目的: 「VMで動く＝正。JITで動かない＝JITのバグ」を可視化、開発ループを短縮
  - 仕様:
    - // @jit-strict または NYASH_JIT_STRICT=1 で有効化
    - Lowerer: unsupported>0 の場合はコンパイルを中止（診断を返す）
    - 実行: JIT_ONLY と併用時はフォールバック禁止（失敗は明示エラー）
    - シム: 受け手解決は HandleRegistry 優先。param-index 互換経路は無効化
  - DoD:
    - Array/Map の代表ケースで Strict 実行時に compile/runtime/シムイベントの整合が取れる
    - VM=JIT の差が発生したときに即座に落ち、原因特定がしやすい（フォールバックに逃げない）

### 10.5a 設計・ABI整合（1–2日）
- ルート選択: 
  - Embedding: NyashプロセスにCPythonを埋め込み、PyObject*をハンドル管理
  - Extending: Python拡張モジュール（nyashrt）を提供し、PythonからNyashを呼ぶ
- ABI方針:
  - ハンドル: TLV tag=8（type_id+instance_id）。Pythonオブジェクトは `PyObjectBox` として格納
  - 変換: Nyash ⇄ Python で Bool/I64/String/Bytes/Handle を相互変換
  - GIL: birth/invoke/decRef中はGIL確保。AOTでも同等

### 10.5b PyRuntimeBox / PyObjectBox 実装（3–5日）
- `PyRuntimeBox`（シングルトン）: `eval(code) -> Handle` / `import(name) -> Handle`
- `PyObjectBox`: `getattr(name) -> Handle` / `call(args...) -> Handle` / `str() -> String`
- 参照管理: `Py_INCREF`/`Py_DECREF` をBoxライフサイクル（fini）に接続
- プラグイン化: `nyash-python-plugin`（cdylib/staticlib）で `nyplug_python_invoke` を提供（将来の静的同梱に対応）

### 10.5c 境界の双方向化（3–5日）
- Nyash→Python: BoxCall→plugin_invokeでCPython C-APIに橋渡し
- Python→Nyash: `nyashrt`（CPython拡張）で `nyash.call(func, args)` を提供
- エラーハンドリング: 例外は文字列化（tag=6）でNyashに返却、またはResult化

### 10.5d JIT/AOT 統合（3–5日）
- JIT: `emit_plugin_invoke` で Pythonメソッド呼びを許可（ROから開始）
- AOT: libnyrt.a に `nyash.python.*` シム（birth_hなど）を追加し、ObjectModuleの未解決を解決
- 静的同梱経路: `nyrt` に type_id→`nyplug_python_invoke` ディスパッチテーブルを実装（または各プラグイン名ごとのルータ）。第一段は動的ロード優先

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
