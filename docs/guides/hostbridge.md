# HostBridge — Unified Box Calls (Hako ABI)

Purpose
- Provide a single call surface for all Box interactions across lines (Rust VM, LLVM/AOT, Hakorune‑VM).
- Hide the difference between dynamic plugins (.so/.dll) and embedded (static) providers behind one face.

Status (Phase 15.7)
- Phase A: Hako‑side façade `HostBridgeBox`（最小）で FileBox 疎通（完了）。
- Phase B: Hako 側の `HostBridgeBox` を Extern 呼びに切替（landed）。
  - `.hako` から `Extern("hostbridge.box_new/box_call")` を呼び出し、Rust 側 HostBridge にフォワード。
  - Rust HostBridge は UnifiedRegistry へ委譲し、PluginLoaderV2（動的）/Provider（静的）を同一面で解決。
  - Hakorune‑VM(nyvm) も同一面に統一（nyvm 内の簡易実装は撤退し、常に Rust extern へ完全移譲）。
  - Runner は起動時に一度だけプラグインを早期ロード（idempotent）。`HAKO_PLUGIN_POLICY` に準拠。

API (minimal)
- `box_new(name: String, args: Array<TLV>) -> Handle|Null`
- `box_call(handle, method: String, args: Array<TLV>) -> TLV|Null`
- （必要時）`extern_call(name: String, args: Array<TLV>) -> TLV|Null`

Resolution Policy
- Env: `HAKO_PLUGIN_POLICY=off|auto|force`（`NYASH_PLUGIN_POLICY` 互換）
  - off: 静的（埋め込み）のみ
  - auto: プラグイン優先→無ければ静的
  - force: プラグインのみ（anchors 失敗は Fail‑Fast）
- ルーティング（Phase B）
  1) PluginLoaderV2 の Spec/Invoke があれば plugin invoker
  2) なければ Provider（embedded）

Early Load (Runner)
- Runner は `init_bid_plugins()` を最初に一度だけ呼び、`hako.toml`（または `nyash.toml`）から [libraries] を読み込み。
- ロード方針は `HAKO_PLUGIN_POLICY` に従う（auto=静穏フォールバック、force=Fail‑Fast）。
  - テスト用途で明示ロードしたい場合は `NYASH_PLUGIN_DIRECT_{LIB,PATH,BOXES}` を使用（短命・開発専用）。

Handles & TLV
- 値は TLV で統一（i64/f64/bool/string/PluginHandle/HostHandle/...）。
- (type_id, instance_id) → Arc のグローバルキャッシュで同一性を維持（Map.set→get でも同一参照）。

Static vs Dynamic
- 静的（embedded）: libnyrt / provider registry に組み込み（AOT/EXE の既定）。
- 動的（plugins/*.so）:
  - 起動時に hako.toml の [libraries] を読み、PluginLoaderV2 が dlopen + hako_box.toml ingest で Spec を構築。
  - `HAKO_PLUGIN_POLICY=auto|force` で有効化。

LLVM/AOT（EXE）での動的解決
- EXE 実行時にランタイムが dlopen する。配置:
  - EXE と同階層、または `./plugins` に `libnyash_*.so` と `hako.toml`。
  - 依存の探索は RPATH=$ORIGIN の利用を推奨。必要時に `LD_LIBRARY_PATH`（Windows: PATH）で補助。

Hakorune‑VM（nyvm）
- `.hako` の `HostBridgeBox` は Extern に統一済み（Phase B）。Rust HostBridge にフォワード。
- nyvm からの `hostbridge.*` は Hako extern を通じて Rust 側へ完全移譲（Hako 内の直実装は撤退）。
- Runner→nyvm は in‑memory 直渡し（ファイルI/Oなし）。
- 旧フェーズの薄い直実装は撤退（最小疎通は extern 経路でカバー）。

Smokes
- FileBox.open/read（plugins=auto）で hello‑bridge 出力 → PASS
- 代表 quick‑selfhost スモーク: `hostbridge_file_plugin_vm.sh`
  - nyvm/provider 検証は JSON プラグイン配置が必要（plugins/nyash-json-plugin/libnyash_json_plugin.so）。
  - `HAKO_MIRIO_PROVIDER=yyjson` で MirIoBox が provider 経路を使用（未配置時は scan にフォールバック）。

Fail‑Fast
- anchors miss（`policy=force`）：即エラー
- Spec/Invoke 不整合：即エラー（候補提示）
- TLV decode 失敗：型名と位置を含むエラー
