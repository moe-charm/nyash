# PHI処理設計ドキュメント（箱理論実装）

## 📖 概要

このドキュメントは、LLVM Python バックエンドにおけるPHI命令処理の設計決定、現状の問題点、改善方針を記載します。

**更新日**: 2025-10-01
**担当**: Claude Code + ChatGPT Pro 協働分析

---

## 🎯 設計原則: 箱理論（Box-First）

### 箱理論の実践
PHI命令処理は「箱理論」に基づいて実装されています：

1. **「箱にする」**: PHI処理を専用モジュール（PhiHandler）に分離
2. **「境界を作る」**: block layer（block_lower.py）とinstruction layer（instruction_lower.py）の責任を明確化
3. **「戻せる」**: 従来の処理フローも維持可能（デバッグ・検証用）
4. **「見える化」**: PHI処理の流れが明確（verbose mode対応）

---

## 🏗️ アーキテクチャ

### 現在の実装（wasm-development）

```
block_lower.py (Block Layer)
  ├─ PhiHandler.collect_phi_instructions()  # PHI命令分離
  ├─ PhiHandler.process_phi_instructions()  # PHI命令処理（ブロック先頭）
  └─ builder.lower_instruction()            # 非PHI命令処理

instruction_lower.py (Instruction Layer)
  ├─ lower_phi()  ← ⚠️ デッドコード（到達不能）
  ├─ lower_binop(), lower_compare(), ...
  └─ InstructionContext ← 🔧 PHI専用、未使用
```

### PHI処理の流れ

```python
# 1. block_lower.py で命令を分離
phi_ops, non_phi_insts = phi_handler.collect_phi_instructions(insts)

# 2. PHI命令を先頭で処理
phi_handler.process_phi_instructions(phi_ops, bb, func)

# 3. 非PHI命令を処理
for inst in body_ops:
    builder.lower_instruction(ib, inst, func)
    # ← PHI命令はここに到達しない（既に分離済み）
```

---

## 🚨 現状の問題点（ChatGPT Pro 分析結果）

### 1. PHI処理の二重経路問題

**問題**:
- `block_lower.py` でPHI命令を処理しているため、`instruction_lower.py:81-100` のPHI処理コードは**到達不能（デッドコード）**

**影響**:
- コード重複によるメンテナンス負荷
- 将来的なバグの温床（二重処理の可能性）

**対策**:
- instruction_lower.pyのPHI処理コードを削除（PhiHandlerに一本化）

---

### 2. Silent Failure（エラー握りつぶし）

**問題**:
```python
# block_lower.py:166-169
except Exception as e:
    trace_debug(f"[llvm-py] PHI processing error: {e}")
    import traceback
    traceback.print_exc()
    # ← ここで処理続行（エラーを握りつぶし）
```

**影響**:
- PHI処理失敗時にビルドが継続してしまう
- 不完全なLLVM IRが生成され、後続でクラッシュ

**対策**:
- フェイルファスト化：PHI処理失敗時は明確に例外を投げてビルド停止
- 環境変数でフォールバックモードを明示的に選択可能にする

---

### 3. vmap二重登録

**問題**:
```python
# phi_handler.py: 両方に登録
self.builder.vmap[dst] = phi
if hasattr(self.builder, '_current_vmap'):
    self.builder._current_vmap[dst] = phi
```

**検討事項**:
- `_current_vmap` はブロックローカルなSSA値追跡用
- `vmap` はグローバルな値マップ
- 両方への登録が本当に必要かを再検証

**対策**:
- ブロック終了時のスナップショット解決で十分か検証
- 不要な場合は片方への登録に統一

---

### 4. InstructionContext の活用不足

**問題**:
```python
# instruction_lower.py:42-44
inst_ctx = InstructionContext.from_owner(owner, builder, builder.block)
# ← 生成後、使われていない（lower_phi に渡していない）
```

**現状**:
- InstructionContext はPHI専用に設計されているが、実際には未使用

