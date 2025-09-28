# origin — Known 化（起源付与）/ PHI 伝播（軽量）

目的（P0）
- 受け手（me/receiver）の「起源クラス」を最小限付与して Known 化する。
- PHI で型/起源が全入力で一致する場合に限り、dst にメタを伝播する。
- 仕様は不変（Fail‑Fast/フォールバック追加なし）。あくまで観測と最小補強のみ。

責務
- infer.rs: me の起源付与（current_static_box もしくは関数名の Box プレフィックスから推測）。
- phi.rs: PHI 伝播（全入力一致時に type/origin を dst へコピー）。

非責務（禁止）
- 観測（DebugHub emit）は observe 層に限定。ここから直接 emit しない。
- 複雑な流量解析/型推論/ポリシー決定は行わない（P5 で検討）。
- 命令生成（MirInstruction の追加）は原則禁止（メタデータのみ変更）。

簡易API（使用側から呼び出し）
- `annotate_me_origin(builder: &mut MirBuilder, me_id: ValueId)`
  - me の ValueId に対し、分かる範囲で `value_origin_newbox` と `value_types(Box)` を設定。
- `propagate_phi_meta(builder: &mut MirBuilder, dst: ValueId, inputs: &Vec<(BasicBlockId, ValueId)>)`
  - `inputs` の型/起源が全て一致する場合のみ、`dst` にコピー。

レイヤールール
- Allowed: `MirBuilder` / `MirType` / `ValueId` に対するメタ操作。
- Forbidden: observe / rewrite への依存、NYABI/VM 呼び出し、命令生成。

