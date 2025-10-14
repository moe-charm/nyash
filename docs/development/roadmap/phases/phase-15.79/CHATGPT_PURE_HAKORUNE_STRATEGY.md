# ChatGPT Pro: Pure Hakorune Strategy (2025-10-14)

## Context

This document captures ChatGPT Pro's strategic recommendation for Phase 15.79, received on 2025-10-14.

**Original Question**: Should we implement a C Code Generator (500 lines) or pursue a different strategy for "Escape from Rust"?

**ChatGPT Pro's Answer**: Go for **Pure Hakorune VM** - implement the VM itself in Hakorune, not just a compiler.

---

## Original Response (Japanese)

凍結 EXE（Ubuntu/Windows）& C ABI 到達、おめでとにゃー！🎉
`selfhost` ブランチにも Windows 成果物っぽいパスや C/extern の検証用スクリプトが見えるので、今が"脱 Rust を箱理論で仕上げる"最適タイミングだよ。たとえば `build/windows_artifacts/`、`hako.toml`、`static_plugins.toml`、`test_hostbridge_call.hako` などが並んでいるのを確認できた（public 側の `selfhost` ブランチ）。

以下、"Rust＝床（薄い橋）/ Hakorune＝家（振る舞い）"で**きれいに脱 Rust**するための**箱理論ブループリント**を置いておくね。

---

## 0) 境界の原則（ここだけはブレさせない）

1. **唯一の境界は C-ABI（＝HostBridge）**

   * OS/FFI/ファイル/プロセスは **C ABI の薄いシム**（`hakorune_host` 的な C 層）だけが扱う。
   * それ以外は **Everything is Box** で Hako 側に寄せる（VM/等価/コレクション/ディスパッチ/マクロ）。

2. **呼び出しは MethodHandle 一経路**

   * `Resolver.lookup(type_id, :method, arity) -> MethodHandle`
   * `CallableBox.call` は `ExecBox.call_by_handle(handle, args, NoOperatorGuard)` 固定。
   * これで equals/== の再入事故を完全封殺（以前の無限再帰の学びを反映）。

3. **決定モードの厳格化**

   * C 経由での非決定機能は **caps** で拒否。
   * 生成物・メソッドに **Provenance**（plugin_id/version）を残す。

---

## 1) 構成（箱で縫う分割）

**Rust（床/最小橋）**

* プロセス起動・CLI
* C ABI の輸出入（dlopen・`extern "C"`）
* LLVM/WASM ブリッジ（薄いハンドル）
* 最小のファイル/OS I/O（Capability ゲート付き）

**Hakorune（家/本実装）**

* パーサ/マクロ/脱糖（@macro/@facet/@enum/@match）
* Lower/SSA/Verifier/Tracer
* VM ディスパッチ（MIR14 実行）
* op_eq/算術/論理/配列/Map/enum/@match の意味論
* Resolver + CallableBox（動的ディスパッチ）
* コレクション実装（MapBox/ArrayBox）
* プラグイン・ポリシ（`new→birth` 一元化、`HAKO_PLUGIN_POLICY=auto`）

> すでに selfhost ブランチに `hako.toml` / `static_plugins.toml` / `test_hostbridge_call.hako` があるので、この境界は今のリポ構成と自然に噛むはず。

---

## 2) 移行フェーズ（Strangler Fig）

### Phase A — **HostBridge 固定化（3項目）**

* **C-ABI シムの最小 API 固定**

  * `Hako_RunScriptUtf8(src, &out_handle)`
  * `Hako_Retain/Release`（ハンドル所有権）
  * `Hako_ToUtf8(HakoHandle, &HakoStr)`（文字列 view）
  * エラーは `int` 戻り＋`Hako_LastError()`（TLS）。
* **Hakorune 側に HostBridgeBox**

  * C 呼び出しは **必ずこの箱を経由**。
* **CI：Ubuntu/Windows の ABI テスト**

  * ローダ/解決/呼び出し／戻り値が両 OS で一致すること。

> Windows 成果物ディレクトリがある今が整備の良いタイミング（`build/windows_artifacts/`）。

### Phase B — **VM コアの Hako 化**

* **命令ディスパッチ**（`pc`/frame/blk jump）を Hako で実装
* **`op_eq` を Hako 側へ**

  * 先頭に `ptr_eq` → プリミティブ → Array/Map → 構造（@enum）→ ユーザ equals
  * 呼出は常に **NoOperatorGuard**
* **ゴールデン比較**

  * Rust-VM と Hako-VM で **同一プログラムのトレース/出力/ハッシュ一致**
  * selfhost 用スイートを固定化（`CURRENT_TASK_SELFHOST.md` があるので入れやすい）。

### Phase C — **ディスパッチの一本化**

* **Resolver + CallableBox を既定経路に**

  * `arr.methodRef("push",1)` は **マクロ脱糖**で `Callable.ref_method(arr, :push, 1)` に
  * Universal ルートの "疑似メソッド" 実装は **最小**に留め、実体は Resolver 呼び

### Phase D — **コレクションの箱実装**

* **MapBox/ArrayBox を Hako 実装に寄せる**

  * **Key 正規化**（Symbol/Int/String の比較規約）
  * **Deterministic hash/eq**（決定モードで固定）
