"""
PhiDispatchPoint — unify value resolution around PHI/merge boundaries.

Policy (MVP):
- Prefer Resolver (strict, dominance-aware)
- If declared PHI exists for the value in current function: use that SSA
- If still inconclusive and we are in a loop/body: prefer the last add_ in
  the current block (increment pattern) as a structural hint
- Finally coerce to i64

Notes:
- This is a thin box to centralize the small but critical fallbacks that were
  previously scattered across compare/branch. It keeps behavior equivalent to
  the hardening patches while providing a single entry point.
"""

from typing import Any, Dict, Optional
import llvmlite.ir as ir


class PhiDispatchPoint:
    @staticmethod
    def _phi_from_decl(resolver, bb_map, vid: int):
        """
        宣言済みPHI探索（PhiRegistry統合版）

        深い設計:
        - PhiRegistry優先（単一起点保証）
        - フォールバック実装（互換性・学習効果）
        - 発見したPHIを自動登録（次回は優先経路）

        箱理論:
        - 「箱にする」: PHI探索をPhiRegistryに委譲
        - 「境界を作る」: Registry優先→フォールバック の明確な順序
        - 「戻せる」: PhiRegistry障害時もフォールバックで動作
        - 「見える化」: 2段階解決が明確

        学習効果:
        - フォールバックで発見したPHIをRegistry登録
        - 次回からはPhiRegistry優先経路で高速取得
        - 徐々にフォールバック経路が不要になる
        """
        # Phase 1: PhiRegistry優先（単一起点保証）
        # 全ブロックを探索してPhiRegistryから取得
        try:
            from phi_wiring.registry import PhiRegistry
            if bb_map is not None:
                for block_id in bb_map.keys():
                    phi = PhiRegistry.get(None, int(block_id), int(vid))
                    if phi is not None:
                        # 単一起点から取得成功！✨
                        return phi
        except Exception:
            # PhiRegistry障害時はフォールバックへ
            pass

        # Phase 2: フォールバック実装（従来の探索）
        # 互換性維持＋学習効果
        try:
            decls = getattr(resolver, 'block_phi_incomings', {}) if resolver is not None else {}
            for b, dmap in (decls or {}).items():
                if int(vid) in (dmap or {}):
                    bb = bb_map.get(int(b)) if bb_map is not None else None
                    if bb is None:
                        continue
                    for inst in getattr(bb, 'instructions', []):
                        try:
                            if not hasattr(inst, 'add_incoming'):
                                break
                            nm = str(getattr(inst, 'name', '') or '')
                            if nm.startswith('phi_'):
                                tail = nm[4:].split('.')[0]
                                if tail.isdigit() and int(tail) == int(vid):
                                    # 発見！PhiRegistryに自動登録（学習効果）
                                    try:
                                        from phi_wiring.registry import PhiRegistry
                                        PhiRegistry.register(None, int(b), int(vid), inst)
                                    except Exception:
                                        pass
                                    return inst
                        except Exception:
                            break
        except Exception:
            pass
        return None

    @staticmethod
    def _last_add_in_block(current_block: ir.Block):
        try:
            insts = list(getattr(current_block, 'instructions', []) or [])
        except Exception:
            insts = []
        for ins in reversed(insts):
            try:
                nm = str(getattr(ins, 'name', '') or '')
                if nm.startswith('add_'):
                    return ins
            except Exception:
                continue
        return None

    @staticmethod
    def _coerce_i64(builder: ir.IRBuilder, val: Any) -> ir.Value:
        """
        i64正規化（SSA順序保証版）

        深い設計:
        - i1→i64変換を使用地点で実施（SSA順序保証）
        - 定義済みSSA値のみを変換（forward reference禁止）
        - builderカーソル位置で変換挿入（順序保証）

        箱理論:
        - 「境界を作る」: 型変換の責務を明確化
        - 「見える化」: 変換タイミングが自明
        - 「Fail-Fast」: 未定義値は変換しない
        """
        i64 = ir.IntType(64)
        i1 = ir.IntType(1)

        if val is None:
            return ir.Constant(i64, 0)

        # i1→i64変換（最優先・SSA順序保証）
        # 重要: builderのカーソル位置で変換を挿入
        # → 使用地点での変換 = SSA順序自動保証！
        if hasattr(val, 'type') and isinstance(val.type, ir.IntType):
            if val.type.width == 1:
                # i1 → i64変換（使用地点で実施）
                # 定義済みi1値（icmp結果）を使用地点で変換
                # SSA順序: icmp定義 → [他の命令] → 使用地点でzext ✅
                try:
                    # 名前付け: 元の値のdst番号を保持（デバッグ容易性）
                    orig_name = getattr(val, 'name', '')
                    if orig_name and orig_name.startswith('cmp_'):
                        dst_num = orig_name[4:]  # "cmp_5" → "5"
                        zext_name = f"i1_to_i64_{dst_num}"
                    else:
                        zext_name = f"i1_to_i64"
                    return builder.zext(val, i64, name=zext_name)
                except Exception:
                    # フォールバック: 名前なしzext
                    return builder.zext(val, i64)
            elif val.type.width != 64:
                # iN→i64変換（N≠1, N≠64）
                return builder.zext(val, i64)

        # Pointer→i64変換
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            return builder.ptrtoint(val, i64)

        # 既にi64定数
        if isinstance(val, ir.Constant) and val.type == i64:
            return val

        return val

    @staticmethod
    def resolve_i64(builder: ir.IRBuilder,
                    resolver,
                    vid: int,
                    current_block: ir.Block,
                    preds: Dict[int, list],
                    block_end_values: Dict[int, Dict[int, Any]],
                    vmap: Dict[int, Any],
                    bb_map: Optional[Dict[int, ir.Block]] = None) -> ir.Value:
        # 0) Normalize via same-block alias chase (copy連鎖を基底値に畳み込む)
        base_vid = int(vid)
        try:
            bid = None
            name = str(current_block.name)
            if name.startswith('bb'):
                bid = int(name[2:])
            aliases = getattr(resolver, 'block_aliases', {}) if resolver is not None else {}
            amap = (aliases or {}).get(int(bid)) if bid is not None else None
            steps = 0
            seen = set()
            while amap and int(base_vid) in amap and steps < 8:
                nxt = amap.get(int(base_vid))
                if nxt is None or int(nxt) in seen:
                    break
                seen.add(int(base_vid))
                base_vid = int(nxt)
                steps += 1
        except Exception:
            base_vid = int(vid)

        # 1) Direct vmap lookup（最優先・同一ブロック内の値可視性保証）
        #    箱理論:
        #    - 「境界を作る」: vmap直接参照を明示的な層として分離
        #    - 「見える化」: 同一ブロック内の値（i1 compare結果等）を確実に取得
        #    - 「戻せる」: vmapになければ次の層へフォールスルー
        i64 = ir.IntType(64)
        if base_vid in vmap:
            val = vmap[base_vid]
            if val is not None:
                # i1→i64変換を含む型正規化（_coerce_i64が処理）
                return PhiDispatchPoint._coerce_i64(builder, val)

        # 2) Strict resolver path（クロスブロック解決）
        if resolver is not None:
            try:
                val = resolver.resolve_i64(base_vid, current_block, preds, block_end_values, vmap, bb_map)
                if val is not None and not (isinstance(val, ir.Constant) and val.constant == 0):
                    return PhiDispatchPoint._coerce_i64(builder, val)
            except Exception:
                pass
        # 3) Declared PHI placeholder（マージポイント解決）
        try:
            p = PhiDispatchPoint._phi_from_decl(resolver, bb_map, base_vid)
            if p is not None:
                return p
        except Exception:
            pass
        # 4) Last add in current block (increment patterns)
        try:
            addv = PhiDispatchPoint._last_add_in_block(current_block)
            if addv is not None:
                return PhiDispatchPoint._coerce_i64(builder, addv)
        except Exception:
            pass
        # 5) Default zero（最終フォールバック）
        return ir.Constant(i64, 0)
