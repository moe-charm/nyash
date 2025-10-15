# Phase 15.7: セルフホスティング実現への道筋 - Hakoruneコンパイラ完成計画

Branch Note (selfhost)
- このブランチでは CLI バイナリ名は `hako` だよ。本文中の `hakorune`/`nyash` は `hako` に読み替えて実行してね。
- 環境変数は `HAKO_*`/`HAKU_*`/`HRN_*` は `NYASH_*` と相互エイリアス（自動マップ）なので、そのまま使ってOK。

## 🎯 **Phase 15.7の真の目的**

**「Hakorune で Hakorune をコンパイルする」完全なセルフホスティングの実現**

### 🎯 First Goal（Milestone M1 — Bootstrap）
- 目的: 「Hakorune コンパイラー（apps/selfhost-compiler/compiler.hako）を Hakorune でビルドし、LLVMで実行ファイル化」
- 成果物: 自己ホスト版コンパイラー実行ファイル（例: /tmp/hako_selfhost_compiler）
- 受け入れ基準（Acceptances）
  - A1: `tools/build_llvm.sh apps/selfhost-compiler/compiler.hako -o /tmp/hako_selfhost_compiler` が 0 終了
  - A2: `/tmp/hako_selfhost_compiler -- --min-json` の標準出力先頭行が非空 JSON ヘッダ（`{"version": …, "kind": …}` を含む）
  - A3（任意）: 生成 EXE で小サンプルを解析→Rust VM での実行結果と等価（quick スモークで 1 本確認）
- スモーク: `tools/smokes/v2/profiles/quick/selfhost/selfhost_bootstrap_llvm.sh`（既定 SKIP、`SMOKES_ENABLE_SELFHOST_BOOT=1` で有効）

### 🎯 Milestone M2 — Self‑Rebuild
- 目的: 生成した自己ホストコンパイラ EXE で compiler.hako を解析し、Stage‑1 JSON を出力できること
- 受け入れ: EXE の標準出力1行目が `"kind":"Program"` を含む
- スモーク: `tools/smokes/v2/profiles/quick/selfhost/selfhost_rebuild_vm.sh`（LLVM未導入時は自動SKIP）

### 🎯 Milestone M3 — E2E Parity（VM ↔ LLVM, selfhost compiler）
- 目的: ランナーから子プロセスに selfhost compiler を使い、VM/LLVM の結果が一致する
- 受け入れ: 代表サンプル（const_ret 等）で `Result:` 行が一致
- スモーク: `tools/smokes/v2/profiles/quick/selfhost/selfhost_e2e_vm_llvm.sh`（LLVM未導入時はVMのみ通過）

### 🧱 Baseline → Selfhost → Legacy Removal（段階）
- Baseline（本ドキュメントの前段）
  - デフォルト: 最小 Kernel 埋め込み、Plugins 既定OFF（オプトイン）
  - 解決順: User > Plugin > Kernel
  - ドキュメント: `docs/guides/kernel-plugin-baseline.md`, `docs/guides/build-runtime-defaults.md`
- Selfhost（M1 以降）
  - M2（計画）: 生成 EXE で compiler.hako を再ビルド → JSON ヘッダ/簡易 MIR を比較
  - M3（計画）: VM/LLVM の E2E 差分ゼロ（代表アプリ 1 本）
- Legacy Removal（M*）
  - VM 便宜 boxes_* の段階撤退（Plugin 実装へ移管）
  - 旧 CoreBox の Kernel 残骸を削減（Null/Missing 以外の stateful 実装を撤去）
  - 文字列エラー依存（"Key not found:") の完全廃止（null チェック統一）


## 🛣️ 実行ルート（道）— 小さく進めて確実に緑にする

この順で小粒に進めると、常に quick 緑を維持しながらセルフホスティングへ到達できるよ。

1) パーサ仕上げ（prelude安全・静的化）
- ObjectParseBox を prelude 安全に（_dq/_bs ヘルパで直文字回避）
- expr/stmt 側も utils を静的呼び出しに統一（new排除）
- 受け入れ: json_native 簡易 roundtrip 緑

2) Stage‑1 JSON の早期経路固定（quiet/--min-json）
- quiet/--min-json では AST プレリュードマージをスキップ容認（エラーにしない）
- main(args) に CLI script args を渡す（-- の後を JSON 経由で注入）
- 受け入れ: selfhost_min_json_header_vm PASS（緑）

3) MIR ローアの段階実装（最小→分岐/PHI）
- Return/Const → BinOp → Compare → Branch/Jump → PHI の順に有効化
- JSON v0 Bridge の到達不能 pred 除外を統一（return/throw）
- 受け入れ: selfhost_mir_m2_*（eq_true/eq_false/compare）を順に UN‑SKIP→緑

4) Hakorune VM 箱化ラインの安定化
- InstructionScannerBox/OpHandlersBox に一本化（無限ループ対策・観測の集約）
- Arithmetic/Compare の委譲（小さな共通箱へ移譲）
- 受け入れ: m2/m3 代表（branch/jump/phi）緑

5) using/[modules] のE2E（AST OFFでも登録維持）
- modules 登録は quiet でも常時維持（AST マージはスキップ可）
- 受け入れ: using_modules_alias_vm / using_modules_rune_host_vm PASS

6) LLVM 連結ラインの最小安定化（AOT/ハーネス）
- hako_kernel に最小シンボルを追加（string.concat_{ss,si,is} / console.* / readline）
- 最小セマンティクスはフラグ NYASH_HAKO_MIN_SEM=1 でON（既定OFF）
- 受け入れ: 代表ベンチのリンク成功・終了コード一致（ハーネス or AOT）

7) スモーク整備（quick → integration）
- selfhost_min_json_header_vm / selfhost_mir_m2_* / pipeline_v2_* / json_native roundtrip を quick に集約
- 受け入れ: quick 全緑、integration 代表緑

8) 性能・回収（ホット箇所のみ）
- Hakorune VM の compare/binop ホットパス最小化（ログ削減・必要なら inlining）
- LLVM の concat/console 経路の cpool 最適化
- 受け入れ: ベンチ中央値で説明可能な改善（仕様不変）

### よく使うコマンド（抜粋）
- ビルド: `cargo build --release`
- quiet ヘッダ確認: `NYASH_USING=1 NYASH_USING_AST=0 NYASH_JSON_ONLY=1 ./target/release/hakorune --backend vm apps/selfhost-compiler/compiler.hako -- --min-json`
- LLVM ハーネス: `NYASH_LLVM_USE_HARNESS=1 NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc NYASH_EMIT_EXE_NYRT=target/release ./target/release/hakorune --backend llvm apps/APP/main.hako`
- AOT最小セマンティクス: `NYASH_HAKO_MIN_SEM=1 NYASH_NYRT_SILENT_RESULT=1 ./app`
### 📊 **現状分析（2025-09-30）**

## 🆕 Updates — 2025-10-08（小粒の安定化）

- Namespace CLI の可視性を強化
  - `--list-modules` / `--modules-show` / `--modules-resolve` で先頭に `[policy] {module-first|path-first}` を表示。
  - Module‑First 運用時の意図が出力から一目で分かるようにした（dev UX）。

- Strict 診断テンプレ（using/modules）をガイド化
  - `NYASH_USING_CHECKS_STRICT=1` 時に 1 行で安定出力: 
    - MissingDep: `workspace missing dependency: <module> → <dep> (<req>)`
    - Conflict: `workspace namespace conflict: <ns> has multiple paths: <p1,p2,...>`
  - 併せて JSON 診断（`{"kind":"modules_error",...}`）も出すが、テストは 1 行文字列で安定比較可能。

- Throw/PHI の Builder 側の扱い修正（JSON v0 経路）
  - Match の then‑arm が Throw で終わる場合、PHI 入力と merge ジャンプを抑止（到達不能の入力を除外）。
  - 受け入れ: `json_v0_match_throw_phi_vm.sh` を常時ON化し、Result: 7 を確認（else のみが PHI に来る）。

- Stage1JsonScannerBox の適用拡大（取りこぼし減）
  - 早期経路（Call/Method）で `extract_name_args` を使う軽量フォールバックを追加。微妙な JSON 差異にも耐性。

- Macro 子プロセスの隔離（テスト/開発）
  - 子に `NYASH_SKIP_TOML_ENV=1` / `NYASH_USING=0` を注入してプロジェクト TOML/using の影響を遮断。ガイドへ追記。


#### ✅ **既に実装済み（堅固な基盤）**
- **Rustコンパイラ**: 完全実装・安定動作（Phase 1-14）
  - Parser（完全実装✅）
  - AST生成（完全✅）
  - MIRビルダー（完全✅）
  - 3バックエンド実装✅
    - Rust VM（712行、開発・デバッグ用）
    - Python LLVM/llvmlite（1,456行、本番・最適化用）
    - PyVM（1,074行、JSON v0ブリッジ・using処理専用）
  - プラグインシステム（完全✅）

