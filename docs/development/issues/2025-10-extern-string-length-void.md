# Extern(String.length) — Void 伝播による TypeError の調査と対策（2025‑10‑16）

状態: 調査完了・修正方針合意（実装中）

## 症状
- quick プロファイルで `json_query_*` など 25 件 FAIL。
- 代表エラー: `Type error: nyrt.string.length expects String`
- 再現: `apps/examples/json_query/main.nyash` の `eval_path_text()` で `path.size()`、MIR 正規化で `call_extern nyrt.string.length(recv)` に降格、実行時 `recv=Void`。

## 原因（3 層構造）
1) Builder の素材化不足（既存ギャップ）
   - Extern 正規化（`nyrt.string.length`）の前に受領者の in‑block Copy を保証できていない。
   - φ 合流で未定義が混入しうるパスがある。
2) 寛容ガード（マスク）
   - VM の Copy 寛容ガード（未定義→Void 初期化）により、本来 Fail‑Fast すべき箇所が通過。
   - 結果として Void が φ を通って Extern 引数に到達する。
3) 露呈のきっかけ
   - Static→singleton 正規化で Extern 経路が prominent になり、Method 経路の保険に頼らない形に。

## 再現ログ
- MIR: `--dump-mir` で `@Main.eval_path_text/2` に `call_extern nyrt.string.length(%recv)` が並ぶ。
- 実行: `NYASH_DEBUG_STRING_LEN=1` で `unexpected arg=Void` の stderr 出力を確認。

## 対策（順序）
1) Builder finalize/repair の徹底
   - Extern 呼び出しにも `finalize_call_operands` を適用し、受領者を in‑block Copy で素材化（Method と同等の網）。
   - 正規化箇所（string_length / emit string dotted）前後で materialize を入れる。
2) VM の Copy 寛容ガードを既定 OFF
   - `NYASH_VM_TOLERATE_VOID` 未設定時は Fail‑Fast。寛容は開発時限定。
3) 回帰スモーク
   - quick-selfhost に「φ 合流 String で Extern length」を 1 本追加。

## 関連ファイル
- Copy: `src/backend/mir_interpreter/handlers/arithmetic.rs`
- Extern(String): `src/backend/mir_interpreter/extern_adapter/extern_string.rs`
- Trampoline: `src/backend/mir_interpreter/handlers/calls/trampolines.rs`
- Builder normalize: `src/mir/builder/builder_calls/emit.rs`, `src/mir/builder/normalize/string_length.rs`

## 備考
- Router 表（builtin/plugin の統一）は問題の中心ではない（String.size は Builder が Extern へ正規化）。
- ENV デバッグ: `NYASH_DEBUG_HOST_SLOT=1`, `NYASH_DEBUG_STRING_LEN=1`。