**方針**:
- 現状はPhiHandler方式で十分
- 将来的に複雑な命令（binop/compare）に箱化を拡張する際に活用

---

## ✅ 実装済み機能

### 1. ループPHI対応
- ✅ 自己参照を含むループヘッダのPHI命令に対応
- ✅ predecessorスナップショット解決による正確なincoming接続

### 2. 箱理論の実践
- ✅ PhiHandler による責任分離
- ✅ verbose mode によるデバッグ可視化
- ✅ ブロックレイヤでのPHI先頭処理

### 3. SSA値管理
- ✅ ブロックローカルvmap（`_current_vmap`）
- ✅ グローバルvmap（`vmap`）
- ✅ block_end_values スナップショット

---

## 🔧 改善方針（Phase 3.1+）

### 優先度A（緊急）

#### A1. デッドコード削除
- **対象**: `instruction_lower.py:81-100` のPHI処理コード
- **理由**: 到達不能コード、メンテナンス負荷
- **作業**: PhiHandler方式への統一

#### A2. フェイルファスト化
- **対象**: `block_lower.py:166-169` のエラーハンドリング
- **変更前**:
  ```python
  except Exception as e:
      trace_debug(f"[llvm-py] PHI processing error: {e}")
      # 処理続行
  ```
- **変更後**:
  ```python
  except Exception as e:
      if os.environ.get('NYASH_LLVM_PHI_LENIENT') == '1':
          trace_debug(f"[llvm-py] PHI processing error (lenient): {e}")
      else:
          raise RuntimeError(f"PHI processing failed: {e}") from e
  ```

### 優先度B（重要）

#### B1. vmap二重登録の検証
- **検証項目**:
  1. `_current_vmap` のみで十分か？
  2. グローバル `vmap` への登録タイミングは適切か？
  3. スナップショット解決との整合性

#### B2. InstructionContext の適用範囲見極め
- **現状**: PHI専用、未使用
- **検討**: binop/compare等への拡張が本当に必要か？
- **方針**: パフォーマンスと複雑さのバランスを取る

---

## 📊 テスト状況

### 実装済みテスト
- ✅ `test_phi_if.json` - if文PHI合流
- ✅ `test_phi_simple.json` - 基本PHI
- ✅ `test_control_flow_smoke.json` - 制御フロー統合

### 今後追加予定
- [ ] ループPHI専用テスト（自己参照確認）
- [ ] 複雑制御フロー（ネストif、多段ループ）
- [ ] エラーハンドリングテスト（フェイルファスト確認）

---

## 🔍 デバッグ環境変数

### 現在サポート
```bash
# PHI処理詳細ログ
NYASH_PHI_VERBOSE=1

# 厳格モード（synthesized zero検出時にエラー）
NYASH_LLVM_PHI_STRICT=1
```

### 追加予定
```bash
# フォールバックモード（エラー時に処理継続）
NYASH_LLVM_PHI_LENIENT=1
```

---

## 📚 参考リソース

- **PhiHandler実装**: `src/llvm_py/builders/phi_handler.py` (197行)
- **InstructionContext**: `src/llvm_py/builders/instruction_context.py` (98行)
- **箱理論ガイド**: `CLAUDE.md` - "箱理論（Box-First）"セクション
- **Phase 3.1完了報告**: `CURRENT_TASK_WASM.md`

---

## 🎯 まとめ

**現状**: PhiHandler方式で箱理論を実践、基本機能は完全動作
**問題**: 二重経路、silent failure、vmap二重登録
**方針**: デッドコード削除→フェイルファスト化→vmap最適化

---

## Phase 15.5 実装ステータス（LLVM Harness Line）

- 単一起点PHI: `PhiRegistry` で (block,dst) ごとに 1 個に統一
- finalize は wire-only: 新規PHIは作らず incoming 配線のみ（`NYASH_LLVM_PHI_ALLOW_CREATE=1` でのみ許可）
- 事前占位: 関数降下の早い段階で `PhiLifecycle.create_phase()` により宣言 (block,dst) の PHI をブロック先頭に生成
- 比較/分岐の安定化:
  - compare は Resolver を一次経路とし、宣言PHIまたは直近 add を最小フォールバックとして採用
  - branch はブロック内の copy 連鎖（aliases）を先に正規化してから厳格解決

