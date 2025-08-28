# Phase 10.10 – Python→Nyash→MIR→VM/Native ラインの実用化整備（Box-First 継続）

目的: Nyash→MIR→VM/Native の実行ラインを日常運用レベルに引き上げ、GC/デバッグ/HostCallの柱を整備する。

## ゴール（DoD）
- エンドツーエンド実行ライン（Parser→AST→MIR→VM→JIT）がビルトインBoxで安定（RO/一部WO）
- GC切替（Null/Counting）をCLI/Boxから操作可能、root領域APIが一箇所化
- デバッグ/可視化の旗振り（DebugConfig/Box）でJIT/VM/イベント/DOTを一本化
- HostCall: 読み取り系はparam受けでJIT直実行（allow）。書き込み系はポリシー/whitelistでopt-in可能
- 最小ベンチと回帰（サンプル）でラインの劣化を検知

## 事前整備（現状）
- HostCall基盤: Registry/Policy/Events/Boundary（10.9-β/δ完了）
- JITイベント: `NYASH_JIT_EVENTS=1` 時に `threshold=1` 自動適用でLower確実実行
- 戻り境界: CallBoundaryBox で JitValue→VMValue を一元化（ハンドル復元含む）

## ワークストリーム
1) GC Switchable Runtime（phase_10_4_gc_switchable_runtime.md）
   - 目標: NullGc/CountingGc の切替、root領域/バリアAPIの一本化
   - タスク:
     - NyashRuntimeBuilder: GC選択をCLI/Box反映（NYASH_GC=none|counting など）
     - ScopeTracker/enter_root_region()/pin_roots() の公開インターフェース確認
     - CountingGcの統計出力（roots/reads/writes/safepoints）
     - 書き込み系HostCallにバリアサイトのフック（Map/Array set/push）
   - 受入: GC切替コマンドで統計差分が取れる／HostCall書き込みでバリアサイトが加算される

2) Unified Debug System（phase_10_8_unified_debug_system.md）
   - 目標: デバッグ/観測フラグを DebugConfig/Box に統合（CLI/env/Boxの単一路）
   - タスク:
     - DebugConfig（Rust側）: dump/events/stats/dot/phi_min 等を集約
     - DebugConfigBox: Boxから get/set/apply/toJson/fromJson
     - Runner: CLI→DebugConfig→env/Box の一本化（env直読み排除）
     - イベント出力先: stdout/file 切替の設定（NYASH_JIT_EVENTS_PATH のBox反映）
   - 受入: Boxから apply 後、JIT/VM/DOTの挙動が即時反映／JSONLが指定先に出力

3) E2Eラインの実用化（builtin→pluginの足場）
   - 目標: ビルトインBoxで日常運用レベル、プラグインBoxはTLV HostCallの足場を準備
   - タスク:
     - Lowerカバレッジの整理（BoxCall/RO/WO・param/非paramの分岐ダンプ）
     - 署名管理: レジストリのオーバーロード運用方針（canonical idと派生idの整理）
     - 返り型推論: MIR Builderのreturn_type推定を確認（main/補助関数とも）
     - Plugin PoC: TLV/handle経由のread-onlyメソッド1つをHostCall経由で通す（allowログまで）
   - 受入: 代表サンプル（math/map/array/string）でallow/fallbackが意図通り、plugin PoCでallowイベントが出る

4) ドキュメントと例の整理
   - 目標: 例の最小集合化（param/非param/RO/WO/HH/Hの代表）、手順の簡潔化
   - タスク:
     - examples/: 重複の削減、README（実行コマンド付き）
     - phase_10_9/10_10 のガイドをCURRENT_TASKと相互参照
   - 受入: 主要ケースが examples/README からそのまま実行可

5) ベンチと回帰（最小）
   - 目標: ラインの性能/退行の早期検知
   - タスク:
     - ny_bench.nyash のケース整理（関数呼出/Map set-get/branch）
     - compare: VM vs JIT（ウォームアップ付き）
   - 受入: ベンチ出力に JIT/VM の比較が出る（改善/退行が見える）

## リスクと対策
- param/非param 分岐の混乱: イベントに reason を必ず出す／docsにベストプラクティス（受け手をparam化）
- mutatingの誤許可: JitPolicyBox の whitelist/プリセットのみで許可、既定はread_only
- 署名の散逸: canonical id（例: nyash.map.get_h）と派生（_hh）の方針を明示

## 受け入れ基準（サマリ）
- DebugConfig/Box/CLIの一貫挙動（apply後の即時反映）
- GC切替とバリアサイト観測が可能
- HostCall（RO/一部WO）が param でallow、非paramでfallback（イベントで確認可）
- 代表サンプルが examples/README の手順で成功

## すぐ試せるコマンド（抜粋）
```bash
# math.min（関数スタイル）
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_NATIVE_F64=1 NYASH_JIT_EVENTS=1 \
  ./target/release/nyash --backend vm examples/jit_math_function_style_min_float.nyash

# Map.get HH直実行
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
  ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash

# Mutating opt-in（Array.push）
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
  ./target/release/nyash --backend vm examples/jit_policy_optin_mutating.nyash
```

