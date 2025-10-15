# Plugins Architecture — Quick Guide

- Policy: `HAKO_PLUGIN_POLICY={off|auto|force}`
  - `off`: builtins only（プラグインは使わない）
  - `auto`: プラグインがあれば使う。無ければ内蔵にフォールバック
  - `force`: プラグイン必須。無ければ Fail‑Fast
- Init order（起動順）
  1) `provider_box::ensure_loaded()` が呼ばれる
  2) `plugin_boot_box::boot()`（config検出→load_libraries）
  3) `reprobe_providers_for([Array,Map,String,…])`（不足分を再解決）
  4) v2 Registry に provider を適用（`new BoxType()` が有効化）
- TypeBox v2 非対応のライブラリ
  - 既定では SKIP（互換モードが必要な場合のみ明示的に有効化）
  - ログに「TypeBox symbol not found」と表示されることがあります（正常）
- よくある失敗と解決
  - config 未検出 → `NYASH_PLUGIN_CONFIG` を明示、または `nyash.toml`/`hako.toml` を配置
  - 非対応プラグイン（TypeBoxなし）→ その箱はロードされません（互換モード時のみ受理）
  - policy=force で未配置 → 明示的に Fail（ロード対象・パスを確認）
- 起動ダイジェスト（1行）
  - 例: `[provider] policy=auto config=nyash.toml loaded={ArrayBox,MapBox}`
  - 既定ON・1プロセス1回のみ出力
