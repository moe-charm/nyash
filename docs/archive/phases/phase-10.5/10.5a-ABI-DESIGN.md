# Phase 10.5a – Python 統合 ABI 設計（Draft）

目的: Everything is Plugin/AOT の既存基盤上に Python を最小リスクで統合するための ABI と型・メソッド定義を固定する。

## スコープ（10.5a）
- v2 プラグイン ABI 準拠（`nyash_plugin_abi/init/invoke`）の Python プラグイン雛形を作成
- 2 Box を定義: `PyRuntimeBox(type_id=40)`, `PyObjectBox(type_id=41)`
- メソッド ID の割り当てと TLV 方針を文書化（実装は 10.5b 以降）

## TLV マッピング（現行運用）
- 1 = Bool (1 byte)
- 2 = I32  (4 bytes, LE)
- 3 = I64  (8 bytes, LE)
- 4 = F32  (4 bytes, LE)
- 5 = F64  (8 bytes, LE)
- 6 = String (UTF-8, n bytes)
- 7 = Bytes  (opaque, n bytes)
- 8 = Handle/BoxRef (`u32 type_id || u32 instance_id`)

備考: 既存ドキュメントには古い表記の混在があるため、VM_README 準拠で統一。

## Box とメソッド設計

### PyRuntimeBox (type_id=40)
- birth(0): ランタイムの生成（後続で GIL 初期化などを担当）。戻り値: `instance_id`（非 TLV, u32 LE）
- eval(1, code: String): Python コードを評価して `PyObjectBox` を返す。戻り値: `Handle(tag=8)`
- import(2, name: String): `__import__(name)` または `importlib.import_module`。戻り値: `Handle(tag=8)`
- fini(MAX): ランタイム破棄（GIL 終了・クリーンアップ）

### PyObjectBox (type_id=41)
- birth(0): 予約（通常は runtime 側から生まれる）
- getattr(1, name: String): 属性取得 → `Handle(tag=8)`
- call(2, args: TLV...): 可変長引数。初期段は I64/String/Bool/Bytes/Handle のサブセットに限定。戻り値: `Handle(tag=8)`
- str(3): Python 側で `PyObject_Str` → String へ。戻り値: `String(tag=6)`
- fini(MAX): 参照カウント `Py_DECREF` に対応（後続）

## 参照管理・GIL（概要）
- GIL: birth/invoke/fini の入口で確保し、出口で解放（再入を許容）。
- 参照: `PyObjectBox` は生成時に `INCREF`、`fini` で `DECREF`。ランタイム終了時に孤立検知の簡易テストを導入。

## 設定ファイル（nyash.toml）
- `libnyash_python_plugin.so` を 2 Box 含む形で登録（path/type_id/method_id を固定）
- JIT/VM 側は既存の `plugin_invoke` 経由で呼び出し（AOT は 10.5d で `libnyrt.a` にシム追加）

## 次フェーズ（10.5b 以降）
- 10.5b: `PyRuntimeBox`/`PyObjectBox` 実装（CPython 埋め込み、最小 RO 経路）
- 10.5c: Python→Nyash 方向（CPython 拡張 `nyashrt`）
- 10.5d: JIT/AOT 連携（`emit_plugin_invoke` 対応・静的リンクシム）
- 10.5e: サンプル・テスト・ドキュメント

---

補足: 本ドキュメントは「設計の固定」を目的とし、実装は段階的に進める。タグ/ID は既存と衝突しない値を選定した（40/41）。
