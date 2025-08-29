
# Box-First JIT: Decoupled, Probe-Driven JIT Enablement in Nyash within 24 Hours (WIP)

## 1. Introduction
JIT を安全に導入しつつ短時間で価値を出すには、実装より先に「戻せる足場」を設計する必要がある。本稿は Nyash において、箱理論（Box-First）— 設定・境界・観測をまず「箱」に封じ込める — を適用し、24時間で分岐/PHI/f64 の JIT 経路と可視化・統計・回帰検出を着地させた実務手法を報告する。

貢献は以下である。
- Box-First の JIT適用法則（設定/境界/可視化/可逆性の箱化）
- 設定箱 JitConfigBox + runtime capability probe + current()参照の実装
- JIT⇔VM 疎結合（JitValue/HandleRegistry/フォールバック）の設計
- 型付きシグネチャの足場（i64/f64/bool混在、b1 は内部→将来ワンフリップ切替）
- CFG/PHI の DOT 可視化（cond:b1/phi:b1）、統合 JIT 統計

## 2. Box-First Design
### 2.1 Configuration as a Box
- JitConfigBox で env 直読みを排し、`apply()` で env と `jit::config::set_current()` を同時反映。
- 起動時に `probe_capabilities()`（例: `supports_b1_sig=false`）を適用して、`native_bool_abi` を自動調整。
- ホットパスは常に `jit::config::current()` を参照（テスト・CLI・箱の三者を束ねる）。

### 2.2 Boundary as a Box
- JIT/VM 境界は `JitValue` と `adapter`、Host 側は `HandleRegistry(u64↔Arc)` に集約。
- VMValue は境界一箇所でのみ変換。panic は `catch_unwind→VM` へフォールバック。

### 2.3 Typed ABI Scaffold
- `prepare_signature_typed(params, ret_is_f64)` に一本化された型決定。
- `ParamKind::B1` は現状 I64(0/1) にマップ（将来 `types::B1` に一行切替）。
- 戻り F64 はビルダ状態 `desired_ret_is_f64` に紐づけてトランポリンを選択（env依存を排除）。

### 2.4 b1 Internal Path (Conservative)
- ABI は I64(0/1) のまま、内部で `push_block_param_b1_at` により b1 正規化（icmp!=0）。
- LowerCore は Compare の dst を bool として記録し、全入力が bool の PHI を bool-phi として b1 で取得。

### 2.5 Observability & Reversibility
- `NYASH_JIT_DUMP/…_JSON` と `NYASH_JIT_DOT`。
- DOT に `cond:b1` と `phi:b1` を表示。統合 JIT 統計に exec_ok/trap/fallback_rate。
- すべての切替は Box/flag/feature によって可逆で、フォールバック経路が常に通る。

## 3. Implementation Overview
- `src/jit/config.rs`: current()/set_current()/probe_capabilities()/apply_runtime_caps()
- `src/boxes/jit_config_box.rs`: env排除・箱中心、`from_runtime_probe()` でcapability反映
- `src/jit/lower/{core,builder}.rs`: 型付きシグネチャ・b1内部API・BinOp/Compareの昇格
- `src/jit/engine.rs`: dump/統計・DOT 出力接続

## 4. Evaluation (Early)
- 正当性（VM vs JIT）：
  - branch_return, jit_phi_demo, jit_f64_arith で一致（f64 は `NYASH_JIT_NATIVE_F64=1`）。
- 性能（compileコスト込み）：
  - arith_loop_100k: JIT ≈ 1.40× VM、f64_add: JIT ≈ 1.06× VM、branch_return: ≈VM 同等。
- 観測：fallback率・handle数・cfg/phi可視化により回帰確認が容易。

## 5. Related Work
- LuaJIT/HotSpot のプロファイル駆動設計、Cranelift/Wasmtime の安全分離設計。
- Box-First は DI/境界設計に近似するが、Box（共有できる実体）で統一し、開発・回帰・視覚化の軸まで包含する点が異なる。

## 6. Limitations & Future Work
- B1 ABI は現行ツールチェーン非対応：`supports_b1_sig=true` 確認後、ParamKind::B1→types::B1 に一行切替。
- f64/bool の完全ネイティブ化には、混在署名の網羅とトランポリン形状整理が必要。
- 回帰強化：小さなゴールデン（b1合流、昇格混在、フォールバック異常系）と stats.jsonl（abi_mode）を追加。

## 7. Conclusion
箱理論（Box-First）により、実装前に戻せる足場を先に固定し、AI支援下でも力づく最適化に頼らず、可視・可逆・切替可能なJITを24時間で実用化した。設定・境界・観測の箱化、typed ABI の一本化、b1内部パス、DOT/統計の可視化は、段階的最適化を安定に進める強力な作法である。
