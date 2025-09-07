# Nyash FFI Calling Convention (Handle-First, TLV, v0)

目的: 10.5c の指針に沿って、JIT/AOT/VM 共通の最小・実用的な呼び出し規約を短文化。a0/a1/a2 は Handle-First を前提とし、戻りは TLV で統一。

## 要点（TL;DR）
- Handle-First: a0 は常に `nyash.handle.of(receiver)`（i64）。他の引数は TLV でエンコード。
- 引数TLV: String/Integer はプリミティブ化、それ以外は Handle(tag=8)。
- 戻りTLV: i64/f64/bool/string/bytes/handle をサポート。`returns_result` 指定時は Result 形状（VMが尊重）。
- by-name 経路: 型名未確定時は `*_invoke_by_name_*` シムで実行時解決（任意、推奨）。
- Strict 原則: Strict 時は JIT 実行を停止（VM=仕様の唯一の基準）。AOT 生成のみ許可。

## シム関数（最小）
```c
// 受け手は a0（i64: handle）で指定。a1/a2 は任意引数のプレースホルダ。
extern "C" long long nyash_plugin_invoke3_i64(
    long long type_id,
    long long method_id,
    long long argc,   // 受け手を含む総数（>=1）
    long long a0,     // receiver: nyash.handle.of(...)
    long long a1,
    long long a2
);

extern "C" double nyash_plugin_invoke3_f64(
    long long type_id,
    long long method_id,
    long long argc,
    long long a0,
    long long a1,
    long long a2
);

// by-name（型名未確定時に利用可能）最小シム
// 実装済み: getattr/call の i64系
extern "C" long long nyash_plugin_invoke_name_getattr_i64(long long argc, long long a0, long long a1, long long a2);
extern "C" long long nyash_plugin_invoke_name_call_i64(long long argc, long long a0, long long a1, long long a2);
```

## 引数の規約（TLV）
- a0 は常に受け手のハンドル（i64）。レシーバは TLV には含めず、直接 a0 から解決。
- a1/a2 は必要に応じて TLV バッファへ詰める（最大2引数の最小シム。将来拡張で可変長）。
- エンコード方針（VM/nyrt共通）
  - StringBox → TLV string(tag=6)
  - IntegerBox → TLV i64(tag=3)
  - BufferBox → TLV bytes(tag=7)（nyrtシムが自動検出）
  - それ以外の Box → TLV handle(tag=8, payload: type_id:u32 + instance_id:u32)
  - i64/f64/bool 等の即値 → 対応する TLV プリミティブ

## 戻り値の規約（TLV）
- 戻り TLV の先頭タグにより分岐し、必要ならネイティブへデコード：
  - tag=3 → i64、tag=5 → f64（`*_i64`/`*_f64` 経由）
  - tag=6 → string、tag=7 → bytes
  - tag=8 → handle（type_id/instance_id を登録し BoxRef 構築）
- `returns_result` のメソッドは、VM で Ok/Err を Result に包む（AOT も同等方針）。

## N引数サポート（暫定）
- `*_invoke3_*` は a1/a2 までの即値/ハンドルを直積み、3引数目以降はレガシーVM引数（位置3..N）からTLVに詰めることで対応（開発向け）。
- ネイティブEXE（AOT）でのN引数の完全化は将来の `*_invokeN_*` で対応予定。

## Lowerer の前提（Handle-First）
- `emit_plugin_invoke(type_id, method_id, argc, has_ret)` を一本化。a0 は常に `nyash.handle.of(receiver)` を積む。
- 受け手箱名が未確定な場面は by-name シムを使用可（最適化は後段）。

## メモ
- TLV タグの例: 1=Bool, 3=I64, 5=F64, 6=String, 7=Bytes, 8=Handle。
- 既存詳細は `docs/papers/nyash-unified-lifecycle/technical-details.md` 参照。
