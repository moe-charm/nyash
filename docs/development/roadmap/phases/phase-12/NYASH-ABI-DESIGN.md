# Nyash ABI 概要（統合ABIダイジェスト）

本ドキュメントは `UNIFIED-ABI-DESIGN.md` の要約です。詳細は統合仕様を参照してください。

- 目的: C ABI を維持しつつ、NyashValue（3×u64相当）でのゼロコピー呼び出しを段階導入
- TypeBox: FQN/stable_id/vtable(C/Nyash) を束ねるディスクリプタ
- 所有権: BORROW/TRANSFER/CLONE を明示（release責務の所在を固定）
- 例外: C ABIはnothrow。越境例外は nyrt_err へ変換
- ディスパッチ: Nyash vtable優先→C vtable/TLVフォールバック（VM/JIT共通）
- 導入順序: TypeBoxレジストリ→統一ディスパッチ→Nyash ABI サンプル→最適化

このフェーズの実装タスクは [TASKS.md](./TASKS.md) を参照。