検証
- `NYASH_LLVM_PHI_VERIFY(_STRICT)` で Fail‑Fast（order/uniqueness/cfg）
- `NYASH_LLVM_SANITIZE_EMPTY_PHI=1`（既定ON）で空PHIを除去、PHI のブロック先頭集約を補助

---

## PhiDispatchPoint（箱）導入計画

ねらい
- compare/branch/binop に分散しがちな値解決フォールバックを 1 箱に統合し、責務を明確化・再利用性を高める。

MVP の API
```
class PhiDispatchPoint:
    def resolve_i64(builder, resolver, vid, current_block, preds, block_end_values, vmap, bb_map) -> i64:
        # 優先: Resolver（厳格）
        # 次点: 宣言PHI（block_phi_incomings）
        # さらに: 直近add（ループのインクリメントパターン）
        # 最後: i64正規化（ptr→i64, iN→i64）
```

段階導入
1) compare/branch を DispatchPoint 経由に置換（ヒューリスティック分散を除去）
2) binop も委譲（比較系経由での一貫性）
3) 将来: BlockAsLoop1（init/step/fini）で LoopForm IR を箱に封入

スモーク
- `tools/smokes/v2/run_phi.sh` が `apps/tests/phi_*.nyash` を自動発見し、VM vs LLVM の Result ライン一致を比較
- 代表: if/else 合流、ネストループ、continue/break、委譲（関数呼び出し）

ChatGPT Proの分析により、設計の強みと改善点が明確になりました。段階的に改善を進めます。

---

### PhiDispatchPoint — Box Spec（契約）

責務（Responsibility）
- 合流点での i64 値解決を一箇所に集約し、PHI/宣言/同ブロック alias/直近 add を扱う。
- compare/branch/binop からのフォールバックを吸収し、支配関係を壊さない形で i64 に正規化する。

入力（Inputs）
- `builder: IRBuilder`（IR 挿入位置）
- `resolver: Resolver`（dominance/型正規化の一次経路）
- `vid: int`（MIR の値ID）
- `current_block: ir.Block`
- `preds, block_end_values, vmap, bb_map`（CFG/値スナップショット/SSAマップ）

出力（Outputs）
- `ir.Value`（i64 正規化済み SSA。必要時に ptr→i64, iN→i64 を実施）

不変条件（Invariants）
- 宣言済み PHI があればそれを唯一のソースとして採用（`PhiRegistry` により (block,dst) で一意）。
- PHI は常にブロック先頭に集合（verify により order/uniqueness/cfg を担保）。
- DispatchPoint 自身は既定で PHI を新規生成しない（wire/参照のみ）。

Fail‑Fast / 環境（Env）
- `NYASH_LLVM_PHI_VERIFY=1`（既定ON）: order/uniqueness/cfg を検証。
- `NYASH_LLVM_PHI_VERIFY_STRICT=1` : 局所合成 PHI を禁止、宣言線のみに一本化。
- `NYASH_LLVM_SYNTH_LOCAL_PHI=1` : 実験用に限り、局所 PHI の合成を許可（既定OFF）。

非対象（Out of Scope）
- 文字列/ポインタの boxing・橋渡しの実体化（externcall/binop/ret が担当）。
- 純算術の最適化（LLVM 最適化段に委譲）。

---

## 追補（2025‑10‑02）— PHI Hardening 方針と実装

目的
- PHI は常に「ブロック先頭にグルーピング」されるLLVM不変条件を、構造で担保する。

