ExternCall vs BoxCall: 分離設計の理由（要約）

- 目的: VM/JIT間で同一挙動を担保しつつ、最適化や診断を明確にするため、ExternCall と BoxCall を上位で分離、下位で合流する。

- 上位（MIR/意味論）
  - ExternCall: env.*（IO/タスク/デバッグ/チェックポイント等）を表現。EffectMaskで最適化境界を明示。
  - BoxCall: 型ディスパッチ（vtable→PIC→汎用）。副作用はBox内部に閉じやすい。

- 下位（VM/JIT実装/ABI）
  - 可能な限り共通のHostCall基盤へ合流（Cシンボル、HostHandle、TLV）。
  - VM: ExternCall→PluginHost（extern_call）→必要に応じて host_api へ。
  - JIT: 同じCシンボル群を直接リンクすることで一致挙動を確保。

- STRICT（厳格モード）
  - `NYASH_ABI_STRICT=1` または `NYASH_EXTERN_STRICT=1` で未登録/未対応を明確なエラーに。
  - vtable側は TypeRegistry に基づき未対応メソッドを検出。
  - ExternCall側は Host/Loader が未登録なら明確な診断を返す。

- 最低限ハードコード
  - print/console.log 等は ExternCall（env.console）側に限定して最小限のハードコード。
  - BoxCall 側へのハードコードは避ける（最適化経路やキャッシュと混ざるのを防止）。

この方針により、最適化・キャッシュ・診断の責務範囲が鮮明になり、VM/JIT一致検証も行いやすくなる。
Extern vs BoxCall — 分離方針とスロット/アリティ一覧（Phase 12）

目的
- VM/JIT を問わず、BoxCall（Box上のメソッド呼び）と ExternCall（env.*）を明確に分離。
- Extern は name→slot 解決により、診断品質と性能を安定化（オプション）。
- BoxCall は vtable→PIC→汎用 の順で正式ルートとし、STRICT時の診断を最終仕様化。

方針
- BoxCall: vtable（TypeRegistry のスロット）→ PIC（poly→mono）→ 汎用メソッド呼び。
  - STRICT: 未登録メソッドは型名・メソッド名・arity・known一覧を含めてエラー。
- ExternCall: `extern_registry` で iface/method/arity を登録、任意で slot 経由のハンドラに集約。
  - `NYASH_EXTERN_ROUTE_SLOTS=1` で name→slot 専用ハンドラへ（VM/JITの挙動安定）。

TypeRegistryの代表スロット
- InstanceBox: 1(getField), 2(setField), 3(has), 4(size)
- ArrayBox: 100(get), 101(set), 102(len)
- MapBox: 200(size), 201(len), 202(has), 203(get), 204(set)
- StringBox: 300(len)

Extern スロット（抜粋）
- env.console: 10（log, warn, error, info, debug, println）
- env.debug:   11（trace）
- env.runtime: 12（checkpoint）
- env.future:  20（new, birth）, 21（set）, 22（await）
- env.task:    30（cancelCurrent）, 31（currentToken）, 32（yieldNow）, 33（sleepMs）

環境変数
- `NYASH_ABI_VTABLE`: VMのvtable経路有効化
- `NYASH_ABI_STRICT`: STRICT診断を有効化
- `NYASH_EXTERN_ROUTE_SLOTS`: Externをslot経路に統一
- `NYASH_JIT_HOST_BRIDGE`: JITのhost-bridge（by-slot経路）を有効化
- `NYASH_VM_PIC_THRESHOLD`: PICモノ化しきい値（既定=8）

