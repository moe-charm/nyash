# ProviderBox — 統合プロバイダ境界

責務
- プラグイン/レジストリ/組み込み（最終手段）への委譲順序を一本化する。
- NewBox 経路の実体化先（PluginHost → v2 Registry → UnifiedRegistry → builtin fallback）を一箇所で制御する。
- 決定モード（deterministic）では IO/NET 能力を持つ Box を拒否する（Fail-Fast）。

順序（Box作成）
1. PluginHost.create_box（必要に応じて targeted reprobe/load）
2. v2 BoxFactoryRegistry.create_box（plugin-on かつ core/plugin-only の場合は抑止）
3. UnifiedRegistry.create_box（最終手段）
   - `HAKO_PLUGIN_ON_STRICT=1` 時は builtin fallback を禁止し、エラーで止める

再プローブ方針
- `ensure_loaded()` は boot 済みか確認し、必要なら最小限のライブラリ/Spec をプローブする。
- create_box 前に、その型を提供するライブラリ候補があれば `load_library_direct()` → `reprobe_providers_for()` を試みる。

注意
- ProviderBox は“薄い箱”として維持し、VM ハンドラからの分岐/条件はここに寄せる。
- 既定ではポリシー auto。プラグインが無ければ副作用は発生しない。