変更点（要旨）
- 占位の強化: block_lower が body 降下前に `block_phi_incomings[bid]` の全 dst について `ensure_phi(builder, bid, dst, bb)` を呼び、PHIプレースホルダを先頭に作成。
- 局所合成の既定OFF: resolver によるローカルPHI合成は `NYASH_LLVM_SYNTH_LOCAL_PHI=1` の opt‑in のみ許可。通常は PhiHandler/ensure_phi で先頭に用意。
- 検証強化: `verify_phi_cfg` に加えて `verify_phi_order` を導入し、PHIが非PHIより後に現れないことを検証。`NYASH_LLVM_PHI_VERIFY_STRICT=1` でFail‑Fast。

実装参照
- `src/llvm_py/builders/block_lower.py` — ensure_phi 呼び出しの追加（body 前占位）
- `src/llvm_py/resolver.py` — ローカル合成PHIの既定OFF（環境変数でのみ許可）
- `src/llvm_py/phi_wiring/verify.py` — `verify_phi_order` 追加
- `src/llvm_py/builders/function_lower.py` — PHI順序検証の導入

運用
- bring‑up 時は `NYASH_LLVM_PHI_VERIFY=1`（既定ON）で軽量検証、`NYASH_LLVM_PHI_VERIFY_STRICT=1` でFail‑Fastに。
- どうしても必要な実験時のみ `NYASH_LLVM_SYNTH_LOCAL_PHI=1` を使用（既定はOFF）。

---

## 🌟 LoopForm IR原則の適用（UltraThink実装方針）

**更新日**: 2025-10-02
**理論的基盤**: `docs/private/papers-archive/paper-e-loop-signal-ir/main-paper-jp.md`

### 📖 背景: LoopSignal IR理論とPHI生成の融合

**LoopForm IRの核心**（20分のたばこ思考💨から生まれた統一直観）:
```
Everything is Box（空間） × Everything is Loop（時間）
= すべてが「Loop1の箱」
```

**PHI生成への適用価値**:
1. **合流点の定型化** - PHIはdispatch point（`loop.begin`）直後のみ
2. **Box=Loop1の統一** - すべてのブロックを `init→step→fini` として扱う
3. **制御の値化** - PHI値も"Signal"として統一的に解決

### 🎯 現在の実装とLoopForm原則の対応関係

#### **完璧な一致（既に実装済み！）**

| LoopForm IR | 現在の実装 | ファイル |
|------------|----------|---------|
| `loop.begin` | `PhiRegistry.ensure()` at block head | `phi_wiring/registry.py:67-95` |
| `loop.iter` | `instruction_lower()` 本体処理 | `builders/instruction_lower.py` |
| `loop.branch` | `lower_branch()` 条件分岐 | `instructions/controlflow/branch.py` |
| `loop.end` | `finalize_phis()` 配線完了 | `phi_wiring/wiring.py` |

**重要な発見**: ChatGPT実装は**LoopForm IR理論の自然な実現**になっている！

### 🚀 Loop1の統一: すべてのブロックを反復単位として扱う

#### **理論（main-paper-jp.md Line 42-46）**:
```
どの Box も init -> step -> fini を持つ
普通の箱は step() が 1回で Break(result) を返す（= Loop1）
```

#### **実装への翻訳**:

```python
# すべてのブロックは「Loop1」として扱える
class BlockAsLoop1:
    """LoopForm IR原則: Box=Loop1統一"""

    # init: ブロック先頭でPHI占位
    def init_phase(self, block_id: int, bb: ir.Block):
        """loop.begin 相当 - PHI先頭占位"""
        # PhiRegistry.ensure() で全PHIを先頭に作成
        for dst_vid in self.get_phi_dsts(block_id):
            ensure_phi(self.builder, block_id, dst_vid, bb)

    # step: 本体命令実行（1回）
    def step_phase(self, instructions: List[Dict]):
        """loop.iter 相当 - 命令降下"""
        for inst in instructions:
            self.builder.lower_instruction(inst)
        # → 普通のブロックは1回で Break(result)

    # fini: ブロック終了処理
    def fini_phase(self, block_id: int):
        """loop.end 相当 - 配線完了"""
        # finalize_phis() でPHI incoming配線
        self.finalize_block_phis(block_id)
```

### 📦 箱化提案: PhiDispatchPoint

