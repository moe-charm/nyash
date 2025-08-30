# CURRENT TASK (Phase 10.5c)

目的: Handle-First + by-name を軸に、Python統合（PyRuntimeBox/PyObjectBox）を汎用・安全に実装する。最適化は後段。

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
