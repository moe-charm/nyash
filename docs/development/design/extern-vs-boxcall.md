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

