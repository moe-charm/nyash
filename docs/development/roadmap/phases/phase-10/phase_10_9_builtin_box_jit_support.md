# Phase 10.9 – Builtin-Box JIT Support（Box-First Plan）

目的: Nyashスクリプト → VM（基準）→ JIT（段階導入）。
まずは「読み取り系」をJITで安全に通し、箱で問題点を包んで順に拡張する。

## 🎯 ゴール（DoD）
- 機能: String/Array/Map の読み取りAPIが JIT 経路で VM と一致
  - length/isEmpty/charCodeAt, Array.get, Map.size/has
- 境界: 署名不一致・未対応は VM フォールバック（理由はイベントJSONに記録）
- 箱: Policy/Events/Registry を 1 箇所参照（切替点の一本化）
- 観測: JSONL イベントが最小1件以上出力（オプトイン）

## 🧱 先に積む箱（最小）
- JitConfigBox（設定）
  - exec/stats/dump/phi_min/hostcall/native_f64/native_bool/threshold を `apply()` でenv反映
  - `toJson()/fromJson()/summary()` で可視化
- JitPolicyBox（ポリシー）
  - read_only/hostcall_whitelist。書込系は既定で拒否（jit-direct等の安全弁）
- JitEventsBox（観測）
  - compile/execute/fallback/trap を 1行JSON（標準出力 or ファイル）で記録
- HostcallRegistryBox（レジストリ）
  - 許可HostCallと args/ret 署名（唯一の切替点）。不一致は `sig_mismatch`
- FrameSlotsBox（スロット）
  - ptr→slot の割付と型注釈（v0は i64 のみ）
- CallBoundaryBox（境界）
  - JIT↔VM の薄い呼出し点（型変換の一本化）。将来多関数へ拡張

最小原則: 箱を先に置く（no-op/ログでもOK）→ 切替点を1箇所に固定 → その箱の内部を順に強化。

## 🛣️ 実行経路の設計（概要）
1) Runner: CLI/env→`JitConfig`→TLSへ反映（env直読を排除）
2) LowerCore: `jit::config::current()` を参照し、BoxCall/Load/Store/Branch/PHIを最小下ろし
3) HostCall: Handle経由で read-only を通す（mutating は Policy で拒否）
4) Fallback: 未対応/署名不一致/ポリシー違反は VM 実行へ委譲
5) Events: `JitEventsBox` 経由で allow/fallback/trap を JSONL 出力

## 🔢 対象API（v0）
- ArrayBox: `length`, `get`, `isEmpty`, `push/set`（mutatingは拒否）
- MapBox: `size`, `has`, `get`, `set`（mutatingは拒否）
- StringBox: `length`, `isEmpty`, `charCodeAt`
- Math（薄接続）: `sin/cos/abs/min/max`（署名一致のみ allow を記録、実体はVMへ）

## 🗺️ マイルストーン
### 10.9-α（足場）
- JitPolicyBox v0: read-only/whitelist を箱へ移動
- JitEventsBox v0: compile/execute の JSONL イベント（オプトイン）
- ドキュメント: 再起動チェック/箱の役割を追記

### 10.9-β（読み取りカバレッジ）
- HostcallRegistryBox v0: String/Array/Map 読み取り API の登録・署名検査
- LowerCore: BoxCall read-only 経路を Registry/Policy 参照に切替
- E2E: `length/isEmpty/charCodeAt/get/size/has` の一致（jit-direct + VM）

### 10.9-γ（生成の足場）
- CallBoundaryBox v0: JIT→VMで `new` 等を委譲（薄い箱）
- `new StringBox/IntegerBox/ArrayBox` の最小経路（方針次第で jit-direct は拒否）

### 10.9-δ（書き込みの導線のみ）
- JitPolicyBox: 書込許可スイッチ（既定OFF）
- LowerCore: 書込命令は Policy 参照で拒否/委譲/許可（1箇所で判断）

## ✅ すぐ使えるチェック
- ビルド
  - `cargo build --release --features cranelift-jit`
- 主要フラグ
  - `NYASH_JIT_EXEC=1` `NYASH_JIT_THRESHOLD=1`
  - `NYASH_JIT_EVENTS=1`（標準出力へJSON）
  - 任意: `NYASH_JIT_EVENTS_PATH=target/nyash/jit-events.jsonl`
- 代表サンプル（VM経由でJITパス通過）
  - 成功: `./target/release/nyash --backend vm examples/jit_hostcall_len_string.nyash`
  - 失敗: `NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm examples/jit_hostcall_array_append.nyash`
  - 境界: `NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm examples/jit_hostcall_math_sin_mismatch.nyash`
  - 署名一致(allow観測): `NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm examples/jit_hostcall_math_sin_allow_float.nyash`
  - 関数スタイル(math.*): `NYASH_JIT_NATIVE_F64=1 NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm examples/jit_math_function_style_sin_float.nyash`
    - `cos/abs/min/max` も同様のサンプルあり
- 詰まったら
  - `--features cranelift-jit` が付いているか
  - イベントJSONに `fallback/trap` の理由が出ているか
  - `cargo clean -p nyash-rust` → 再ビルド

## 🧪 検証と観測
- 統合JIT統計（テキスト/JSON）: sites/compiled/hits/exec_ok/trap/fallback_rate/handles
- `JitStatsBox.perFunction()` で関数単位の統計（JSON配列）
- CFG/PHIダンプ: `NYASH_JIT_DUMP=1`、`NYASH_JIT_DOT=path.dot`（最小）
- b1正規化カウンタ: `b1_norm_count`（分岐条件/PHI）
- HostCallイベント: `argc`/`arg_types`/`reason`でデバッグ容易化（mutatingは `policy_denied_mutating`）

## ⚠️ リスクとその箱での緩和
- 署名不一致（args/ret）
  - HostcallRegistryBox で一元検査。不一致は `sig_mismatch` でイベント記録→VMへ
- mutatingの混入
  - JitPolicyBox.read_only で抑止。Registryの Mutating 分類と併用
- 型崩れ/ABIの揺れ
  - `JitValue`（I64/F64/Bool/Handle）へ統一、変換は境界1箇所
- 観測不足
  - JitEventsBox の粒度を最小から用意（必要に応じ拡張）

## 🔧 実装ノート（現状）
- Config: Rust側 `jit::config::JitConfig` に集約。Nyash側は JitConfigBox で操作
- LowerCore: BoxCallの read-only は Registry/Policyに委譲。math.* は署名一致なら allow を記録（実行はVM）
- Handle: `rt::handles` による u64→Arc<Box>。JIT↔ホストをVM型非参照で独立
- 数値緩和: `NYASH_JIT_HOSTCALL_RELAX_NUMERIC=1` で i64→f64 コアーションを許容（既定は `native_f64=1` 時に有効）。`JitConfigBox.set_flag("relax_numeric", true)` でも切替可能

## 📌 次の拡張（10.9の後）
- f64ネイティブ最小経路（`NYASH_JIT_NATIVE_F64=1`）の拡充
- Boolネイティブ（b1）署名サポート（ツールチェーンcapに連動）
- HostCallブリッジの拡大（Map.getの多型キー、String操作の追加）
- CallBoundaryBox経由の `new`/副作用命令の段階的JIT化

---

最短ルート: 箱（Policy/Events/Registry/Boundary）を先に置き、読み取り系でJITを安全に通す→観測を増やす→署名とポリシーの一本化で切替点を固定→必要最小限のネイティブ型（f64/b1）を段階導入。
