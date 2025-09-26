# Nyash ABI Minimal Core and Evolution Strategy

目的
- 極小コアのABIを長期安定化（仕様固定）し、将来拡張は交渉＋予約＋フォールバックで吸収する。
- VM/JIT/プラグインを同じ枠組み（TypeBox + vtable + NyrtValue）に統一する。

最小コア（安定化対象）
- NyrtValue: 16B固定 `#[repr(C)]`（`tag:u32 | reserved:u32 | payload:u64`）
- InstanceHeader: 先頭に `vtbl:*const NyVTable, refcnt:u32, flags:u32`（実体は不透明）
- NyMethodFn: `fn(ctx:*mut VmCtx, recv:*mut InstanceHeader, args:*const NyrtValue, argc:u32, out:*mut NyrtValue) -> NyStatus`
- NyVTable: `abi_version:u32, type_id:u128, methods:*const NyVEntry, methods_len:u32, /*reserved*/`
- NyVEntry: `NyMethodSig + NyMethodFn`
- Selector: `selector_id:u64`（xxh3_64推奨）。衝突時は文字列による二段判定。

エボリューション・ハッチ（逃げ道）
- struct_size: すべての公開構造体先頭に `struct_size:u32` を置く。
- Feature bits: `RuntimeApi` で `feature_bits:u128` を返し機能交渉。
- 予約領域: `selector_id` の上位レンジ（例: `0xF000...`）は内部/拡張に予約。
- フォールバック: vtable未解決 → PIC → 汎用ディスパッチへ必ず落ちる。
- VmCtxは不透明: `get_ctx_api(version)` で関数テーブルを取得（中身は自由）。

互換ポリシー
- SemVer+ABI: 例 `nyash 0.13.0 / ABI 1`。破壊的変更は ABI++、非破壊は feature_bits++。
- 既存selectorの意味は変えない/削除しない（非推奨→2リリース猶予）。
- vtable項目の順序非依存（二分探索前提）。
- ランタイムは ABI(現行, 現行-1) をサポート。

安全ルール
- 例外横断禁止: 例外/パニックは NyStatus に畳み込む（unwind禁止）。
- 所有権: 越境は基本 own。borrowは同期フレーム内のみ。handleはHostHandle経由。
- メモリ規約: retain/release か refcnt を明文化。ヘッダに将来予約フラグ。
- 実装前提: LE/64bit固定（flagsにも宣言）。

性能レール
- ディスパッチ順: vtable直行 → PIC(最大4) → 汎用。
- AOT/JIT: `len()` hoist、境界チェック併合、純関数/no_throw の軽量inline。
- メトリクス: ICヒット率/ガード失敗率/レイテンシを収集（`NYASH_ABI_TRACE=1`）。

“全部Box”統一ポイント
- ユーザー/プラグイン/内蔵 = 同じ `InstanceHeader + NyVTable`。
- TypeRegistry に `register(Arc<NyVTable>)`。`NYASH_ABI_STRICT=1` で未登録/不一致は即エラー。

最小ヘッダ（C向けスケッチ）
```c
typedef struct { uint32_t struct_size, tag, reserved; uint64_t payload; } NyrtValue; // 16B
typedef struct InstanceHeader InstanceHeader; // opaque
typedef int32_t NyStatus;
typedef NyStatus (*NyMethodFn)(void* ctx, InstanceHeader* recv,
                               const NyrtValue* args, uint32_t argc, NyrtValue* out);
typedef struct { uint32_t struct_size, abi_version; unsigned __int128 type_id; const char* type_name; 
                 void (*drop_fn)(InstanceHeader*); InstanceHeader* (*clone_fn)(const InstanceHeader*);
                 const struct NyVEntry* methods; uint32_t methods_len; /* reserved */ } NyVTable;
typedef struct { uint32_t struct_size, abi_version; uint64_t selector_id; uint16_t arity, flags, reserved; } NyMethodSig;
typedef struct NyVEntry { NyMethodSig sig; NyMethodFn fn_ptr; } NyVEntry;
```

テストとツール
- ABI Compliance: C/C++/Rust雛形を自動生成→ロード→全selector呼出で比較。
- ワイヤ互換: 現行/前ABIの双方でロード→結果一致をCIで検証。
- ヘッダ自動生成: `cbindgen/abi-dumper` で差分チェック。
- 同一実行: VM/JIT の二重実行で戻り値/副作用/ログの一致を比較。

即実行チェックリスト
- [ ] `nyash_abi.{rs,h}` に最小コア定義（struct_size/feature_bits付き）。
- [ ] `RuntimeApi` と feature_bits 交渉の雛形。
- [ ] `execute_boxcall`: vtable直行→PIC→汎用の三段整備（STRICTガード）。
- [ ] TypeRegistry: Array/Map/String の "get/set/len" を登録→vtable優先呼び出し。
- [ ] ABIコンプライアンステストと同一実行テストの設計をdocsに反映。
