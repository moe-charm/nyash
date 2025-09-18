# Phase 12: ユーザー箱とプラグイン箱の境界撤廃 + Nyash ABI 導入計画（下準備）

目的
- 境界撤廃: ユーザー箱／プラグイン箱／内蔵箱を「TypeBox + Instance」に統一。
- Nyash ABI: vtable（slot優先）で高速・型安全に呼び出し、未対応は現行C ABI(TLV)へフォールバック。
- 綺麗な箱化: VM/JIT/LLVM/WASMが同一ディスパッチ構造を共有できる形へ段階導入。
- 同一実行: Nyashコードが VM と JIT で「意味・結果・副作用」が一致（同値性がテストで担保）。

非スコープ（当面）
- 既存C ABIの破壊的変更。段階導入のためC ABIは維持（fallback専用）。
- 直ちに全プラグインを移行すること（まずは1プラグインでPoC）。

設計の要点（サマリ）
- TypeBox（静的メタ）: 型名、メソッドslot→関数ポインタ、属性（可変/不変など）。
- Instance（実体）: type_id + 実体ハンドル（ユーザー/プラグイン/内蔵いずれも統一の箱）。
- VMディスパッチ: method_id/slotがあれば vtable 直呼び。なければ name→slot 解決→vtable／PIC→C ABI。
- JIT: まずはホストコールthunkでTypeBox vtable呼び出し→後続でモノモーフィック直埋め最適化。
- GC/Barrier: BoxCall(setField)/ArraySetでWrite Barrier発火（既存fast-path維持）。

トグル（管理棟に集約予定）
- NYASH_ABI_VTABLE=1 … vtable経路を有効化（既定OFF）
- NYASH_ABI_STRICT=1 … vtable未登録メソッド時にC ABIフォールバック禁止（実験）

段階導入（Tier）
1) Tier-0（雛形）
   - 追加: `src/runtime/type_box_abi.rs`（NyrtValue/TypeBox/関数ポインタ型、最小API）
   - 追加: `src/runtime/type_registry.rs`（TypeId→TypeBox参照）
   - VM: `execute_boxcall` に vtable 優先のstubを追加（`NYASH_ABI_VTABLE=1`時のみ）
   - Docs/CI: 仕様追記・スモーク追加準備（まだvtableは未実装でもOK）
2) Tier-1（実証）
   - 1プラグイン（例: MapBox.getS）を Nyash ABI で動作させる（VM→vtable→関数ポインタ）
   - JIT: vtable呼び出しthunk追加（VM側レジストリから関数ポインタ取得）
   - テスト: C ABI とNyash ABIの同等性（差分テスト）
3) Tier-2（拡張）
   - 内蔵箱のTypeBox化（Array/String/Mapの主要メソッド）
   - ユーザー箱（InstanceBox）をTypeBox上に還元（Rust関数を関数ポインタで包む）
   - name→slot化の徹底（slot_registry と連携強化）
4) Tier-3（セルフホスティング準備）
   - Nyash ABI のC実装を開始（Rust⇔C シム）
   - JIT: モノモーフィックサイトで vtable_slot 直呼び最適化

完了条件（Phase 12の下準備）
- PLAN/TASKS/TECHNICAL_DECISIONS に統一方針と段階計画が明記されている。
- `NYASH_ABI_VTABLE` トグルの導入方針が定義済み（まだコードは雛形で可）。
- VM側にvtable優先呼び出しstubの追加計画が固まり、レジストリ/TypeBox ABIの最小構成が決まっている。
- Cross-backend同値テストの設計方針が固まっている（VM/JITを同条件で走らせ、結果/ログ/副作用を比較する仕組み）。

次アクション（このPR/コミット範囲外の実装）
- TypeBox ABIモジュール雛形の追加、VM vtable stub の実装（既定OFF）。
- MapBoxで最小PoC（getS）を先行導入（C ABIと同じ結果を返す）。
- docs/TASKSのTier-0チェックを更新、CIスモークの草案を追加。

検証（同一実行）の方針
- テストハーネス: 同一 Nyash プログラムを VM と JIT で実行し、以下を比較
  - 戻り値の等価（NyashValue同値: int/float/bool/string/null/void）
  - Box状態の差分（対象: Map/Array/Instance の代表ケース）
  - ログイベントの正規化比較（必要最小限）
- スコープ: array/field/arithmetic/extern_call を最小セットにし、段階的に拡張
- CI連携: `consistency` ラベルのテストジョブを追加（将来）

参考ドキュメント
- Nyash ABI Minimal Coreと進化戦略: `docs/reference/abi/NYASH_ABI_MIN_CORE.md`
