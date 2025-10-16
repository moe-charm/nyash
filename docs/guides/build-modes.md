# Build Modes — Plugin‑Only / Legacy‑Only / Mixed

目的
- `src/boxes/`（legacy）と `plugins/`（v2 TypeBox）の運用を「排他」にして、設計の見通しと再現性を高める。

結論（推奨）
- 既定は plugin‑only（legacy は OFF）。必要なときだけ明示で切り替える。
- ルータのフォールバックは禁止（Fail‑Fast）。plugins‑only で builtin に落とさない、legacy‑only で plugin に落とさない。

## モードとコマンド

- Plugin‑only（推奨・既定）
```
cargo build --release --no-default-features -F cli,plugins,host-anchors
```

- Legacy‑only（移行検証用）
```
cargo build --release --no-default-features -F cli,legacy-boxes,host-anchors
```

- Mixed（移行期のみ。フォールバック禁止のまま）
```
cargo build --release -F plugins,legacy-boxes
```

## スモーク/CI の指針
- プロファイルごとにモードを固定して走らせる（ENVでの切替は禁止）。
- 最低限、plugin‑only ラインを必須に（CI必須）。legacy‑only は任意（互換確認）。

## AI/開発者ガイド
- 新規コードから `crate::boxes::*` を参照しない。やむを得ない場合は `#[cfg(feature="legacy-boxes")]` を付ける。
- 機能追加は plugin と HostHandle/Extern の正道で実装。どうしても plugin で難しい場合のみ、短命の直接埋め込み（Doc/TTL 付き）。

## トラブルシュート
- plugin 無効で `keysS/valuesS` が無い → plugin‑only で実行するか、`keys/values` を使う（HostHandle/Extern の表経路）。
- plugin が Void を返す → Fail‑Fast で検出。HostSlot/ExternAdapter で正道に修正（フォールバック禁止）。