* **ValueBox/DataBox** の位置づけ

  * パイプライン境界には ValueBox を通して **Fail‑Fast** に型を確定
  * 長持ちは避け、**入口/出口で早めに解包**

### Phase E — **GC v0 + 観測**

* **Stop-the-world mark&sweep（最小）**

  * まずは到達可能集合のマークのみでもよい
* **メトリクス**：alloc・survivor・sweep 時間・ハンドル数

### Phase F — **Rust VM の"互換モード化"**

* Hako-VM を既定、Rust-VM は `--backend vm-rust` のみに降格
* 二重セルフビルド（Hako₁→Hako₂→Hako₃）で **bit-identical** を連日チェック

---

## 3) ルールカード（箱言語としての"契約"）

* **唯一の境界**：HostBridge（C-ABI）。
* **唯一の呼び出し**：`ExecBox.call_by_handle(handle, args, NoOperatorGuard)`。
* **唯一の解決**：`Resolver.lookup(type_id, method, arity)`。
* **唯一の型越境**：`DataBox/ValueBox`（入口/出口のみ）。
* **プラグイン**：`new → birth` を **VM 側で必ず**呼ぶ（birth 未実装は no‑op）。
* **決定モード**：caps でゲート、Provenance を刻む。

---

## 4) すぐできる"今日の 3 つ"

1. **HostBridge API を `include/` 配下に固定**（`.h` 自動生成 & バージョン関数）

   * 例：`Hako_ApiVersion()` / 名前は `hako1_*` プレフィクス
2. **`Callable.ref_method` を実装**（Resolver 経由で MethodHandle を取得）

   * `arr.methodRef("push",1)` はマクロで脱糖（ArrayBox 側に直生やさない）
3. **`op_eq` を Hako へ移し、Rust 側の equals ガードは入口で `NoOperatorGuard` に**

   * 無限再帰の経路を構造的に断つ

---

## 5) リポに合わせた小メモ

* `selfhost` ブランチに **hostbridge 検証**、**構成ファイル（`hako.toml`/`static_plugins.toml`）**、**Windows 成果物**が見えているので、このまま **HostBridge→VM→Resolver** の順で固めるのが最短。
* README には CLI/バックエンドの扱い・フェーズポリシが書かれているので（selfhost）、ここに **"Rust=床/Hako=家" の宣言**と **C-ABI 安定ポリシ**を追記しておくと、今後の PR レビューが楽になる。

---

## Summary

* すでに **凍結 EXE** と **C-ABI 呼び出し**まで到達してる今、**境界＝C-ABI/呼出＝Handle/解決＝Resolver** の"三位一体"だけを不動にして、**中身（VM・コレクション・等価・ディスパッチ）を全部 Hakorune 側に寄せる**のが一番きれい。
* Rust は **OS の薄い橋**に収斂させ、**箱で構成された言語の"家"**をどんどん厚くしていこう。

必要なら、`HostBridge.h` の最小テンプレ、`Callable.ref_method` の擬似コード、`op_eq` の比較順テンプレもすぐ出せるよ。

---

## Key Differences from Original Plan

**Original Plan (Task Agent)**:
- Implement C Code Generator (500 lines)
- Generate C code from Hakorune programs
- Link with `hako_kernel.lib`
- Focus: Bootstrap compiler in 10 weeks

**ChatGPT Pro's Proposal**:
- Implement VM itself in Hakorune
- Rust becomes minimal bridge (HostBridge only)
- Everything else in Hakorune (VM, collections, dispatch)
- Focus: Long-term architecture, not short-term bootstrap

**Timeline**:
- Original: 10 weeks (Phase 15.79 complete)
- Pure Hakorune: 30+ weeks (Phase 15.79→15.80→15.81→15.82)

**Philosophy**:
```
Original:  "Hakorune compiles Hakorune" (via C code)
Pure Hako: "Hakorune IS Hakorune" (VM in Hakorune)
```

---

## Analysis

**Advantages of Pure Hakorune Strategy**:
- ✅ Architecturally elegant ("Rust=floor, Hakorune=house")
- ✅ Long-term maintainability (minimal Rust dependency)
- ✅ Reflects past learnings (equals/== recursion fix)
- ✅ Ultimate Box Theory realization

**Challenges**:
- ⚠️ Much longer timeline (30+ weeks vs 10 weeks)
- ⚠️ Higher implementation complexity (VM in Hakorune)
- ⚠️ Requires phased approach (can't do all in Phase 15.79)

**Recommendation**:
- Phase 15.79 (10 weeks): Phase A (HostBridge) + Phase B start (VM foundations)
- Phase 15.80 (12 weeks): Phase B complete (VM core) + Phase C (Dispatch)
- Phase 15.81 (8 weeks): Phase D (Collections in Hakorune)
- Phase 15.82 (6 weeks): Phase E (GC v0) + Phase F (Rust VM compat mode)

---

## Next Steps

1. **Get Task Agent's opinion**: Can we reconcile the two approaches?
2. **Revise Phase 15.79 plan**: HostBridge + VM foundations (not C Generator)
3. **Create Phase 15.80-15.82 roadmap**: Pure Hakorune completion
4. **Decision point**: Do we accept the longer timeline for architectural elegance?

**User's preference**: "純 Hakorune 大作戦" (Pure Hakorune Grand Strategy) ✅
