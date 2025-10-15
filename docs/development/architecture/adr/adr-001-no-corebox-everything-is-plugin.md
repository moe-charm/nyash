# ADR-001: No CoreBox, Everything is Plugin — Provider/Type 分離方針

Status: Accepted (Phase 15.5 設計)  
Last updated: 2025-09-26

## 決定

- CoreBox（言語ランタイム内の特別な箱実装）は復活させない。
- 最小カーネル（NyKernel）は GC/Handle/TLV/Extern/PluginRegistry/ABI だけを提供し、箱の実装は一切持たない。
- 機能はすべて Plugin（TypeBox v2）で提供する（Everything is Plugin）。
- 型名（Stable Type Name: STN）と実装提供者（Provider ID: PVN）を分離する。
  - 例: STN = `StringBox`、PVN = `kernel:string@1.0` / `acme:string@2.1`。
  - コードは常に STN に依存し、どの PVN を使うかは `nyash.toml` で選択する。
- 起動シーケンスは「Kernel init → plugins.bootstrap（静的束）+ plugins.dynamic（任意）登録 → Verify（必須メソッド等）→ 実行」。
- 呼び出しは VM/LLVM 共通で `ny_new_box` / `ny_call_method` に一本化（MIR Callee 確定から統一）する。

## 背景 / 文脈

- using の SSOT（唯一の真実）を `nyash.toml` に置き、実体結合は AST マージに一本化した。これにより宣言≻式の曖昧性を排除し、依存は設定で一元管理できる。
- CoreBox を残すと特別経路や例外が増え、Everything is Plugin と相反する。Kernel/Plugin の責務分離が維持性と置換性を高める。

## 設計詳細

### NyKernel の責務
- GC / Roots / Safepoint / Write barrier
- Handle / TLV
- Extern registry（`env.console.*` 最小）
- Plugin registry（登録・検索・呼び出し）
- C ABI: `ny_new_box(type_id, args)`, `ny_call_method(recv, type_id, method_id, args, out)`, `ny_gc_*`

### Bootstrap Pack（静的リンクの基本プラグイン束）
- String/Integer/Array/Map/Console などの最小実用セット。
- 静的リンクして起動時に一括登録（特権経路は作らない）。
- 動的プラグインで override 可能（ポリシーで制御）。

### Provider/Type 分離（TOML スキーマ案・既定OFF）
```toml
[types.StringBox]
provider = "kernel:string@1.0"
allow_override = true

[providers."kernel:string@1.0"]
crate = "nyash-plugin-base-string"

[providers."acme:string@2.1"]
path = "./plugins/libacme_string.so"
override = true

[policy]
factory = "plugin-first"   # compat_plugin_first | static_only
```

### Verify（必須）
- plugin-tester で Lifecycle/TLV/必須メソッドを検証。欠落時は即エラー。

## 代替案の却下（kernelString 等）

- 型名に provider を織り込む（例: `kernelString`）案は、差し替え不能化・特権化・型ID安定性の破壊リスクが高く不採用。区別は provider 名で行い、型名（STN）は常に不変とする。

## ロードマップ（小さく段階的・既定OFF）
- K0: ADR/Docs 追加（本書）。
- K1: TOML スキーマの雛形（types/providers/policy）を docs と設定ローダに受け口だけ追加（挙動不変）。
- K2: 起動時に provider 解決の受け口をログ出力のみで導入（挙動不変）。
- K3: Verify フックを preflight_plugins に統合（既定OFF）。
- K4: Bootstrap Pack の登録導線（prod 限定フラグ・既定OFF）。

## 移行 / 互換
- 既存コードは型名（STN）のまま変更不要。provider 置換は TOML で完結。
- VM fallback の暫定個別救済はフラグ付き/短期で撤去。最終形は `ny_call_method` に集約。