#### **LoopForm原則（main-paper-jp.md Line 143）**:
```
ループ正規形: PHIはdispatch直後のみ
```

#### **実装: PhiDispatchBox**

```python
# src/llvm_py/builders/phi_dispatch.py (新規)

class PhiDispatchPoint:
    """
    LoopForm IR原則: dispatch合流点管理箱

    責務:
    1. ブロック先頭でのPHI占位（loop.begin）
    2. dispatch pointからの値解決（LoopSignal方式）
    3. 合流点の定型化保証

    箱理論:
    - 「箱にする」: dispatch合流点を専用箱に分離
    - 「境界を作る」: PHI生成/配線/解決を明確化
    - 「戻せる」: 従来のPhiHandler方式とも共存可能
    - 「見える化」: dispatch点の状態が明確
    """

    def __init__(self, builder, block_id: int, bb: ir.Block):
        self.builder = builder
        self.block_id = block_id
        self.bb = bb
        self.phi_map = {}  # dst_vid -> PHI instruction

    def ensure_phis_at_head(self):
        """
        loop.begin 相当: ブロック先頭でPHI占位

        LoopForm原則: すべてのPHIはdispatch point（ブロック先頭）に配置
        """
        from phi_wiring.registry import PhiRegistry

        # block_phi_incomings から必要なPHIを収集
        phi_incomings = self.builder.block_phi_incomings.get(self.block_id, {})

        for dst_vid in phi_incomings.keys():
            # PhiRegistryで単一起点保証
            phi = PhiRegistry.ensure(self.builder, self.block_id, dst_vid, self.bb)
            self.phi_map[dst_vid] = phi

    def resolve_from_dispatch(self, vid: int) -> Optional[ir.Value]:
        """
        dispatch合流点から値解決（LoopSignal方式）

        LoopForm原則: 制御を「値」として扱う
        → PHIも「値」として dispatch点から取得
        → ヒューリスティック不要！
        """
        # 1. dispatch点のPHIを優先
        if vid in self.phi_map:
            return self.phi_map[vid]

        # 2. PhiRegistryで確認（他のdispatch点で作成済みかも）
        from phi_wiring.registry import PhiRegistry
        phi = PhiRegistry.get(self.builder, self.block_id, vid)
        if phi is not None:
            return phi

        # 3. 見つからない場合のみ None（呼び出し側でフォールバック）
        return None
```

### 🔧 値解決のシンプル化: compare.pyヒューリスティック削除

#### **現状の問題**:
```python
# src/llvm_py/instructions/compare.py
# 値解決経路が5つもある！
1. resolve_i64_strict()           # メイン解決
2. vmap/global_vmapからPHI探索   # 106-125行
3. _phi_from_decl()                # 138-171行
4. _last_add_in_block()            # 173-194行 ← ヒューリスティック！
5. _phi_from_pred()                # 196-240行
```

#### **LoopForm原則による改善**:

```python
# src/llvm_py/instructions/compare.py (改善版)

def lower_compare(
    builder: ir.IRBuilder,
    op: str,
    lhs: int, rhs: int, dst: int,
    vmap: Dict[int, ir.Value],
    resolver=None,
    current_block=None,
    **kwargs
):
    """
    LoopForm原則適用版: dispatch point優先解決

    原則:
    - すべての値はdispatch point（ブロック先頭PHI）から取得
    - ヒューリスティック不要（PhiRegistryが保証）
    """
    i64 = ir.IntType(64)

    # 1. dispatch pointから解決（LoopForm原則）
    dispatch = getattr(builder, 'current_dispatch_point', None)
    if dispatch is not None:
        lhs_val = dispatch.resolve_from_dispatch(lhs)
        rhs_val = dispatch.resolve_from_dispatch(rhs)
        if lhs_val is not None and rhs_val is not None:
            # dispatch点から取得成功 → これで十分！
            pred = op if op in ('<','>','<=','>=','==','!=') else '=='
            cmp_result = builder.icmp_signed(pred, lhs_val, rhs_val, name=f"cmp_{dst}")
            vmap[dst] = cmp_result
            return

    # 2. フォールバック: Resolver経由（従来互換）
    lhs_val = resolve_i64_strict(resolver, lhs, current_block, ...)
    rhs_val = resolve_i64_strict(resolver, rhs, current_block, ...)

    # ヒューリスティック削除！
    # _last_add_in_block(), _phi_from_pred() は不要
    # → dispatch point優先で解決できる

    if lhs_val is None:
        lhs_val = ir.Constant(i64, 0)
    if rhs_val is None:
        rhs_val = ir.Constant(i64, 0)

    # 比較実行
    pred = op if op in ('<','>','<=','>=','==','!=') else '=='
    cmp_result = builder.icmp_signed(pred, lhs_val, rhs_val, name=f"cmp_{dst}")
    vmap[dst] = cmp_result
```