#### 🔄 **実装中（Hakoruneコンパイラ）**
- **場所**: `apps/selfhost-compiler/`
- **現状**:
  - パーサーMVP ✅（Stage-2/3サポート）
  - MIR生成基本 ✅（const/binop/compare/branch/jump/ret）
  - JSON v0出力 ✅（最小動作確認済み）
  - **Pipeline V2 🔄（Box-First emit-only architecture）**
    - 📋 **[詳細設計](../../selfhosting/pipeline_v2.md)** - 全体像・Boxes・制約
    - 📦 **[実装](../../../../selfhost/compiler/pipeline_v2/)** - ExecutionPipelineBox/BackendBox/MirBuilderBox
    - 🔧 **[契約](../../../../apps/selfhost-compiler/INTERFACES.md)** - インターフェース仕様
    - 🧪 **[スモーク](../../selfhosting/pipeline_v2.md#smokes-quick)** - 受け入れテスト

#### ❌ **未完成（Phase 15.7の目標）**
1. **branch/jump最小生成** ✅（完了）
2. **LocalSSA.ensure_cond** ✅（最終パスに統合）
3. **全構文サポート** 📝（match式、property、lambda等）
4. **最適化パス** 📝（デッドコード削除、インライン化等）
5. **完全なブートストラップ** 🎯（c0→c1→c1'）

### 🤔 **VM層も一緒に作った方が楽？** → **YES！絶対YES！**

#### 💡 **理由1: 相互検証が可能**
```
Hakoruneコンパイラ（apps/selfhost-compiler/compiler.hako｜互換: .nyash）
    ↓ MIR生成
Hakorune-VM（apps/hakorune/vm/boxes/hakorune_vm_min.hako｜互換: selfhost/vm/boxes/mir_vm_min.hako）
    ↓ 実行
Rust VM（src/backend/mir_interpreter/）
    ↓ 比較検証
差分があれば即座に発見！
```

#### 💡 **理由2: デバッグが容易**
- **Hakoruneコンパイラのバグ**: Hakorune VMで実行 → エラー出る → MIRを見る → Rust VMと比較
- **Hakorune VMのバグ**: Rustコンパイラ生成MIRで実行 → Rust VMと比較 → 差分発見

#### 💡 **理由3: 完全な理解（教材として最高）**
```
Hakoruneでコンパイラ書く
    +
Hakoruneで実行器書く
    =
完全な理解（世界一美しい自己参照システム）
```

### 🎯 **Phase 15.7の正しい優先順位**

#### **P0: Rust VM層の安定化（既存バグの点修正・回帰防止）**
- 受け手推定・RouterPolicy・LocalSSA/材化・VarMapGuard 等の補強を優先
- quick/integration 常緑維持（既定の品質基準）
- **理由**: Rust VMは比較検証の基準点として絶対的に安定している必要がある

#### **P1: Hakorune‑VM 仕上げ（完了✅）**
- M2/M3 の代表＋エッジスモークを quick に追加
- 単一パス＋厳密セグメントで緑維持
- **成果**: `apps/hakorune/vm/boxes/hakorune_vm_min.hako` 安定動作（旧パスとの互換ルート維持）

【2025-10-01 追記】
- Hakorune VM に call/boxcall/newbox の最小意味論（i64 引数の総和）を追加。代表スモーク（exec）を quick に追加:
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_call_exec_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_method_exec_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_newbox_exec_vm.sh`
- Stage‑1 抽出器を負数/空白に寛容化。Emit 側は配列/文字列の両方から引数を正規化材化。
- Pipeline V2 に `LocalSSA.ensure_calls(...)` を導入（call/method/new の材化ポリシー集約）。
- v1 `mir_call` の shape スモーク（VM-only）を追加:
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_call_v1_shape_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_method_v1_shape_vm.sh`
  - `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_newbox_v1_shape_vm.sh`
- LLVM ハーネス compile-only の PHI 形状スモークを追加（STRICT=1）:
  - `tools/smokes/v2/profiles/quick/llvm/phi_if_merge_compile_ok.sh`
  - `tools/smokes/v2/profiles/quick/llvm/phi_loop_compile_ok.sh`
- 返り値→終了コードの統一（VM/WASM/AOT）: Rust VM はプログラムの戻り値をプロセス終了コードへ反映（0..255）。

【2025-10-02 追記】
- FlowEntryBox / FlowRunner（箱化・薄導線）
  - 追加: `selfhost/compiler/pipeline_v2/flow_entry.hako`（emit-only 入口）
  - 追加: `apps/selfhost/vm/flow_runner.hako`（Hakorune VM 実行薄箱）
  - 役割分離: emit は selfhost-compiler 配下、実行は selfhost/vm 配下（箱境界）
- LocalSSA 材化ポリシーの統一
  - `ensure_calls`（call/method/new）、`ensure_cond`（branch cond）ともに「PHI直後に copy 挿入」に統一
  - JSONテキスト整形で挙動不変・Fail‑Fast（未対応形は無変更）
- MirCall v1（統一呼出し）
  - 薄箱: `selfhost/compiler/pipeline_v2/mir_call_box.hako` を追加（emit-only）
  - ハーネス時の v1→v0 ダウングレード (`NYASH_LLVM_DOWNGRADE_V1=1`) を前提に shape/compile を安定化
  - 未解決 Global は v0 extern へ降格（compile-only）、VM/AOT は未解決エラー（Fail‑Fast）
- CLI: `--emit-mir-json` をグローバル早期ゲートに（バックエンド非依存）
  - どの backend 指定でも、パース→MIR→JSON 書き出し→即終了
  - ベンチ／WASM パイプラインの自動化に利用
- 工具: WASM 一括スクリプトを追加
  - `tools/build_and_run_wasm.sh`（.nyash → MIR(JSON) → WASM → 実行/exit code）
  - 依存: python3+llvmlite, node（wasm_runner.js）
- LLVM ハーネス（PHI）
  - Φ生成=PhiHandler、配線=finalize の不変を明記
  - 関数境界で `phi_wired`/`block_phi_incomings` をクリア（リーク防止）
  - ハーネス compile 前に IR をサニタイズ: 空PHI除去＋ブロック先頭へのPHIグループ化（検証を安定化）

【2025-10-05 追記 — 小粒前進のまとめ】


【2025-10-11 追記 — M2/M3達成と盤石化】
- Selfhost M2/M3 達成（quick常時ONの小セットで緑）
  - M2: selfhost_rebuild_vm（EXEで自分自身を解析、ProgramヘッダOK）
  - M3: parity_q_*（VM↔LLVM最小パリティ）＋ plugin identity 緑
  - 追加パリティ: JSON stringify / <=, >= 境界
- Provider 起動ダイジェストを固定化（1行）
  - 形式: policy=… config=… loaded={…} anchors=ok|miss stage2=on|off
  - dlsym セルフチェック導入（nyash_array_new_h / nyrt_host_call_slot / nyash_host_from_plugin_handle）
  - policy=force では anchors miss 時に Fail‑Fast
- plugins profile の最小LLVM交差を追加（軽量・常時ON）
  - plugin_parity_min_vm_llvm（VM/LLVM最小一致の検査）
- WASM Phase‑A を開始（ベンチ準備の最小実装）
  - ArrayBox: len/get/set/push/clear をコード生成（固定cap=8、OOB=0）
  - WAT生成スモークを追加（auto‑SKIPつき）:
    - wasm_compile_array_ops_wat（最小）

【2025-10-12 追記 — SSOT/診断/スモーク更新】
- SSOT 優先の Type 解決に切替（既定ON）
  - TypeBox の slot/arity/aliases 解決と Core の type_id を SSOT → config(hako/nyash.toml) → 既定 の順に統一。
  - 一時無効化ゲート: `HAKO_REGISTRY_SSOT_DISABLE=1`（`NYASH_*` 互換）。
  - 参照: `src/runtime/type_registry.rs:66`, `src/runtime/type_registry.rs:312`。
- 診断の一元化（arity/unknown‑slot 等）
  - ルーター側の直書き文言を `diagnostics::msg` に統合し、Fail‑Fast メッセージの安定性を向上。
  - 参照: `src/runtime/method_router_box/mod.rs:16`, `map_callable.rs:44`, `method_ref.rs:29`。
- スモーク運用の指針（軽量・常緑）
  - M2 quick 代表を常時（rebuild は環境により自動SKIP）。
    - 実行: `tools/smokes/v2/run.sh --profile quick --filter 'selfhost_*_vm'`
  - M3 integration‑core の小セット（5〜8本）を代表で常時、フル20本は任意。
    - 実行: `tools/smokes/v2/run.sh --profile integration-core`
  - SSOT バリデータ（編集時に任意で実行）
    - `./tools/check_ssot_table.sh`（name→slot 多重割当/重複の検出）

## 🧭 MIR 生成ロードマップ（Phase 15.7）

目的
- Rust 側の MIR 生成（Builder/Emitter）を“箱言語側（selfhost compiler）”へ段階移行し、Rust 側は JSON 受け口に縮退。
- 既定の意味論は不変（Fail‑Fast 強化は可）。VM/LLVM のパリティを維持したまま、自己ホスト化の歩幅を刻む。

境界と原則（Box‑First）
- 生成面（Builder/Emitter）: selfhost/compiler/pipeline_v2/ 配下（EmitMirFlow/Map 等）。
- 実行面（VM/LLVM）: 既存の Rust VM と llvmlite ハーネス。
- 意味論の確定点:
  - Eq/Ne は Extern("nyrt.ops.op_eq") に統一（Builder 正規化）。
  - メソッド呼出は SSOT（specs/type_registry.toml）の slot/arity に依存。
- 失敗ポリシー: Fail‑Fast（静かなフォールバックは禁止）。

段階計画（P1→P5）
- P1（最小ブート）
  - 対象: const/ret、compare→branch/jump→ret。
  - 実装: EmitMirFlow(Map) を活用して 1関数1エントリ CFG を生成。Phi は不要（簡易ダイアモンド）。
  - 受け入れ:
    - quick: selfhost_emit_mir_min_rc_vm（rc‑only）緑。
    - integration‑core: 既存 parity セット（const_ret/compare/branch）緑。
- P2（式）
  - 対象: binop（+,-,*,/,%）と単項（neg,not,bitnot）最小。
  - 受け入れ: integration‑core の算術代表がVM/LLVM一致。
- P3（呼出）
  - 対象: Call/Method/Extern の最小統一（Extern は registry/generated を参照）。
  - 受け入れ: op_eq primitives/reflexive の代表が Builder→VM/LLVM で一致。
- P4（NewBox: Core Collections）
  - 対象: ArrayBox/StringBox/MapBox の最小 new（birth は Runner/VM 規約に従う）。
  - 受け入れ: quick のコレクション最小ケース rc‑only 緑。
- P5（整形/材化）
  - 対象: LocalSSA ensure_cond/ensure_calls の最小適用（未定義形は Fail‑Fast）。
  - 受け入れ: 既存の LocalSSA スモーク rc‑only 緑。

ゲート/フラグ（既存の活用）
- NY 系（エイリアス受理: HAKO_*）
  - NYASH_USE_NY_COMPILER=1（自己ホスト子を使用）
  - NYASH_NY_COMPILER_MIN_JSON=1（最小 JSON）
  - NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir"（MIR(JSON v0) 出力）
  - NYASH_JSON_ONLY=1（quiet 受理）
  - 既定はOFF。quick の代表は rc‑only で常時OK。

SSOT 連動
- slots/arity/aliases は specs/type_registry.toml を SSOT とし、Builder 参照は SSOT に統一（静的表は互換 fallback）。
- type_id も SSOT→config→既定の優先で参照（既に Runtime 実装済）。

テスト/スモーク方針
- quick: rc‑only を基本（本文依存を避ける）。
- integration‑core: VM/LLVM パリティは既存代表を利用（5〜8本）。段階追加は最小限。
- plugin‑on: 代表1本のみ常時（rc‑only）。広いE2Eは opt‑in。

ロールバック/安全策
- 生成経路の切替は環境フラグでいつでもOFFに戻せる構成を維持。
- Fail‑Fast は既定ON。問題時は quick を崩さずに段階差し戻しが可能。

完了定義（Phase 15.7 時点）
- P1〜P3 が緑、P4 は最小 new が通る、P5 は ensure_cond の最小が効く。
- quick 全緑（rc‑only代表を含む）、integration‑core 全緑。

### 進捗（2025‑10‑12 現在）

- P1 完了（const/ret, compare diamond）
  - 共有箱を導入: `selfhost/shared/common/mir/{mir_schema_box.hako,block_builder_box.hako}`
  - emit 経路を薄アダプタ化（出力互換）:
    - `selfhost/compiler/pipeline_v2/emit_mir_flow_map.hako`
    - `selfhost/compiler/pipeline_v2/emit_mir_flow.hako`
  - quick 代表（rc‑only）: `selfhost_emit_mir_min_rc_vm` が常時緑

- P2 準備完了（binop/loop 最小）
  - BlockBuilder に `binop/loop_counter` を追加し、emit 経路から利用開始
  - quick 代表（rc‑only）: `selfhost_emit_mir_binop_min_rc_vm` を追加

- P3 最小導線（Extern op_eq）
  - CallEmit/MirJsonBuilderMin を拡張し、`mir_call callee=Extern("nyrt.ops.op_eq")` を生成
  - emit 経路に `emit_op_eq(lhs,rhs)` を追加（出力互換）
  - quick 代表（rc‑only）: `selfhost_pipeline_v2_op_eq_vm`（true）と `selfhost_pipeline_v2_op_eq_false_vm`（false）を追加

- P3 追加（Global/Method/Constructor E2E）
  - v1→v0 変換（`MirJsonV1Adapter.to_v0`）→ `--json-file` 実行のE2E代表を追加（rc‑only）
    - Global(print): `selfhost_mircall_global_print_e2e_vm`（副作用は無視。ret 0でrcのみ検証）
    - Constructor+Method(size): `selfhost_mircall_ctor_method_e2e_vm`（ArrayBox.size）
    - Global(JSON.stringify): `selfhost_mircall_global_json_stringify_e2e_vm`（`NYASH_JSON_STRINGIFY_DEV=1` で純関数化。ret 0でrcのみ）

### ENV ゲート（Builder Eq/Ne 正規化）
- `NYASH_BUILDER_EQ_TO_OPEQ=1|0`（別名 `HAKO_BUILDER_EQ_TO_OPEQ`）
  - 1/true/on: Eq/Ne を Extern("nyrt.ops.op_eq") に降格（NeはNotで反転）
  - 0/未設定: 降格しない（Compareのまま）
  - 既定: OFF（Phase 15.7時点）。回帰が安定したらONに昇格可能。
  - quick 代表（rc-only）: `tools/smokes/v2/profiles/quick/core/builder_eq_gate_off_rc_vm.sh`

備考
- 出力 JSON 形状は既存の `{ functions:[...] }` に揃えており、Rust 側受け口の互換性を維持。
- SSOT は既定ON。slots/arity/aliases は specs/type_registry.toml を優先参照。Extern シグネチャは specs/externs/registry.toml 由来の生成物を利用。

    - wasm_compile_bench_suite_wat（apps/benchmarks/wasm/basic/*.hako 一括）
- ENV/プロファイルの整理（迷いの削減）
  - HAKO_PLUGIN_POLICY=auto を主。NYASH_* は互換
  - Stage‑2（HostHandle Array）は profile 限定でON

受け入れ（更新）
- quick: parity_* + M2代表 + plugin identity（1本）+ wasm WAT（1本）緑
- plugins: identity（2本）+ 最小LLVM交差（1本）緑
- provider digest が毎回表示、policy=forceは anchors=ok 必須
- Hakorune VM の箱化・安定化（自己ホスト向け）
  - HakoruneVmMin: InstructionScannerBox.next + OpHandlersBox.handle_* 経由に統一。無限ループ対策/観測の集約を反映。
  - 共通箱の拡充: ProgramStateBox（bb/prev/steps）, CfgNavigatorBox（index_of_from/head/tail）, RetResolverBox（ret一元化）, DiagnosticsBox（DEVでdebug）。
  - using/[modules]: alias（hakorune.vm.mir_min）を追加し、flow_runner などの呼び先を段階的に新別名へ統一。
- JSON v0 Bridge の堅牢化
  - 到達不能 pred（return/throw）を PHI incoming から除外する判定を unify（if/match両方）。
- 文字列/数値ヘルパの重複解消
  - StringHelpers（to_i64/int_to_str/json_quote/read_digits）へ委譲。JsonFrag/JsonScan/Compiler 側の重複を削減。
  - selfhost VM 補助の .nyash→.hako 統一（一部）。
- WASM ABI スケルトン（handoff 用）
  - docs/guides/wasm-abi.md に最小ABI（nykernel.malloc/load_i64/store_i64）と契約を記載。
  - crates/nykernel-wasm（wasm32向けbump allocator + load/store）を追加（ワークスペース未接続）。
  - hakorune-std/core/array.hako を追加（extern_call一本化）。VMでは NYASH_ENABLE_NYKERNEL_STUB=1 で開発スタブ稼働。
- スモーク整備
  - noisy系は opt-in 化（ゲート環境変数で明示ON）。
  - ArrayBox の最小E2E（push/get/resize）を quick に opt-in で追加（VMスタブ）。

次の小粒（Phase 15.7 継続）
1) Stage‑1/2 最小 E2E（Ny→JSON→MIR(JSON)→Hakorune VM）を代表3本で緑固定（const→ret／compare→ret／compare→branch→phi）。
2) UsingResolverBox/NamespaceBox を実装し、Pipeline V2 に統合（Callee::ModuleFunction 正規化を前段で完了）。
3) Hakorune VM：ProgramStateBox/CfgNavigatorBox の参照を全面 get 化（残差つぶし）＋ 代表CFG（diamond/jump_chain）を1本ずつ追加。
4) （先送り）index_of_from の集約（JsonFrag/flow_runner/flow_debugger など）を CfgNavigatorBox へ段階移行（Phase 15.12）。


## 🔁 Rust → Nyash 移植計画（Phase 15.7 拡張）

目的
- Hakorune Compiler/VM の“自己完結”を高め、Rust 実装への依存を段階的に縮小。
- Box‑First 原則に従い、移植点を小箱の境界に分解して安全に前進。

優先度P1（1–2週）— コンパイラ側（apps/selfhost-compiler）
- Using/Namespace 解決（未完の中核）
  - 新規: `UsingResolverBox`（modules/symbols の登録・照会）
  - 新規: `NamespaceBox`（"A.B" → Callee::ModuleFunction への正規化）
  - 統合: Pipeline V2 の compare/call/method 前段で using 解決を完了
- Emit/Builder の整流
  - `emit_*_box.hako` 群を Map→to_json に統一（HeaderEmitBox 経由）
  - `CallEmitBox/NewBoxEmitBox` の API 統一（args_array/text の併設）
- 署名検証（コンパイル時Fail‑Fastの拡張）
  - `SignatureVerifierBox` を全発行点に適用
  - `MethodRegistryBox` のカバレッジ拡張（toJSON/length 別名整理, startsWith/endsWith 等）

優先度P2（1週）— VM 側（apps/selfhost/vm, apps/hakorune/vm）
- Hakorune VM の箱化仕上げ
  - `InstructionScannerBox`/`OpHandlersBox` 採用の残差つぶし
  - `ProgramStateBox`（bb/prev/steps）利用の全面化
  - `CfgNavigatorBox.index_of_from` への検索統一（残差）
- PHI/Throw 周辺の最小意味論
  - 非到達 Throw の PHI 除外（Bridge/Builder は済）
  - Throw は VM での最小 return 化（将来の例外伝播は別フェーズ）

優先度P3（1週）— 共有ユーティリティ
- `JsonCursorBox` の採用拡大（minivm_probe/step_runner 等の直接スキャン撤去）
- `StringStd.index_of_from` の横展開（ツール系・旧コードの2引数 indexOf 残差つぶし）
- DEV リントを段階的に厳格化（`LINT_INDEXOF_FAIL=1`）

マイルストーン（WBS 概算）
- Week 1: UsingResolver/Namespace 実装→Pipeline V2 統合, Call/Method/New emit の to_json 仕上げ
- Week 2: SignatureVerifier/MethodRegistry 拡張, Hakorune VM 箱化仕上げ（state/scanner/handlers）
- Week 3: JsonCursor 置換の残差、2引数 indexOf の完全撤去、Throw/PHI スモーク常時ON化の可否判断

受け入れ（本セクション）
- quick: 自己ホスト代表3本（const→ret / compare→ret / compare→branch→phi）常時緑
- Throwスモーク: if 非到達側に Throw を含むケースを常時ONで緑（実行側 Throw は別テストでゲート）
- lint-ny: 2引数 `.indexOf(a,b)` のワークスペース内ゼロ（archive/互換/外部除外）


#### **P2: Hakoruneコンパイラ MVP（次の主作業）**
- **既存**: `apps/selfhost-compiler/compiler.hako` を軸に実装（.nyash は後方受理）
- **目標**: Stage‑2/3 入力から JSON v0 を安定排出
- **直近TODO**:
  1. branch/jump 最小生成（完了✅）
  2. LocalSSA.ensure_cond 材化コピー（完了✅）
  3. Hakorune VM 代表追加（If/Compare 代表、Loop カウンタ 代表 追加済み✅）
  4. **🔥 UsingResolverBox実装（最優先・未着手）** - 詳細は下記「using解決の2つの側面」参照

#### **🔍 using解決の2つの側面（重要な洞察）**

**結論**: usingなどの解決は**コンパイラー側のUsingResolverBox実装**が核心。Hakorune VM側は既に設計完了！

##### **A. Hakorune VM側 = 既に解決済み！✅**
- MIR JSONには**解決済みのCallee::ModuleFunction**が入る
- VM側は`BoxCall`/`ExternCall`を機械的に実行するだけ
- using解決は**不要**（名前解決は事前完了）
- **現状**: M2/M3で完全動作中、追加実装不要

##### **B. コンパイラー側 = これが残課題！🔥**
```hako
// これを解析してMIR生成する機能が未実装
using "apps/lib/timer" as Timer
using "apps/lib/array_ops" as Arr

flow main() {
    local t = Timer.now_ms()  // ← シンボル解決が必要
    local a = Arr.map(...)    // ← モジュール関数解決が必要
}
```

**必要な実装（3段階）**:

1. **UsingResolverBox**（新規箱、優先度P2-A、見積もり1週間）
   ```hako
   static box UsingResolverBox {
       modules: MapBox  // "Timer" -> "/abs/path/to/timer.hako"
       symbols: MapBox  // "Timer.now_ms" -> Callee::ModuleFunction

       resolve_using(path, alias) {
           // using文を解析してmodules/symbolsに登録
       }

       get_module_path(alias) {
           // "Timer" -> "/abs/path/to/timer.hako"
       }
   }
   ```

2. **NamespaceBox**（新規箱、優先度P2-B、見積もり5日）
   ```hako
   static box NamespaceBox {
       resolve_call(namespace, method) {
           // "Timer.now_ms" -> Callee::ModuleFunction
           // UsingResolverBoxと連携
       }

       resolve_static_access(namespace, field) {
           // "Config.VERSION" -> 静的フィールドアクセス
       }
   }
   ```

3. **Pipeline V2統合**（優先度P2-C、見積もり3日）
   - CompareExtractBoxの前段階でusing解決
   - 既存のLocalSSABoxと連携
   - MIR生成時に`Callee::ModuleFunction`に変換

**関心の分離の完璧さ（設計の優秀さ）** ⭐
```
┌─────────────────┬──────────────────┐
│ コンパイラー側  │ Hakorune VM側    │
├─────────────────┼──────────────────┤
│ using解決       │ 解決済みCallee   │
│ 名前空間管理    │ 機械的実行       │
│ シンボル解決    │ 型情報不要       │
│ → 複雑          │ → シンプル       │
└─────────────────┴──────────────────┘
```
**これがPhase 15.7設計の真の優秀さ！**

Quick smokes from using/new依存削減の真の意味：
- Hakorune VM自体は**using不要で動く**（静的メソッド＋extern_call）
- using解決は**コンパイラー側の責務**
- **関心の分離**が完璧に実現されている！

#### **P3: Known/Rewrite 統合 Stage‑1 の仕上げ（dev観測）**
- 仕様は不変のまま、観測（resolve.try/choose / ssa.phi）と関数化の一貫性を高める
- **理由**: 開発者体験の向上（デバッグ情報の充実）

#### **P4: NYABI Kernel 下地の維持（未配線・既定OFF）**
- 将来の拡張性のための下地準備（Phase 16以降で本格化）

【2025-10-03 追記 — Core Kernel: TimerBox (P1)】
- ねらい: ベンチ/待機/パリティ検証のための「最小の時刻API」をコアBoxで提供する。
- Extern 仕様（最小）:
  - nyrt.time.now_ms → i64（単調時刻; ms）
  - LLVM: src/llvm_py/instructions/externcall.py にバインドを追加
  - VM(Rust): extern_call("nyrt","time.now_ms") を実装（std::time::Instant ベース）
  - WASM: JS の Date.now() で一時バインド（将来 Monotonic を検討）
- Box 仕様（最小）:
  - TimerBox.now_ms(): i64（上記 extern の薄ラップ）
  - modules 登録: selfhost.core.timer = "apps/core/timer/TimerBox.hako"（配置は後追い）
- 受け入れ（quick 最小スモーク）:
  - tools/smokes/v2/profiles/quick/core/timer_now_ms_vm.sh（2回の now_ms の差 ≥ 0）
  - tools/smokes/v2/profiles/quick/llvm/timer_now_ms_harness.sh（ハーネス時のみ; 無ければ SKIP）
- 注意:
  - まずは now_ms のみ（sleep_ms/async は別フェーズ）。
  - 壁時計と混在しないよう「単調時刻」を明記（壁時計は ClockBox として将来導入）。

【2025-10-03 実装完了メモ】
- 実装:
  - VM/LLVM/WASM へ `nyrt.time.now_ms` を配線。TimerBox は薄い導線（`apps/core/timer/TimerBox.hako`）。
  - フロント（Builder）で `TimerBox.now_ms()`/`new TimerBox().now_ms()` を `ExternCall("nyrt.time","now_ms")` に正規化。
- テスト:
  - quick: `core/timer_now_ms_vm.sh` で Result 行を検証。
  - quick/llvm: `llvm/timer_now_ms_harness.sh` はハーネス環境が無い場合 SKIP（Fail‑Fast設計）。
- 付記:
  - quick プロファイルでは LLVM/AOT の一部ケースを環境依存にしないため、ハーネス未検出・静的プラグイン未構成時は SKIP 方針に統一（詳細は各スモークスクリプト参照）。

【Core Kernel 候補（検討メモ）】
- ConsoleBox（既存）: log/print の最小API。現状維持。
- TimerBox（本件 P1）: now_ms のみ。sleep_ms は P2 以降（協調スケジューラ設計後）。
- RandomBox（候補）: seed(u64)/next_u64() のみ（テスト再現性目的）。導入は CI/シード方針が固まってから。
- EnvBox（候補）: get(name)/set(name,value) の最小。既定OFF; 影響範囲が広いので Box 境界で隔離。
- FSBox（候補）: read_file(path)/write_file(path,data) の最小。Runner/サンドボックス方針の下で将来。

## 📊 **進捗評価と残り作業（客観的評価・2025-10-03）**

### **現在位置: 80%完成、残り20%（見積もり3-4週間）**

```
Phase 15.7進捗: ████████░░ 80%完成

完了（80%）：
✅ Hakorune VM M3基盤（6命令動作実証）
  - const, binop, compare, branch, jump, ret
✅ Pipeline V2基礎実装
  - CompareExtractBox, EmitCompareBox
  - LocalSSABox導入済み
  - MirJsonBuilderMin（JSON v0生成）
✅ FlowRunner/JsonProgramBox
  - FlowEntryBox（emit-only入口）
  - Hakorune VM実行薄箱
✅ Quick smokes 172/172 PASS
  - コア常在ルール達成
  - プラグイン無効化対応
✅ TimerBox実装完了
  - nyrt.time.now_ms実装（VM/LLVM/WASM）
✅ EffectResolver一元化導線

残り20%（3-4週間）：
🔲 UsingResolverBox実装（1週間）
🔲 NamespaceBox実装（5日）
🔲 Pipeline V2統合（using）（3日）
🔲 Hakorune VM命令拡張（2週間）
  - newbox（2日・最重要）
  - boxcall（2日・最重要）
  - phi（2日）
  - load/store（2日）
  - externcall（1日）
  - unaryop/typeop（2日）
🔲 セルフホストループE2E（1週間）
```

### 📅 **進捗更新（2025-10-06）**

#### ✅ **完了済み（予想より早期達成！）**

**コンパイラー側（P2系）- 見積もり18日 → 実績2日！**
- ✅ **P2-A UsingResolverBox実装**（1日で完了、見積もり7日 → **85%短縮！**）
  - 追加: `selfhost/compiler/pipeline_v2/using_resolver_box.hako`
  - 機能: alias→path/namespace マッピング、using/modules JSON読込
  - スモーク: `quick/selfhost/selfhost_using_resolver_basic_vm.sh`

- ✅ **P2-B NamespaceBox実装**（1日で完了、見積もり5日 → **80%短縮！**）
  - 追加: `selfhost/compiler/pipeline_v2/namespace_box.hako`
  - 機能: `Timer.now_ms` → `Callee::ModuleFunction` 正規化
  - スモーク: `quick/selfhost/selfhost_namespace_box_basic_vm.sh`

- ✅ **P2-C Pipeline V2統合（using）**（1日で完了、見積もり3日 → **67%短縮！**）
  - 追加: `PipelineV2.lower_stage1_to_mir_with_usings(...)`
  - 機能: using解決後のMIR生成パイプライン

**品質強化（計画外の追加成果！）**
- ✅ **SignatureVerifierBox**（コンパイル時型検証）
  - 追加: `selfhost/compiler/pipeline_v2/signature_verifier_box.hako`
  - 機能: ビルトインメソッドのarity検証（compile-time Fail-Fast）

- ✅ **MethodRegistry拡大**（ビルトイン署名管理）
  - 追加: `apps/hakorune/vm/boxes/method_registry.hako`
  - 機能: String/Array/Map の toString/stringify/startsWith/endsWith 等を登録

- ✅ **JsonCursorBox採用**（JSONスキャン統一）
  - 機能: index_of_from/seek_array_end の一貫API化
  - 影響: minivm_probe/step_runner等の直接スキャン箇所を委譲

- ✅ **Hakorune VM への改名**（2025-10-05完了）
  - Mini-VM → Hakorune VM（ブランディング統一）
  - ファイルパス: `apps/hakorune/vm/boxes/hakorune_vm_min.hako`（正式）
  - 互換パス: `selfhost/vm/boxes/mir_vm_min.nyash`（alias維持）

#### 🔥 **現在の最優先タスク**

**Hakorune VM命令拡張（P1系）- 残り10-15%の核心**
- 🔥 **newbox実装**（2日・最重要）← **今ココ！**
  - ファイル: `apps/hakorune/vm/boxes/hakorune_vm_min.hako`
  - 目標: `new StringBox()`, `new ArrayBox()` の実行
  - 依存: なし（最初に実装可能）

- 🔥 **boxcall実装**（2日・最重要）
  - 目標: `x.length()`, `arr.push(42)` の実行
  - 依存: newbox完了

- 📝 **phi実装**（2日）
  - 目標: if/loop のPHI合流点動作
  - 依存: なし（並行実装可能）

- 📝 **load/store実装**（2日）
  - 目標: フィールドアクセス（`me.field`, `obj.field`）
  - 依存: newbox完了

- 📝 **externcall実装**（1日）
  - 目標: `print()`, `console.log()` の実行
  - 依存: なし（並行実装可能）

#### 📊 **進捗率の再評価**

```
Phase 15.7進捗: █████████░ 85-90%完成（上方修正！）

✅ 完了（85-90%）:
  ✅ P2-A/B/C（Using解決系）完全実装
  ✅ SignatureVerifier/MethodRegistry（品質強化）
  ✅ Hakorune VM基盤（InstructionScannerBox/OpHandlersBox/ProgramStateBox等）
  ✅ FlowRunner/JsonProgramBox
  ✅ Pipeline V2基礎実装
  ✅ Quick smokes 常緑（172/172 PASS）
  ✅ TimerBox実装完了
  ✅ Hakorune VM への改名完了

❌ 残り10-15%（1-2週間、下方修正！）:
  🔥 Hakorune VM命令拡張（最後の砦）
    - newbox（2日・最重要）
    - boxcall（2日・最重要）
    - phi（2日）
    - load/store（2日）
    - externcall（1日）
  🔲 セルフホストループE2E（1週間）
```

**旧見積もり**: 80%完成、残り20%（3-4週間）
**新見積もり**: **85-90%完成、残り10-15%（1-2週間）**

#### 💡 **教訓（Lessons Learned）**

1. **Box-First設計の威力**: 新機能追加が予想の**9倍速**
   - UsingResolver/Namespace実装は1日ずつで完了
   - Pipeline V2の強固な設計が成功の鍵

2. **見積もりの精度**: 初期見積もりは慎重すぎた
   - コンパイラー側: 見積もり18日 → 実績2日
   - 基盤の成熟度を過小評価していた

3. **並行開発の難しさ**: 実際は順次開発が正解
   - Using解決がモジュールシステムの基盤
   - コンパイラー完成 → Hakorune VM拡張の順が合理的

4. **品質ファーストの重要性**: 計画外の成果が大きい
   - SignatureVerifier/MethodRegistry/JsonCursorBox
   - Fail-Fast文化の確立が開発速度を加速

#### 🎯 **次の具体的アクション（1-2週間）**

**Week 1: Hakorune VM命令拡張**
- Day 1-2: newbox実装（Box生成基盤）
- Day 3-4: boxcall + phi 並行実装
- Day 5-6: load/store + externcall 並行実装
- Day 7: 統合テスト・スモークテスト

**Week 2: セルフホストループE2E**
- Day 1-2: .hakoソース→MIR JSON生成確認
- Day 3-4: MIR JSON→Hakorune VM実行確認
- Day 5-6: パリティテスト（Rust VM vs Hakorune VM）
- Day 7: ブートストラップ達成！🎉

### **作業分解（週別見積もり）**

#### **Week 1: UsingResolver + Namespace実装**
- Day 1-3: UsingResolverBox実装・テスト
- Day 4-5: NamespaceBox実装・テスト
- Day 6-7: Pipeline V2統合（using解決）

#### **Week 2: Hakorune VM命令拡張（最重要）**
- Day 1-2: newbox実装（Box生成）
- Day 3-4: boxcall実装（メソッド呼び出し）
- Day 5-6: phi実装（SSA合流）
- Day 7: load/store実装開始

#### **Week 3: Hakorune VM命令拡張（完了）**
- Day 1-2: load/store実装完了
- Day 3: externcall実装（print等）
- Day 4-5: unaryop/typeop実装
- Day 6-7: 統合テスト・スモークテスト整備

#### **Week 4: セルフホストループE2E**
- Day 1-2: .hakoソース→MIR JSON生成（コンパイラー）
- Day 3-4: MIR JSON→実行（Hakorune VM）
- Day 5-6: パリティテスト（Rust VM vs Hakorune VM）
- Day 7: ブートストラップ達成！🎉

### **達成基準（明確な終了条件）**

✅ **Phase 15.7完了 = 以下すべて満たす**:
1. UsingResolverBox/NamespaceBox動作
2. Hakorune VM 14命令すべて実装
3. .hakoソース→MIR JSON→Hakorune VM実行成功
4. c0（Rustコンパイラ）→c1（Hakoruneコンパイラ）動作
5. c1→c1'（自己コンパイル）成功
6. Quick smokes 全PASS維持

## 🚀 **セルフホスティング実現への道筋**

### 🔄 **セルフホストループの具体的4ステップ（詳細化）**

```
┌──────────────────────────────────────┐
│ Step 1: .hako ソース解析             │
│    ↓                                  │
│ Step 2: MIR JSON生成（コンパイラ）   │
│    ↓                                  │
│ Step 3: MIR JSON実行（Hakorune VM）  │
│    ↓                                  │
│ Step 4: 出力検証（パリティテスト）    │
└──────────────────────────────────────┘
```

#### **Step 1: .hakoソース解析（Rust VMで実行）**
- **入力**: `apps/selfhost-compiler/compiler.hako`
- **処理**:
  1. using文解析（UsingResolverBox）
  2. 名前空間解決（NamespaceBox）
  3. AST構築
  4. シンボルテーブル構築
- **出力**: 解決済みAST
- **検証**: `HAKO_CLI_VERBOSE=1`で解決状況確認

#### **Step 2: MIR JSON生成（コンパイラ）**
- **入力**: 解決済みAST
- **処理**:
  1. Pipeline V2実行
  2. CompareExtractBox/EmitCompareBox適用
  3. LocalSSABox適用（材化コピー）
  4. MirJsonBuilderMin実行
- **出力**: `output.json`（JSON v0形式）
- **検証**:
  ```bash
  ./target/release/hakorune --emit-mir-json output.json input.hkr
  cat output.json | jq .  # 整形表示
  ```

#### **Step 3: MIR JSON実行（Hakorune VM）**
- **入力**: `output.json`
- **処理**:
  1. FlowEntryBox初期化
  2. JsonProgramBox読み込み
  3. FlowRunner実行
  4. 命令順次実行（const/binop/compare/branch/jump/ret/newbox/boxcall/phi/load/store/externcall）
- **出力**: 実行結果（標準出力/終了コード）
- **検証**:
  ```bash
  HAKO_CLI_VERBOSE=1 ./target/release/hakorune \
    apps/selfhost/vm/flow_runner.hako -- output.json
  echo $?  # 終了コード確認
  ```

#### **Step 4: 出力検証（パリティテスト）**
- **比較対象**:
  1. Rust VM実行結果
  2. Hakorune VM実行結果
  3. 期待値（テストケース）
- **検証項目**:
  - 標準出力一致
  - 終了コード一致
  - 実行トレース一致（`HAKO_VM_DUMP_MIR=1`）
- **自動化**:
  ```bash
  # パリティテストスクリプト
  tools/smokes/v2/run.sh --profile quick --filter "selfhost_parity_*"
  ```

### 📅 **推奨実装順序（並行開発戦略）**

#### **Week 1-2: Hakoruneコンパイラ MVP完成（P2優先）**
- **Day 1-2**: branch/jump最小生成 ✅
- **Day 3**: LocalSSA.ensure_cond最終化 ✅
- **Day 4-7**: UsingResolverBox実装 🔥
- **Day 8-10**: NamespaceBox実装 🔥
- **Day 11-14**: Pipeline V2統合（using解決） 🔥

#### **Week 3: 統合＋検証**
- JSON v0完全出力
- Rust VM で実行確認
- スモークテスト整備
- **目標**: `tools/smokes/v2/profiles/quick/` 全緑維持

#### **Week 4: ブートストラップ達成**
- Hakoruneコンパイラ + Hakorune VM統合
- セルフホスティング初期検証
- 基本プログラム実行成功
- **成果**: c0→c1→c1' 完全動作

### 📊 **実装優先度マトリックス（2025-10-06更新）**

| 項目                   | 優先度   | ステータス | 理由       | 見積 | 実績 | 担当領域 |
|----------------------|-------|-------|----------|------|------|------|
| branch/jump生成        | 🔴 P2 | ✅完了 | 制御フロー必須  | 2日   | 2日 | コンパイラ |
| LocalSSA.ensure_cond | 🔴 P2 | ✅完了 | 条件分岐安定化  | 1日   | 1日 | コンパイラ |
| **UsingResolverBox実装** | 🔴 P2-A | ✅**完了** | **using解決の核心** | 1週間 | **1日**✨ | コンパイラ |
| **NamespaceBox実装** | 🔴 P2-B | ✅**完了** | 名前空間解決 | 5日 | **1日**✨ | コンパイラ |
| **Pipeline V2統合（using）** | 🔴 P2-C | ✅**完了** | using→MIR変換 | 3日 | **1日**✨ | コンパイラ |
| **SignatureVerifier** | - | ✅**完了** | **計画外追加** | - | **1日** | コンパイラ |
| **MethodRegistry拡大** | - | ✅**完了** | **計画外追加** | - | **1日** | コンパイラ |
| **JsonCursorBox採用** | - | ✅**完了** | **計画外追加** | - | **1日** | 共通 |
| **Hakorune VM改名** | - | ✅**完了** | **ブランディング統一** | - | **1日** | VM |
| **Hakorune VM newbox実装** | 🟡 P1-A | 🔥**最優先** | **Box生成（最重要！）** | 2日 | **未着手** | Hakorune VM |
| **Hakorune VM boxcall実装** | 🟡 P1-B | 🔥未着手 | **メソッド呼び出し** | 2日 | **未着手** | Hakorune VM |
| Hakorune VM phi実装 | 🟡 P1-C | 📝計画 | SSA合流 | 2日 | 未着手 | Hakorune VM |
| Hakorune VM load/store実装 | 🟡 P1-D | 📝計画 | メモリアクセス | 2日 | 未着手 | Hakorune VM |
| Hakorune VM externcall実装 | 🟡 P1-E | 📝計画 | print等外部呼び出し | 1日 | 未着手 | Hakorune VM |
| match式完全対応 | 🟡 P1-F | 📝計画 | 頻繁に使用 | 2日 | 未着手 | コンパイラ |
| Hakorune VM unaryop/typeop | 🟢 P3-A | 📝計画 | 単項演算・型操作 | 2日 | 未着手 | Hakorune VM |
| 最適化パス | 🟢 P3-B | 📝計画 | 性能向上 | 1週間 | 未着手 | コンパイラ |
| エラーハンドリング | 🟢 P3-C | 📝計画 | UX向上 | 3日 | 未着手 | コンパイラ |

**凡例**:
- 🔴 P2: 最優先（セルフホスティング必須）
- 🟡 P1: 高優先度（基本機能実装）
- 🟢 P3: 中優先度（改善・UX向上）
- ✅完了 / 🔥最優先 / 🔥未着手 / 📝計画 / ✨予想より早い達成

**達成状況**:
- ✅ **P2系完全達成**（コンパイラー側：using解決・品質強化）
- 🔥 **P1系が最優先**（Hakorune VM命令拡張：残り10-15%）

**重要な優先順位**:
1. ~~**P2-A/B/C（using解決）**~~: ✅完了！（見積もりの9倍速）
2. **P1-A/B（newbox/boxcall）**: ← **今ココ！** これだけでほとんどの.hakoが動く
3. **P1-C/D/E（phi/load/store/externcall）**: 完全動作に必要

### 🎯 **具体的な実装提案**

#### **Option A: 並行開発（推奨✨）**

**トラック1（Hakoruneコンパイラ）**:
- 担当: ChatGPT + Claude
- 期間: 3週間
- 成果: 完全なHakoruneコンパイラ
- ファイル: `apps/selfhost-compiler/`

**トラック2（Hakorune VM拡張）**:
- 担当: ChatGPT + Claude
- 期間: 2週間
- 成果: 完全な Hakorune VM（M4-M7）
- ファイル: `selfhost/vm/boxes/mir_vm_min.nyash`

**統合**:
- 期間: 1週間
- 成果: セルフホスティング達成

**総期間**: 4週間

#### **Option B: 順次開発**

1. コンパイラ完成（3週間）
2. Hakorune VM完成（2週間）
3. 統合（1週間）

**総期間**: 6週間

### 💎 **Hakorune VM実装のメリット**

#### **1. 教材として最高**
```hakorune
// Hakorune で Hakorune VM を書く
box MirVmMin {
    registers: MapBox
    blocks: MapBox

    execute(mir_json) {
        // MIR実行ロジック
        // これ自体が教材になる！
    }
}
```

#### **2. デバッグの容易さ**
- **Rust VM**: コンパイル必要、デバッグ困難
- **Hakorune VM**: 即座に修正、即座に実行

#### **3. 完全な制御**
- **Rust VM**: 複雑、変更リスク大
- **Hakorune VM**: シンプル、実験容易

#### **4. セルフホスティングへの道**
```
Hakoruneコンパイラ（Hakorune実装）
    ↓ MIR生成
Hakorune VM（Hakorune実装）
    ↓ 実行
完全なセルフホスティング達成！🎉
```

### 🎯 **受け入れ条件（Acceptance Criteria）**

#### **Phase 15.7完了基準**

1. **quick プロファイル**: 全緑維持（96/96 PASS）
   - Hakorune VM（M2/M3）代表スモーク緑
   - const/binop/compare/branch/jump/ret 動作確認
   - call/boxcall/newbox（最小意味論）実行スモーク緑
   - v1 `mir_call` 形状スモーク（VM-only）緑
   - LLVM/PHI compile-only スモーク緑（if-merge / loop）

2. **integration プロファイル**: 代表パリティ緑（llvmlite/ハーネス）
   - VM↔LLVM↔Ny のパリティ一致

### 🔧 ENV クイックリファレンス（関連）
- `NYASH_PIPELINE_V2=1` — Selfhost Pipeline V2 を有効化
- `NYASH_LLVM_USE_HARNESS=1` — LLVM llvmlite ハーネス経路
- `NYASH_LLVM_PHI_STRICT=1` — PHI: create-only（PhiHandler）/ wiring（finalize）
- `NYASH_JSON_SCHEMA_V1=1` — JSON v1（mir_call）を有効化（shape 検証用）
- `NYASH_LLVM_DOWNGRADE_V1=1` — ハーネス出力時に v1→v0 ダウングレード（compile-only 安定化）
- `NYASH_VM_USE_PY=1` — PyVM 経路（開発/比較用）


3. **Builder観測**: resolve.try/choose と ssa.phi が dev‑only で取得可能
   - 環境変数: `HAKO_DEBUG_*`

4. **表示API統一**: QuickRef/ガイドが `str()` に統一
   - 実行挙動は従前と同じ（互換性維持）

5. **Selfhost Compiler（dev限定）**:
   ```bash
   HAKO_JSON_ONLY=1 ./target/release/hakorune \
     apps/selfhost-compiler/compiler.hako -- --stage3   # 互換: compiler.nyash も受理
   ```
   → JSON ヘッダ（`{"version":…, "kind":…}`）を出力（非空）

6. **ブートストラップ成功**:
   ```bash
   # c0（Rustコンパイラ）→ c1（Hakoruneコンパイラ）
   ./target/release/hakorune apps/selfhost-compiler/compiler.hako \
     -- input.hkr > output.json

   # c1 → c1'（自己コンパイル）
   ./target/release/hakorune selfhost/vm/boxes/mir_vm_min.nyash \
     -- output.json
   ```

補足（Branding/Flags の整理）
- 設定ファイルは `hako.toml` を優先（互換: `nyash.toml`/`hakorune.toml`）。
- プラグイン仕様は `hako_box.toml` を優先（互換: `nyash_box.toml`）。
- 環境変数の公式前置詞は `HAKO_*`（互換: `HAKU_*`/`HRN_*`/`NYASH_*`）。

## 📋 **実装タスクリスト（小粒・段階的）**

### **Phase 1: Hakoruneコンパイラ基本強化（Week 1-2）**

1. **branch/jump最小生成実装**（2日）
   - ファイル: `apps/selfhost-compiler/boxes/mir_emitter_box.nyash`
   - 目標: if/loop の制御フローを JSON v0 で正しく出力
   - 検証: Rust VM で実行して期待値一致

2. **LocalSSA.ensure_cond最終化**（1日）
   - ファイル: `apps/selfhost-compiler/builder/ssa/local.nyash`
   - 目標: 条件分岐前の材化コピー完全動作
   - 検証: compare/branch の組み合わせテスト

3. **基本構文完全対応**（4日）
   - if/else（完了✅）
   - loop（実装中🔄）
   - call/method（実装中🔄）
   - new/me（計画📝）

### **Phase 2: Hakorune VM拡張（Week 2-3、並行可能）**

4. **M4: ループサポート**（3日）
   - ファイル: `selfhost/vm/boxes/mir_vm_min.nyash`
   - 目標: loop命令の実行
   - 検証: 累積計算テスト

5. **M5: Box操作サポート**（2日）
   - new/field access/method call
   - 検証: StringBox/ArrayBox基本操作

6. **M6-M7: プラグインBox対応**（2日）
   - FileBox/PathBox統合
   - 検証: パリティテスト全PASS

### **Phase 3: 統合＋ブートストラップ（Week 4）**

7. **統合テスト**（3日）
   - Hakoruneコンパイラ + Hakorune VM連携
   - JSON v0 完全出力確認
   - スモークテスト整備

8. **ブートストラップ達成**（4日）
   - c0→c1コンパイル成功
   - c1→c1'自己コンパイル成功
   - パリティテスト合格

## 🔧 **開発環境・ツール**

### **スモークテスト実行**
```bash
# quick プロファイル（全体）
tools/smokes/v2/run.sh --profile quick

# セルフホスティング関連のみ
tools/smokes/v2/run.sh --profile quick --filter "selfhost_*"

# Hakorune VM テスト
tools/smokes/v2/run.sh --profile quick --filter "selfhost_mir_m*"
```

### **手動テスト**
```bash
# Hakoruneコンパイラ実行
./target/release/hakorune apps/selfhost-compiler/compiler.hako \
  -- --stage3 sample.hkr > output.json

# Hakorune VM実行
./target/release/hakorune selfhost/vm/boxes/mir_vm_min.nyash \
  -- output.json

# Rust VM比較
./target/release/hakorune --backend vm sample.hkr
```

### **デバッグ用環境変数**
```bash
# 詳細診断
HAKO_CLI_VERBOSE=1

# MIR出力
HAKO_VM_DUMP_MIR=1
./target/release/hakorune --dump-mir program.hkr

# JSON IR出力
./target/release/hakorune --emit-mir-json debug.json program.hkr

# セルフホスト専用
HAKO_JSON_ONLY=1      # JSON のみ出力
HAKO_COMPILER_TRACK=1 # コンパイラトラック有効化
```

## 🎊 **Phase 15.7完了の意義**

### **技術的成果**
1. ✅ **完全なセルフホスティング達成**
   - Hakoruneで Hakorune をコンパイル
   - 外部コンパイラ依存からの完全解放

2. ✅ **教材として最高の価値**
   - コンパイラ実装: `apps/selfhost-compiler/` 全体
   - Hakorune VM実装: `selfhost/vm/boxes/mir_vm_min.nyash`
   - 完全な理解が可能な規模

3. ✅ **保守性の革命**
   - Hakorune でコンパイラを書く → 誰でも改造可能
   - MIR 13命令 → 究極のシンプルさ
   - Everything is Box哲学の完成

### **次のマイルストーン（Phase 16）**
- 最適化パス追加（デッドコード削除、インライン化）
- エラーメッセージ改善
- LLVM バックエンド完全統合
- ネイティブバイナリ生成（EXE化）

## 📚 **関連ドキュメント**

### **Phase 15シリーズ**
- [Phase 15: セルフホスティング全体計画](../phase-15/README.md)
- [Phase 15.5: Core Box統一](../phase-15.5/README.md)
- [Phase 15.6: MIR Call革新](../phase-15.6/README.md)

### **実装ガイド**
- [セルフホスティング戦略 2025年9月版](../phase-15/implementation/self-hosting-strategy-2025-09.md)
- [LLVM EXE生成戦略](../phase-15/implementation/llvm-exe-strategy.md)

### **言語リファレンス**
- [Quick Reference](../../../../reference/language/quick-reference.md)
- [完全言語仕様](../../../../reference/language/LANGUAGE_REFERENCE_2025.md)
- [MIR命令セット](../../../../reference/mir/INSTRUCTION_SET.md)

## 🌟 **結論**

**「VM層も一緒に作った方が楽」 = 絶対YES！✨**

理由:
- ✅ 相互検証で品質向上
- ✅ デバッグ容易
- ✅ 完全な理解
- ✅ 教材として最高
- ✅ セルフホスティング直結

**推奨**: コンパイラとHakorune VMを並行開発！4週間でセルフホスティング達成！

---

背景（技術詳細）
- Instance→Function 正規化の方針は既定ON。Known 経路は関数化し、VM側は単純化する。
- resolve.try/choose（Builder）と ssa.phi（Builder）の観測は dev‑only で導入済み（既定OFF）。
- Hakorune VM は M2/M3 の代表ケースを安定化（パス/境界厳密化）。
- VM Kernel の Ny 化は後段（観測・ポリシーから段階導入、既定OFF）。

優先順（2025‑09‑29 リバランス / 2025‑10‑04 反映）
- P0: Rust VM 層の安定化（既存バグの点修正・回帰防止）
  - 受け手推定・RouterPolicy・LocalSSA/材化・VarMapGuard 等の補強を優先（quick/integration 常緑）。
- P1: Hakorune VM 仕上げ（完了）
  - M2/M3 の代表＋エッジスモークを quick に追加し、単一パス＋厳密セグメントで緑維持。
- P2: Nyash コンパイラ MVP（Phase 15.6）の前進（次の主作業）
- 既存 `apps/selfhost-compiler/compiler.hako` を軸に、Stage‑2/3 入力から JSON v0 を安定排出（.nyash は後方受理）。
  - 受け入れ（dev限定）: `NYASH_JSON_ONLY=1` で `version/kind` を含む JSON ヘッダが非空であること。
  - 既定挙動は不変。コンパイラは別アプリ（apps/）として進め、VM/LLVM 本線は影響最小。
  - 直近 TODO: branch/jump 最小生成＋LocalSSA.ensure_cond の材化コピー最終化、Hakorune VM 代表追加1件。
- P3: Known/Rewrite 統合 Stage‑1 の仕上げ（dev観測）
  - 仕様は不変のまま、観測（resolve.try/choose / ssa.phi）と関数化の一貫性を高める。
- P4: NYABI Kernel 下地の維持（未配線・既定OFF）

Compiler Track（大規模変更の部分解禁 — apps/selfhost-compiler/ 限定）
- 目的: Selfhost Compiler を段階的に実用化。Core（src/）は引き続き安定運用。
- ガード:
  - 既定OFFのフラグ/引数（例: `NYASH_COMPILER_TRACK=1`, `--min-json`, `--emit-mir`）。
  - quick/integration 常緑を維持。影響は Selfhost 実行時に限定。
- 受け入れ（dev）:
  - `NYASH_JSON_ONLY=1 ... --min-json` で JSON ヘッダ非空。
  - `--emit-mir` で最小 MIR(JSON v0)（const→ret）を生成可能。

Unified Call（開発既定ON）
- 呼び出しの統一判定は、環境変数 `NYASH_MIR_UNIFIED_CALL` が `0|false|off` でない限り有効（既定ON）。
- メソッド解決/関数化を `emit_unified_call` に集約し、以下の順序で決定:
  1) 早期 toString/stringify→str
  2) equals/1（Known 優先→一意候補; ユーザーBox限定）
  3) Known→関数化（`obj.m → Class.m(me,…)`）／一意候補フォールバック（決定性確保）
- レガシー側の関数化は dev ガードで抑止可能: `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1`（移行期間の重複回避）

スコープ（やること）
1) Builder: Known 化 + Rewrite 統合（Stage‑1）
   - P0: me 注入・Known 化（origin 付与/維持）— 軽量PHI補強（単一/一致時）
   - P1: Known 経路 100% 関数化（obj.m → Class.m(me,…)）。special は `toString→str（互換:stringify）/equals` を統合
   - 観測: resolve.try/choose / ssa.phi を dev‑only で JSONL 出力（既定OFF）。`resolve.choose` に `certainty` を付加し、KPI（Known率）を任意出力（`NYASH_DEBUG_KPI_KNOWN=1`, `NYASH_DEBUG_SAMPLE_EVERY=N`）。

2) 表示APIの統一（挙動不変）
   - 規範: `str()` / `x.str()`（同義）。`toString()` は早期に `str()` へ正規化
   - 互換: `stringify()` は当面エイリアスとして許容
   - QuickRef/ガイドの更新（plus混在の誘導も `str()` に統一）

3) Hakorune VM（MirVmMin）安定化（devのみ）
   - 厳密セグメントによる単一パス化、M2/M3 代表スモーク常緑（const/binop/compare/branch/jump/ret）
   - パリティ: VM↔LLVM↔Ny のミニ・パリティ 2〜3件

4) NYABI（VM Kernel Bridge）下地（未配線・既定OFF）
   - docs/abi/vm-kernel.md（関数: caps()/policy.*()/resolve_method_batch()）
   - スケルトン: selfhost/vm/boxes/vm_kernel_box.nyash（policy スタブ）
 - 既定OFFトグル予約: NYASH_VM_NY_KERNEL, *_TIMEOUT_MS, *_TRACE

非スコープ（やらない）
- 既定挙動の変更（Rust VM/LLVMが主軸のまま）
- PHI/SSAの一般化（Phase 16 で扱う）
- VM Kernel の本配線（観測・ポリシーは dev‑only/未配線）

リスクと軽減策
- 性能: 境界越えは後Phaseに限る（本Phaseは未配線）。Hakorune VMは開発補助で性能要件なし。
- 複雑性: 設計は最小APIに限定。拡張は追加のみ（後方互換維持）。
- 安全: すべて既定OFF。Fail‑Fast方針。再入禁止/タイムアウトを仕様に明記。

受け入れ条件（Acceptance）
- quick: Hakorune VM（M2/M3）代表スモーク緑（const/binop/compare/branch/jump/ret）
- integration: 代表パリティ緑（llvmlite/ハーネス）
- Builder: resolve.try/choose と ssa.phi が dev‑only で取得可能（NYASH_DEBUG_*）
- 表示API: QuickRef/ガイドが `str()` に統一（実行挙動は従前と同じ）
- Unified Call は開発既定ONだが、`NYASH_MIR_UNIFIED_CALL=0|false|off` で即時オプトアウト可能（段階移行）。
- Selfhost Compiler（dev限定・任意ゲート）:
- `NYASH_JSON_ONLY=1 ./target/release/nyash apps/selfhost-compiler/compiler.hako -- --stage3` が JSON ヘッダ（`{"version":…, "kind":…}`）を出力（非空）。

実装タスク（小粒）
1. origin/observe/rewrite の分割方針を CURRENT_TASK に反映（ガイド/README付き）
2. Known fast‑path の一本化（rewrite::try_known_rewrite）＋ special の集約
3. 表示APIの統一（toString→str、互換:stringify）— VM ルータ特例の整合・ドキュメント更新
4. MirVmMin: 単一パス化・境界厳密化（M2/M3）・代表スモーク緑
5. docs/abi/vm-kernel.md（下書き維持）・スケルトン Box（未配線）

トグル/ENV（予約、既定OFF）
- NYASH_VM_NY_KERNEL=0|1
- NYASH_VM_NY_KERNEL_TIMEOUT_MS=200
- NYASH_VM_NY_KERNEL_TRACE=0|1

ロールバック方針
- Hakorune VMの変更は apps/selfhost/ 配下に限定（本線コードは未配線）。
- NYABIは docs/ と スケルトンBoxのみ（実行経路から未参照）。
- Unified Call は env で即時OFF可能。問題時は `NYASH_MIR_UNIFIED_CALL=0` を宣言してレガシーへ退避し、修正後に既定へ復帰。

補足（レイヤー・ガード）
- builder 層は origin→observe→rewrite の一方向依存を維持する。違反検出スクリプト: `tools/dev/check_builder_layers.sh`

関連（参照）
- Phase 15（セルフホスティング）: ../phase-15/README.md
- Phase 15.5（基盤整理）: ../phase-15.5/README.md
- Known/Rewrite 観測: src/mir/builder/{method_call_handlers.rs,builder_calls.rs}, src/debug/hub.rs
- QuickRef（表示API）: docs/reference/language/quick-reference.md
- Hakorune VM: selfhost/vm/boxes/mir_vm_min.nyash
- スモーク: tools/smokes/v2/profiles/quick/core/

更新履歴
- 2025‑10‑06 v3（本版）: "Mini-VM" → "Hakorune VM" ブランディング統一、進捗反映（85-90%完成）
- 2025‑09‑28 v2: Known 化＋Rewrite 統合（dev観測）、表示API `str()` 統一、Hakorune VM 安定化へ焦点を再定義
- 2025‑09‑28 初版: Hakorune VM M3 + NYABI下地の計画

## ステータス（2025‑09‑28 仕上げメモ）
- M3（compare/branch/jump）: Hakorune VM（MirVmMin）が厳密セグメントの単一パスで動作。代表 JSON 断片で compare(Eq)→ret、branch、jump を評価。
- 統合スモーク: integration プロファイル（LLVM/llvmlite）は PASS 17/17（全緑）。
- ルータ／順序ガード（仕様不変）:
  - Router: 受信者クラスが Unknown のメソッド呼び出しは常にレガシー BoxCall にフォールバック（安定性優先・常時ON）。
  - Router（補足）: `InstanceBox × {length,len,substring,indexOf,lastIndexOf}` は Unified に固定し、`StringBox` 正規化へ導く（VM救済に依存しない）。
  - BlockSchedule: φ→Copy(materialize)→本体(Call) の順序を dev‑only で検証（`NYASH_BLOCK_SCHEDULE_VERIFY=1`）。
  - LocalSSA: 受信者・引数・条件・フィールド基底を emit 直前で「現在のブロック内」に必ず定義。
- VM 寛容フラグの方針:
  - `NYASH_VM_TOLERATE_VOID`: dev 時の救済専用（quick テストからは除去）。
  - Router の Unknown→BoxCall は常時ON（仕様不変・安定化目的）。

## 次のTODO（短期）
- Rust VM 安定化（点補修の仕上げ）
  - 既知箇所の観測を最小ONで確認（必要時のみ）。
- json_query_vm（VM）: LocalSSA/順序の取りこぼし補強（救済OFFで緑）。
- ループ PHI 搬送: header/合流の VarMapGuard 観測（break/continue を安定）。
- Hakorune VM M2/M3: 追加エッジ（複数compare/ret先頭/ゼロ除算/no‑retフォールバック）を quick で常緑（完了済）。
- Selfhost Compiler（dev）: JSONヘッダ非空スモーク（任意ゲート）を準備。

## Builder 小箱（Box 化）方針（仕様不変・段階導入）
- S-tier（導入）:
  - MetadataPropagationBox（型/起源伝播）: `metadata/propagate.rs`
  - ConstantEmissionBox（Const発行）: `emission/constant.rs`
  - TypeAnnotationBox（最小型注釈）: `types/annotation.rs`
  - RouterPolicyBox（Unified vs BoxCall ルート）: `router/policy.rs`
  - EmitGuardBox（emit直前の最終関所）: `emit_guard/mod.rs`
  - NameConstBox（関数名Const生成）: `name_const.rs`
- A/B-tier（計画）:
  - Compare/BranchEmissionBox、PhiWiringBox、EffectMask/TypeInferenceBox（Phase16以降）

採用順（小さく安全に）
1) Const → metadata → 最小注釈の順に薄く差し替え（代表箇所→全体）
2) RouterPolicyBox を統一Call経路に導入（utils側は後段で移行）
3) EmitGuardBox で Call 周辺の finalize/verify を集約（Branch/Compare は後段）
4) NameConstBox を rewrite/special/known に段階適用

ドキュメント
- 詳細は `docs/development/builder/BOXES.md` を参照。

## Unskip Plan（段階復帰）
- P0: json_query_vm → 期待出力一致、寛容フラグ不要。
- P1: loops（break/continue/loop_statement）→ PHI 搬送安定。
- P2: Hakorune VM（M2/M3）→ 代表4件 PASS、coarse 撤去・単一パス維持。


【2025-10-05 更新】Hakorune‑VM と箱の適用／WASM toolchain ゲート

- Hakorune‑VM の配置と alias（段階移行）
  - 追加: `apps/hakorune/vm/boxes/hakorune_vm_min.hako`
  - `hako.toml` に `[modules] hakorune.vm.mir_min` を追加（旧 `selfhost.vm.mir_min` は1リリース alias 維持）
  - FlowRunner/dev サンプルの using を hakorune alias へ寄せ（印字は `run`、実行は `run_min`）

- 箱（Box‑First）の導入と責務分解（薄い境界）
  - InstrDecoderBox: JSON v0 のオブジェクト走査（`InstructionScannerBox.next` 委譲）
  - ProgramStateBox: `regs/bb/prev_bb/steps` の状態管理（write 全面／read は段階導入中）
  - CfgNavigatorBox: ブロック head/tail と `index_of_from` を集約し、VM/Scanner から委譲
  - RetResolverBox: `ret` 値決定を一本化（last_cmp 優先→regs→Fail‑Fast）
  - PhiWiringBox: Φ 適用の薄いラッパ（既存 apply に委譲）
  - DiagnosticsBox: 既定静音、`{"__dev__":1}` マーカーで `debug()` を有効化（将来 CLI→Box 橋渡し）

- 実装ポイント（HakoruneVmMin）
  - `run_min` を標準入口に統一（`run` は印字付きアダプタ）
  - `ProgramStateBox` を regs/steps/bb/prev に導入（write 全面、read を段階拡大）
  - `CfgNavigatorBox` の head/tail、`index_of_from` へ呼び替え（重複排除）
  - `RetResolverBox.resolve` へ ret 判定を委譲（診断は Diagnostics に一元化）

- スモーク（代表最小・増やしすぎ回避）
  - 追加: `hakorune_vm_m2_eq_true_vm.sh`
  - 追加: `hakorune_vm_m3_branch_true_vm.sh` / `..._branch_false_vm.sh` / `..._jump_vm.sh` / `..._jump_chain_vm.sh` / `..._phi_diamond_vm.sh`
  - Builder auto‑birth のクロスモジュール確認: `builder_autobirth_cross_module_{vm,alias}_vm.sh`（常時ONに寄せ）

- using/[modules] の扱い
  - `hakorune.vm.mir_min` への alias 寄せを段階実施（selfhost.* は1リリース alias 維持）
  - E2E は代表1本に限定して確認（増やしすぎ回避）

- WASM: llvmlite 制限の回避（オプトイン・フォールバック有）
  - `NYASH_LLVM_WASM_TOOLCHAIN=1` で `llc + wasm-ld` 経路を使用（既定OFF）
  - 追加エクスポートは `NYASH_WASM_EXPORTS="Main.main,main"`（既定で `ny_main`）
  - 失敗時は llvmlite `emit_object` にフォールバック（既定挙動は不変）
  - ドキュメント追記: `docs/guides/execution-modes-guide.md` にツールチェイン手順を追加

- 次の小粒 TODO（48h）
  - ProgramStateBox の read を残差2箇所で get 化（bb/prev_bb 一貫性）
  - CfgNavigatorBox への `index_of_from` 呼びを段階置換（VM 側の重複を削減）
  - CLI→DiagnosticsBox の DEV 橋渡し（起動時に dev マーカー注入）
  - hakorune_* の代表を1本だけ追加（compare→branch→phi entry の複合）


【2025-10-05 追記 2】DEV ブリッジと箱化の仕上げ（FlowRunner/CLI／スキャン統一／プラグイン検証）

- DEV ブリッジ（CLI 主導）
  - CLI（Rust）で `NYASH_DEV_JSON_MARKER=1` の時、MIR JSON 出力に `{"__dev__":1}` を付与（`src/runner/mir_json_emit.rs`）。
  - FlowRunner は注入を既定OFFにし、将来用の薄い導線 `_maybe_inject_dev_marker(j, ast_json)` を追加。
    - AST JSON 側に `{"__cli_dev__":1}` が含まれる時のみ `{"__dev__":1}` に正規化（既定は未使用）。
  - HakoruneVmMin は `{"__dev__":1}` を検出して DiagnosticsBox.debug を有効化（既定静音）。

- ProgramStateBox の read を debug/trace でも統一
  - `bb/prev_bb` は ProgramStateBox から取得（write/read の一貫化）。
  - 情報系ログは DiagnosticsBox.debug（DEVのみ）に寄せ、[ERROR]のみ従来出力。

- CfgNavigatorBox 委譲の仕上げ（ホットパス外から）
  - InstructionScannerBox に加え、MiniVmScan（selfhost/common）側でも `index_of_from` を委譲。
  - JSON スキャン系の重複ロジックを段階的に削減（性能影響は最小）。

- プラグイン birth 仕様の固定（E2E）
  - 既存: `plugin_birth_e2e_vm.sh`（明示 birth の冪等）、`plugin_no_birth_nop_vm.sh`（no‑birth は no‑op）。
  - 追加: `plugin_autobirth_e2e_vm.sh`（new→method の auto‑birth）。
  - 追加（opt‑in）: `userbox_birth_idempotent_vm.sh`（二回目 birth が no‑op、パーサ制約のためゲート付与）。

—

この先（セルフホスティングに向けた大まかな計画）

1) コンパイラ最小ルートの完成（Ny→JSON v0→MIR(JSON v0)）
   - Stage‑1: AST→JSON v0 の出力見直し（UsingResolverBox の導入、prelude 安全）。
   - Stage‑2: MIR(JSON v0) の最小命令（const/binop/compare/branch/jump/ret/phi）を安定排出。
   - 代表ケース（const→ret／compare→ret／compare→branch→phi）で Hakorune‑VM 実行＝緑。

2) VM/箱の仕上げ（薄い境界を維持）
   - ProgramStateBox: read 残差の完全移行（log/trace/観測点まで get 化）。
   - CfgNavigatorBox: 依存箇所の委譲をもう一段だけ拡大（重複ロジックの排除）。
   - RetResolverBox/DiagnosticsBox: 失敗系と観測の一元化（Fail‑Fast＋DEV静音）。

3) DEV ブリッジの一本化
   - 既定は CLI→MIR JSON emit で `{"__dev__":1}` を付与。
   - FlowRunner の `_maybe_inject_dev_marker` は将来の AST 側からの橋渡し用（既定OFFのまま）。

4) スモーク/テスト（増やしすぎず代表性重視）
   - hakorune_*: m2/m3 の代表を1本ずつ（既存 eq_true / branch / jump / phi + 複合1本）。
   - プラグイン系: auto/明示/no‑birth の3本で固定。userbox 冪等は opt‑in で補助。

5) ドキュメントとエコシステム
   - Phase‑15.7 の更新点を継続追記（箱の責務・導線・ゲート説明）。
   - CI は quick 緑維持、重い/環境依存は opt‑in（ゲート付き運用）。


## 2025-10-06 Update — 小粒前進（緑維持）

今回の着地（strict化＋箱境界で安定化）
- VM 再帰ガードの導入（tail fallback）
  - ModuleFunction の tail フォールバックで即時自己参照を Fail‑Fast（循環検出）。
  - `NYASH_VM_REENTER_LIMIT` と併用して開発時の暴走を抑止。
- Wide フォールバックの停止（quick/integration/full）
  - `NYASH_VM_GLOBAL_TAIL_FALLBACK=0`、`NYASH_VM_MODULE_TAIL_WIDE=0` を各プロファイルの env に設定。
  - strict 解決を既定に戻し、偶発的な再解決ループを構造的に遮断。
- EntryBox の導入と浸透（呼び名の安定化）
  - `selfhost.vm.entry` / `hakorune.vm.entry` を追加。スモークは EntryBox 経由に統一。
  - 役割: 呼び名固定・解決バグ遮断・観測/契約の入口集約・差替え容易化。
- String/JSON スキャナの安定化（Ny 実装）
  - `StringScanBox.read_char` を自己再帰から正常実装へ修正、`find_quote` 追加。
  - `JsonScanBox.seek_obj_end/seek_array_end` から文字列スキップに `find_quote` を使用。
- スモーク状況
  - quick: selfhost m2/m3（eq_true/eq_false/branch/jump）・hakorune m2 eq true → PASS
  - integration: LLVM ハーネス 30/30 → PASS
  - full: suites/core（代表） → PASS

仕様/契約まわり（現状）
- auto‑birth 既定（unbornのみ抑止）。instance.birth() 受理。birthは冪等。
- プラグイン birth 未定義は移行期 no‑op 合成（1リリース相当）。
- unborn Fail‑Fast（ユーザーBox）を既定ON（`NYASH_CHECK_CONTRACTS=1`）。

次アクション（Phase 15.7 継続・ブレークダウン）
1) VM: Throw terminator の最小対応とスモーク有効化（非到達側の PHI 除外は実装済み）
2) Resolver: [modules] もう1本 E2E 追加＋ログ衛生（dev でのみ詳細）
3) Macro/call!: .hako 実装の堅牢化（ネスト・エスケープ）と自己ホストへの小規模導入拡張
4) Mini‑VM: binop/compare カバレッジ拡大と代表スモーク追加（ホットパスのログ抑制）
5) Lints: report→fail の段階移行（noise管理しつつ）

工数目安（合計 8〜16日）
- Throw/PHI/LLVM/マクロ/Resolver 小粒を並列に、常に quick 緑を維持する方針。

参照
- EntryBox: `selfhost/vm/boxes/mini_vm_entry.hako`, `apps/hakorune/vm/boxes/hakorune_vm_entry.hako`
- 環境: `tools/smokes/v2/configs/env/{quick,integration,full}.env`
- スモーク: `tools/smokes/v2/profiles/quick/selfhost/*`, `tools/smokes/v2/profiles/integration/*`, `tools/smokes/v2/suites/core/*`


共通化（Compiler/VM 共有ポリシーの導入）
- call_policy（Rust, `src/common/call_policy.rs`）
  - 即時循環判定（base名一致でFail‑Fast）と tail 候補生成を関数化。
  - VM の Global/ModuleFunction の末尾一致系をこのポリシーで統一。
  - 影響ファイル: `src/backend/mir_interpreter/handlers/calls/function.rs`（wide=OFF前提でも安定）。
- 次段（提案）
  - LifecycleContracts（birth/unbornの診断メッセージ・Fail‑Fast文言統一）
  - ExternCallRegistry（Timer.now_ms 等の外部呼び出し名→Externの一元化）
  - PhiCore（到達不能pred除外の判定を共有化、Bridge/VM/LLVM で共通ルールに）


### 2025-10-07 — Commonization (Lifecycle/Extern/Phi)

- Added `src/common/lifecycle_contracts.rs`: unborn diagnostics + birth idempotence helpers.
- Added `src/common/extern_registry.rs`: facade over MIR extern registry; builder routes check `exists()`.
- Added `phi_core::common::is_unreachable_pred()` and routed builder to it.
- VM now uses unified unborn guard/message and birth recording via common helpers.
- Unit tests: unchanged count but quick run remains green.

## 🆕 Updates (2025-10-07)

- CompareScanBox（v0/v1 正規化）を導入し、Hakorumne VM/自家製 Mini‑VM の compare 抽出を箱に一本化（残差も段階置換）。
- RetResolveSimpleBox を導入し、Mini‑VM の末尾フォールバック順を明示（last_cmp → ret解決 → first const）。
- with_usings E2E 緑化（dev / Module‑First）:
  - VM 自動登録ブリッジ（NYASH_VM_AUTO_REGISTER_DIR_NS=1）を builder/ssa/.hako に拡張（Module‑First 時のみ）。
  - UsingResolverBox に `upgrade_aliases()` を追加し、modules_map を使って alias→full ns に昇格（tail 一意）。
  - PipelineV2.with_usings 入口で upgrade_aliases を呼び、NamespaceBox 正規化の前提を強化。
- Mini‑VM トレース導線（dev）:
  - `__trace__=1` を JSON 先頭に置くと [DEBUG] を出力。`MiniVmEntryBox.run_trace(json)` が注入を補助。
  - Docs: Namespace Quickstart に Mini‑VM Debugging を追記。

### Next (small & safe)
1) Module‑First bridge の対象拡張（exports の広い拾い上げ）と失敗時診断（missing ModuleFunction）を1行で明示。
2) CompareScanBox の完全適用（Hakorumne 側 compare の残差置換）と最小スモークを1本追加。
3) with_usings の追加E2E（別 alias）を1本だけ quick に常設（過多回避）。
4) Mini‑VM Debug の独立ページ（docs/guides/mini-vm-debugging.md）を作成し、CLI/ENV/落とし穴（末尾数値の抽出）を記載。


## Rust ↔ Selfhost Coverage (Gap Analysis) — 2025‑10‑12

What Rust has today (reference) vs What Selfhost has wired, and how we close the gaps in Phase 15.7.

- MIR instruction coverage
  - Implemented (Selfhost Mini‑VM): const, binop(Add/Sub/Mul/Div/Mod), compare(Eq/Ne/Lt/Le/Gt/Ge), branch, jump, ret, copy, (early) phi apply/scan
  - Pending (Selfhost): load/store, typeop (is/cast), safepoint/barrier, throw/try, nop (no‑op semantics), unary (neg/not) — plan: P4/P5 small adapters, rc‑only smokes

- Call shapes (builder/emitter → VM/LLVM)
  - Implemented: mir_call v1/v0 for Extern(Global op_eq), Constructor/Method (minimal), Global(print/JSON.stringify) E2E via v1→v0 adapter
  - Pending: BoxCall minimal parity, ModuleFunction tail normalize (strict), Method arities beyond size/indexOf/substring — plan: P4 thin adapters + SSOT arity table first

- Registry/SSOT
  - Implemented: slots/arity/aliases SSOT at runtime (TypeRegistry), diagnostics unification
  - Pending: full SSOT for type resolution (type_id, box table), generator‑first flow — plan: promote SSOT to the sole source; keep static as fallback until green

- Plugins (Provider/Handles)
  - Implemented: early provider load (nyash.toml), identity cache for PluginBox/HostHandle
  - Pending: full handle TLV coverage for map.values/keys in plugin impl, deterministic ordering policy (or tests made order‑agnostic) — plan: Stage‑2 host handle tests remain opt‑in

- Using / Modules
  - Implemented: prelude merge, alias resolver, quick profile ON; opt‑in selfhost suite with gates
  - Pending: always‑on module aliases for selfhost.* in dev runner — plan: keep opt‑in to avoid CI drift; docs updated

### Next Steps (Phase 15.7 finalize)
- P4 (adapters): wire NewBox/Call/Method helpers to shared MirSchema/BlockBuilder and keep output‑compatible; add 1–2 rc‑only smokes
- P5 (LocalSSA): enable ensure_calls/ensure_cond mini; add 1 rc‑only smoke and un‑skip 1 M2/M3 target
- SSOT: prefer SSOT for resolve_typebox_by_name everywhere; generate tables from specs at build time (static fallback remains)
- Plugins: keep plugin‑on reps order‑agnostic; strict reps preflight build via plugin‑tester; keep SKIP on missing artifacts
- Docs/Smokes: selfhost opt‑in env (SMOKES_SELFHOST_ENABLE / SMOKES_SELFHOST_M2M3_ENABLE) documented; CI defaults remain quick+integration‑core


## ✅ P4/P5 小結（2025‑10‑12）
- P4（薄アダプタ直結）
  - emit_mir_flow(_map): extern/global/method/constructor を BlockBuilder 直結の薄アダプタに統一（出力互換）
  - emit_call/emit_method/emit_newbox（v0/v1）: 内部を shared BlockBuilder 経由に寄せ、材化は LocalSSA に委譲
  - 代表（rc-only）: selfhost_pipeline_v2_p4_calls_rc_vm（Constructor/Method/Global/Extern の最小E2E）
- P5（LocalSSA 最小）
  - ensure_materialize_last_ret(mod): ret 値の定義直後に copy を 1 本
  - ensure_cond(mod): 全ブロック走査で branch の cond を検出→定義直後に copy を 1 本（If/Loop 両対応）
  - 代表（rc-only）: selfhost_localssa_ensure_calls_rc_vm / _ensure_cond_rc_vm / _ensure_cond_if_loop_rc_vm
- Pipeline（v1 経路）
  - MirCallBox 依存を Emit*（BlockBuilder 直結）に段階置換（差分最小）
- plugin‑on の安定化
  - quick の plugin_on_* を在庫プリチェック＋preflight 失敗時 SKIP へ統一（常緑を維持）
- SSOT（type/slot/arity/aliases）
  - specs/type_registry.toml を再スキャンし不足なしを確認。resolve_* は SSOT 優先（静的 fallback）で維持
- 状態: quick 288/288 PASS、integration‑core 20/20 PASS（環境依存は SKIP）

## 🧭 Backend と CLI 方針（単一 CLI, 複数バックエンド）
- 単一 CLI: `hakorune` に集約し、`--backend {nyvm|rust|llvm}`（`HAKO_BACKEND`）で切替
- バックエンド実体:
  - `hakorune-vm`（Ny 製 Mini‑VM）: MIR(JSON v0) 直実行、自己ホスト検証用
  - `hakorune-rust`（Rust VM/Runtime）: 既定の実用ライン（動的プラグイン/完全実行）
  - `hakorune-llvm`（LLVM ライン）: llvmlite ハーネス / AOT（将来 `hako-llvmc`）
- コンパイラーは分離: `hakorune-compiler`（selfhost/pipeline_v2）。dev ではバンドル起動を opt‑in で提供
- 互換: `NYASH_*` は `HAKO_*` のエイリアス受理（docs 表記は HAKO を第一）

## 🔧 ツール解決ポリシー（CLI→ツールのパス解決）
- 優先順: dist/bin → workspace（tools/*, target/*）→ hako.toml [tools]/[backends] → ~/.config/hakorune/config.toml → ENV（HAKO_TOOL_PATHS, 個別変数）→ PATH →（任意）autobuild
- オプション/ENV:
  - `--llvm-harness`/`HAKO_LLVM_HARNESS`, `--plugin-tester`/`HAKO_PLUGIN_TESTER`, `HAKO_TOOL_PATHS`, `--autobuild-tools`
  - 診断: `hakorune --which <tool>`（最終解決パス表示）, `--doctor tools`（在庫レポート）
- plugin‑on の代表は在庫なしで SKIP（前検あり）に統一

## 🛣 次の段階（hakorune‑vm を Rust VM 同等へ）
- Phase‑A（安定化小結）
  - JSON MIR v0 Reader の CLI 昇格（dev ゲート撤去）
  - LocalSSA: ensure_after_phis_copy の代表を 1 本追加（If/Loop）
- Phase‑B（機能差分の吸収）
  - Stage‑3: break/continue/throw/try の最小実装（自己ホスト側 emit→VM）
  - Map plugin: 値の handle（PluginHandle/HostHandle）対応と identity スモーク
- Phase‑C（仕上げ）
  - SSOT を型解決ホットパスに全面適用（静的は保険）
  - 診断の直書き掃除（helpers 統一）
- 受け入れ基準（同等の目安）
  - quick 288/288 緑（在庫依存は SKIP）
  - integration‑core 20/20 緑
  - selfhost M2/M3（opt‑in）代表緑
  - plugin_on_*（在庫あり）代表 PASS、診断文言は安定


## 🧱 nyvm の実体と Mini‑VM 凍結の方針（2025‑10‑12）
- nyvm の実体
  - CLI `--backend nyvm` は Hakorune VM（selfhost/hakorune‑vm/*）にマップする方針。
  - Mini‑VM（selfhost/vm/*）は dev/教育用のサンドボックスとして温存（rc‑only/opt‑in）。
- Mini‑VM の扱い
  - selfhost/vm に DEPRECATED マーカーを追加（新機能追加を抑制）。
  - スモークは rc‑only とし、既定は OFF。Hakorune VM の成長を阻害しない。
- 受け入れ（切替の前段）
  - quick/integration 緑維持、selfhost M2/M3 緑。
  - plugin‑on（在庫あり）代表 PASS。診断は helpers 統一。


## 【2025-10-12 追記】Mir IO 一本化と HostBridge 設計

- MirIoBox（Phase A）
  - Hako 側に `MirIoBox` を追加。`validate/functions/blocks/instructions/terminator` の最小 API を提供。
  - nyvm ブリッジが function オブジェクトのみを渡す narrow JSON も許容（bridge 実用性優先）。
  - VM コアは run() 先頭で `MirIoBox.validate` を呼び構造エラーを Fail‑Fast。
  - スモーク: `terminator_whitespace_vm`（op 後空白差）、`entry_nonzero_vm`（entry≠0 開始）を quick-selfhost に追加。

- HostBridge（設計）
  - 目的: プラグイン（動的）と埋め込み（静的）を同じ ABI で呼び出す一本化レイヤ。
  - 方針: `HostBridgeBox.{box_new,box_call,extern_call}` の3関数に最小化。TLV で引数/戻り値を統一。
  - 解決: `UnifiedRegistry` にて Spec（hako_box.toml or 生成SSOT）と Invoker（PluginLoaderV2/Provider）を集約。
  - ルート: `HAKO_PLUGIN_POLICY=off|auto|force`（auto=プラグイン優先→無ければ静的）。
  - 最初の疎通: FileBox.open/read（両経路で同形に動作）。

- 次の実装
  - run.sh の backend 正規化（空/"."/未知→vm）でスモークからノイズを根治。
  - MirIoBox.validate に terminator 必須・参照妥当性の検証を集約（dev 緩和は ENV）。
