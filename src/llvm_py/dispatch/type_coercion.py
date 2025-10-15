"""
TypeCoercion — 型変換統一管理箱

箱理論の完璧な実践（StringTagPolicyと同じパターン）:
1. 「箱にする」: 型変換ロジックを専用箱に完全分離
2. 「境界を作る」: 変換ルールを明確に定義
3. 「戻せる」: 既存コードは段階的に移行可能
4. 「見える化」: 全変換ルールが一箇所で確認可能

深い設計:
- SSA順序保証: 使用地点変換で定義→使用の順序を保証
- 冪等性: 同じ型への変換は何もしない（最適化）
- デバッグ容易性: 名前付け規則の統一

責務:
- Any型 → i64/i1 の統一変換
- SSA順序保証（builderカーソル位置で変換）
- 変換履歴の可視化（命名規則）
"""

from typing import Any, Optional
import llvmlite.ir as ir


class TypeCoercion:
    """
    型変換統一管理箱

    StringTagPolicyと同じ設計パターン:
    - Pure Functions（副作用なし）
    - SSA順序保証（使用地点変換）
    - 冪等性（同じ型への変換は何もしない）
    """

    @staticmethod
    def to_i64(
        builder: ir.IRBuilder,
        val: Any,
        name_hint: str = ""
    ) -> ir.Value:
        """
        統一型変換: Any → i64

        深い設計:
        - i1 → i64: zext（SSA順序保証・使用地点変換）
        - iN → i64: zext/trunc（N < 64 / N > 64）
        - pointer → i64: ptrtoint
        - 既にi64: そのまま返す（冪等性）
        - None: Constant(i64, 0)

        Args:
            builder: LLVM IR builder（SSA順序保証）
            val: 変換対象の値
            name_hint: 命名ヒント（例: "cmp_5" → "i1_to_i64_5"）

        Returns:
            i64型のLLVM IR値

        箱理論:
        - 「境界を作る」: 型変換の責務を明確化
        - 「見える化」: 変換タイミングが自明（使用地点）
        - 「Fail-Fast」: 未定義値はConstant(0)で安全に処理
        """
        i64 = ir.IntType(64)

        # None → Constant(i64, 0)
        if val is None:
            return ir.Constant(i64, 0)

        # i1 → i64 変換（最優先・SSA順序保証）
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
                    if orig_name and name_hint:
                        # name_hintが指定されている場合は優先
                        zext_name = f"i1_to_i64_{name_hint}"
                    elif orig_name and orig_name.startswith('cmp_'):
                        dst_num = orig_name[4:]  # "cmp_5" → "5"
                        zext_name = f"i1_to_i64_{dst_num}"
                    elif name_hint:
                        zext_name = f"i1_to_i64_{name_hint}"
                    else:
                        zext_name = "i1_to_i64"
                    return builder.zext(val, i64, name=zext_name)
                except Exception:
                    # フォールバック: 名前なしzext
                    return builder.zext(val, i64)

            elif val.type.width == 64:
                # 既にi64: そのまま返す（冪等性）
                return val

            elif val.type.width < 64:
                # iN → i64変換（N < 64: zext）
                try:
                    name = f"zext_i{val.type.width}_to_i64_{name_hint}" if name_hint else f"zext_to_i64"
                    return builder.zext(val, i64, name=name)
                except Exception:
                    return builder.zext(val, i64)

            else:
                # iN → i64変換（N > 64: trunc）
                try:
                    name = f"trunc_i{val.type.width}_to_i64_{name_hint}" if name_hint else f"trunc_to_i64"
                    return builder.trunc(val, i64, name=name)
                except Exception:
                    return builder.trunc(val, i64)

        # Pointer → i64変換
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            try:
                name = f"ptr_to_i64_{name_hint}" if name_hint else "ptr_to_i64"
                return builder.ptrtoint(val, i64, name=name)
            except Exception:
                return builder.ptrtoint(val, i64)

        # 既にi64定数
        if isinstance(val, ir.Constant) and val.type == i64:
            return val

        # その他: デフォルトConstant(i64, 0)
        return ir.Constant(i64, 0)

    @staticmethod
    def to_i1(
        builder: ir.IRBuilder,
        val: Any,
        name_hint: str = ""
    ) -> ir.Value:
        """
        統一型変換: Any → i1（条件式用）

        深い設計:
        - i64 → i1: icmp ne 0（Truthiness変換）
        - pointer → i1: icmp ne null（null check）
        - 既にi1: そのまま返す（冪等性）

        Args:
            builder: LLVM IR builder
            val: 変換対象の値
            name_hint: 命名ヒント

        Returns:
            i1型のLLVM IR値
        """
        i1 = ir.IntType(1)
        i64 = ir.IntType(64)

        # None → Constant(i1, 0)
        if val is None:
            return ir.Constant(i1, 0)

        # 既にi1: そのまま返す（冪等性）
        if hasattr(val, 'type') and isinstance(val.type, ir.IntType) and val.type.width == 1:
            return val

        # i64 → i1: icmp ne 0（Truthiness）
        if hasattr(val, 'type') and isinstance(val.type, ir.IntType):
            zero = ir.Constant(val.type, 0)
            try:
                name = f"i64_to_i1_{name_hint}" if name_hint else "i64_to_i1"
                return builder.icmp_unsigned('!=', val, zero, name=name)
            except Exception:
                return builder.icmp_unsigned('!=', val, zero)

        # Pointer → i1: icmp ne null
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            null = ir.Constant(val.type, None)
            try:
                name = f"ptr_to_i1_{name_hint}" if name_hint else "ptr_to_i1"
                return builder.icmp_unsigned('!=', val, null, name=name)
            except Exception:
                return builder.icmp_unsigned('!=', val, null)

        # その他: デフォルトConstant(i1, 0)
        return ir.Constant(i1, 0)

    @staticmethod
    def ensure_i64(
        builder: ir.IRBuilder,
        val: Any,
        name_hint: str = ""
    ) -> ir.Value:
        """
        to_i64のエイリアス（API互換性用）

        使用例:
        >>> val_i64 = TypeCoercion.ensure_i64(builder, some_val, "arg0")
        """
        return TypeCoercion.to_i64(builder, val, name_hint)

    @staticmethod
    def ensure_i1(
        builder: ir.IRBuilder,
        val: Any,
        name_hint: str = ""
    ) -> ir.Value:
        """
        to_i1のエイリアス（API互換性用）
        """
        return TypeCoercion.to_i1(builder, val, name_hint)

    @staticmethod
    def to_type(
        builder: ir.IRBuilder,
        val: Any,
        target_type: ir.Type,
        name_hint: str = ""
    ) -> ir.Value:
        """
        任意の型への変換（return型マッチング用）

        深い設計:
        - pointer → int: ptrtoint
        - int → pointer: inttoptr
        - int → int: zext/trunc（幅調整）
        - 既に同じ型: そのまま返す（冪等性）

        Args:
            builder: LLVM IR builder
            val: 変換対象の値
            target_type: 目標の型
            name_hint: 命名ヒント

        Returns:
            target_type型のLLVM IR値

        箱理論:
        - 「境界を作る」: 型変換の責務を明確化
        - 「戻せる」: 既存の型幅変換コードを置き換え可能
        """
        # None → デフォルト値
        if val is None:
            if isinstance(target_type, ir.IntType):
                return ir.Constant(target_type, 0)
            elif isinstance(target_type, ir.DoubleType):
                return ir.Constant(target_type, 0.0)
            elif isinstance(target_type, ir.PointerType):
                return ir.Constant(target_type, None)
            else:
                return ir.Constant(target_type, 0)

        # 既に同じ型: そのまま返す（冪等性）
        if hasattr(val, 'type') and val.type == target_type:
            return val

        # Pointer → Int
        if isinstance(target_type, ir.IntType) and hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            try:
                name = f"p2i_{name_hint}" if name_hint else "p2i"
                return builder.ptrtoint(val, target_type, name=name)
            except Exception:
                return builder.ptrtoint(val, target_type)

        # Int → Pointer
        if isinstance(target_type, ir.PointerType) and hasattr(val, 'type') and isinstance(val.type, ir.IntType):
            try:
                name = f"i2p_{name_hint}" if name_hint else "i2p"
                return builder.inttoptr(val, target_type, name=name)
            except Exception:
                return builder.inttoptr(val, target_type)

        # Int → Int（幅調整）
        if isinstance(target_type, ir.IntType) and hasattr(val, 'type') and isinstance(val.type, ir.IntType):
            if target_type.width < val.type.width:
                # Truncate
                try:
                    name = f"trunc_to_i{target_type.width}_{name_hint}" if name_hint else f"trunc_to_i{target_type.width}"
                    return builder.trunc(val, target_type, name=name)
                except Exception:
                    return builder.trunc(val, target_type)
            elif target_type.width > val.type.width:
                # Zero extend
                try:
                    name = f"zext_to_i{target_type.width}_{name_hint}" if name_hint else f"zext_to_i{target_type.width}"
                    return builder.zext(val, target_type, name=name)
                except Exception:
                    return builder.zext(val, target_type)
            else:
                # 同じ幅: そのまま返す
                return val

        # その他: デフォルト値
        if isinstance(target_type, ir.IntType):
            return ir.Constant(target_type, 0)
        elif isinstance(target_type, ir.DoubleType):
            return ir.Constant(target_type, 0.0)
        else:
            return ir.Constant(target_type, None)


# ========================================
# Module-Level Convenience（後方互換）
# ========================================

def coerce_to_i64(
    builder: ir.IRBuilder,
    val: Any,
    name_hint: str = ""
) -> ir.Value:
    """
    モジュールレベル関数（後方互換用）

    Note: TypeCoercion.to_i64() の直接利用を推奨
    """
    return TypeCoercion.to_i64(builder, val, name_hint)


def coerce_to_i1(
    builder: ir.IRBuilder,
    val: Any,
    name_hint: str = ""
) -> ir.Value:
    """
    モジュールレベル関数（後方互換用）

    Note: TypeCoercion.to_i1() の直接利用を推奨
    """
    return TypeCoercion.to_i1(builder, val, name_hint)