### 📋 実装ロードマップ（LoopForm原則段階適用）

#### **Phase 1: PhiDispatchPoint導入**（1-2日）
```bash
# 1. PhiDispatchPoint実装
src/llvm_py/builders/phi_dispatch.py  # 新規作成（100-150行）

# 2. block_lowerに統合
src/llvm_py/builders/block_lower.py   # dispatch point作成

# 3. テスト
apps/tests/loop_if_phi.nyash          # 既存テスト流用
```

**期待効果**:
- ✅ dispatch point集約（LoopForm原則準拠）
- ✅ PHI解決経路の単純化
- ✅ 箱理論の更なる実践

#### **Phase 2: 値解決シンプル化**（2-3日）
```bash
# 1. compare.py ヒューリスティック削除
src/llvm_py/instructions/compare.py  # 285行→150行（135行削減！）

# 2. binop.py, unop.py にも適用
src/llvm_py/instructions/binop.py
src/llvm_py/instructions/unop.py

# 3. スモークテスト
tools/smokes/curated_llvm.sh         # 全テスト通過確認
```

**期待効果**:
- ✅ コード行数 50%削減
- ✅ 保守性向上（ヒューリスティック削除）
- ✅ LoopForm原則の完全実現

#### **Phase 3: BlockAsLoop1統一**（実験的・将来）
```bash
# すべてのブロックをLoop1として扱う統一実装
# → generator/async への布石
```

### 🎯 検証項目（LoopForm原則適合性）

#### **構造的検証**:
- [ ] すべてのPHIがdispatch point（ブロック先頭）に配置されているか
- [ ] 値解決がdispatch point優先で行われているか
- [ ] ヒューリスティックが不要になっているか

#### **性能検証**:
- [ ] コンパイル時間（同等〜微差）
- [ ] 実行時間（同等〜微差）
- [ ] コード行数（50%削減目標）

#### **互換性検証**:
- [ ] 既存スモークテスト全通過
- [ ] PyVM vs llvmlite パリティ維持

### 💡 期待される効果（LoopForm原則適用）

#### **1. コードの簡潔化**
```
Before: compare.py 285行（5つの解決経路）
After:  compare.py 150行（dispatch point優先）
削減:   135行（47%削減！）
```

#### **2. 保守性向上**
- ヒューリスティック削除 → バグの温床排除
- dispatch point集約 → 合流点が自明
- LoopForm原則準拠 → 理論的裏付け

#### **3. 拡張容易性**
- generator/async → LoopSignal拡張のみ
- Yield → dispatch pointでの分岐追加
- 状態保存 → Loop1の`state`として自然に表現

### 🌟 結論: LoopForm IR理論の実証

**発見**:
1. ChatGPT実装は**既にLoopForm IR原則に準拠**している
2. PhiRegistry先頭生成 = `loop.begin` の実装そのもの
3. Resolver集約 = 制御の値化に合致

**次のステップ**:
1. PhiDispatchPoint導入で dispatch点を明示化
2. compare.pyヒューリスティック削除でシンプル化
3. LoopForm原則の完全実現

**20分のたばこ思考💨が生んだ統一直観が、実装の理論的基盤になっている！**

---
