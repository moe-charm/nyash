# Nyash Box → C ABI → Multi‑Language FFI

要約（Abstract）
- Nyash 言語で高レベルに実装した Box を LLVM AOT でオブジェクト化し、安定した C ABI をエクスポートすることで、Python / Rust / Go / Node.js / C++ など任意言語から単一の .so/.dll を即利用可能にする開発手法を提案する。内部では NyRT の C ABI を用いて組み込み Box（String/Array/Map 等）を安全に操作し、外部には簡潔で言語非依存の API を提供する。C ABI → C ABI の多層構成により、実装の生産性と配布・統合の容易さを両立する。

背景と課題
- 既存のライブラリ開発では、C/C++ 実装＋各言語向けバインディングの整備が恒常的なコストとなる。
- 高速化や保守を優先すると低レベル実装が増え、開発速度や品質保証（型・所有権）が課題化する。
- Nyash には Box 抽象と NyRT（ランタイム）があり、文字列・配列などの基本操作を統一的に扱える基盤が存在する。

提案（Nyash×C ABI 多層アーキテクチャ）
- 外層（公開）：安定した C ABI（関数名・引数/戻り値・所有権規約）
- 内層（実装）：Nyash で Box を高レベル記述し、NyRT の C ABI を通じて組み込み Box を操作
- ビルド：Nyash → LLVM IR/obj（PIC）→ .so/.dll/.dylib（NyRT にリンク）

呼び出し経路（概念）
- Host(App/Script) → FFI(C ABI) → lib<your_box>.so → NyRT(C ABI) → OS/stdlib

API/ABI 設計指針（最小）
- 基本型：i64 / f64 / bool / (ptr,len) 文字列・バイト列
- 文字列返却：ヒープ確保して返す→呼び手が `ny_free_str(void*)` で解放
- エラー：int 戻り値（0=OK, 非0=ERR）；詳細は必要なら out-param or errno スタイル
- 可視性：公開関数は `visibility=default`，内部記号は `hidden`
- 初期化：`ny_init()/ny_shutdown()` を用意（方針により省略可）。NyRT と二重初期化しない運用規約を明示
- ビルド：全 .o を `-fPIC`，NyRT は動的（推奨）または PIC な静的リンクに統一

ケーススタディ（PoC：StringBuilderBox）
- Nyash 実装（内部）
  - `birth()` で内部バッファ（ArrayBox）初期化
  - `append(string)` で push，`toString()` で連結
- 公開 C API（例）
  - `sb_handle* sb_new();`
  - `int sb_append(sb_handle*, const char* s, size_t n);`
  - `int sb_to_string(sb_handle*, char** out, size_t* out_len); // 呼び手が ny_free_str()`
  - `void sb_free(sb_handle*);
     void ny_free_str(void*);`
  - （任意）`int ny_init(); int ny_shutdown(); const char* ny_get_version();`

多言語からの利用例（断片）
- Python(ctypes)
  ```python
  from ctypes import *
  lib = cdll.LoadLibrary('./libstringbuilder.so')
  lib.sb_new.restype = c_void_p
  h = lib.sb_new()
  lib.sb_append(h, b"hello", 5)
  out = c_void_p(); ln = c_size_t()
  lib.sb_to_string(h, byref(out), byref(ln))
  s = string_at(out.value, ln.value).decode('utf-8')
  lib.ny_free_str(out)
  lib.sb_free(h)
  ```
- Rust(ffi)
  ```rust
  extern "C" {
      fn sb_new() -> *mut core::ffi::c_void;
      fn sb_append(h:*mut core::ffi::c_void, p:*const u8, n:usize) -> i32;
      fn sb_to_string(h:*mut core::ffi::c_void, out:*mut *mut u8, len:*mut usize) -> i32;
      fn ny_free_str(p:*mut core::ffi::c_void);
      fn sb_free(h:*mut core::ffi::c_void);
  }
  ```

評価計画（実務＋学術）
- 生産性：実装 LoC / 時間，改修差分の小ささ（C 実装比）
- 性能：連結 10^6 回のスループット，FFI 境界のオーバーヘッド（まとめ API で最適化）
- 信頼性：ASan/valgrind，文字列所有権（リークゼロ）
- 多言語統合：Python/Rust/Go/Node で同一 .so をスモーク
- 再利用性：同一成果物を Nyash プラグイン(v2) と汎用 C ライブラリの両用途で利用

限界とリスク
- NyRT とのリンク方針不一致（動的/静的）やバージョン差異で未定義動作の恐れ → 版照合 API と CI を用意
- 返却バッファの所有権取り決めが曖昧だとリーク → `ny_free_str()` を ABI の一部に
- マルチスレッドでの再入性 → 仕様として明示（必要に応じロック/TLS）

ロードマップ
1) PoC：StringBuilderBox を .so 化（PIC＋NyRTリンク），Python/Rust スモークとリーク検査
2) Nyash プラグイン(v2) ラッパ追加（`nyash.toml` に登録）
3) JSON/文字列ユーティリティの横展開（共通 ABI ルール化）
4) ABI 自動生成：Nyash シグネチャ → C ヘッダ/ラッパ生成
5) パッケージ化テンプレ（ヘッダ＋pkg-config＋CI スモーク）

関連文書
- ScopeBox/LoopForm（制御構造の正規化構想）：docs/guides/loopform.md
- Nyash LLVM/LlvmPy 概要：docs/design/LLVM_LAYER_OVERVIEW.md
- Seam‑aware JSON Unification（前処理と決定論実装）：papers/paper-y-seam-aware-json-unification/README.md

