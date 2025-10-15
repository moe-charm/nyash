"""
StringTagPolicy — 外部関数返り値タグポリシー管理箱

箱理論の完璧な実践:
1. 「箱にする」: タグポリシーを専用箱に完全分離
2. 「境界を作る」: 関数名→タグ種別の明確なマッピング
3. 「戻せる」: ポリシー変更が局所的（この箱だけ）
4. 「見える化」: 一箇所で全ポリシー確認可能

深い設計:
- Immutable設計: POLICYは定数（実行時変更不可）
- Fail-Fast: 不明な関数は何もしない（silent success）
- テスト容易性: ポリシー判定とタグ適用を分離

責務:
- 外部関数の返り値型ポリシーを一元管理
- string handle/pointer返りの自動タグ付け
- 拡張時の一箇所修正保証
"""

from typing import Optional


class StringTagPolicy:
    """
    外部関数返り値タグポリシー管理

    Immutable Policy Design:
    - ポリシーは定数として定義（実行時変更不可）
    - 判定メソッドは副作用なし（pure function）
    - タグ適用は統一エントリポイント（apply_tag）
    """

    # ========================================
    # Policy Definition（Immutable）
    # ========================================

    # String handle返り（i64 handle型）
    # handle型返りの関数は resolver.mark_string() で文字列タグ付与
    STRING_HANDLE_RETURNS = frozenset({
        "nyash.string.concat_hh",      # handle + handle → handle
        "nyash.string.substring_hii",   # handle + int + int → handle
        "nyash.box.from_i8_string",    # i8* → handle（boxing）
        # Note: lastIndexOf_hh は i64（数値）返りなのでタグ不要
    })

    # String pointer返り（i8* 型）
    # pointer型返りの関数は resolver.string_ptrs に登録
    STRING_POINTER_RETURNS = frozenset({
        "nyash.string.concat_ss",      # i8* + i8* → i8*
        "nyash.string.concat_si",      # i8* + i64 → i8*
        "nyash.string.concat_is",      # i64 + i8* → i8*
        "nyash.string.substring_sii",  # i8* + int + int → i8*
    })

    # ========================================
    # Policy Query（Pure Functions）
    # ========================================

    @staticmethod
    def is_string_handle(func_name: str) -> bool:
        """
        String handle返りか判定（副作用なし）

        Args:
            func_name: 外部関数名（例: "nyash.string.concat_hh"）

        Returns:
            True: string handle型返り
            False: それ以外

        深い設計:
        - frozenset.inでO(1)判定
        - 副作用なし（pure function）
        - テスト容易
        """
        return func_name in StringTagPolicy.STRING_HANDLE_RETURNS

    @staticmethod
    def is_string_pointer(func_name: str) -> bool:
        """
        String pointer返りか判定（副作用なし）

        Args:
            func_name: 外部関数名

        Returns:
            True: string pointer型返り
            False: それ以外
        """
        return func_name in StringTagPolicy.STRING_POINTER_RETURNS

    @staticmethod
    def needs_any_tag(func_name: str) -> bool:
        """
        何らかのタグが必要か判定（最適化用）

        Returns:
            True: タグ付与が必要
            False: タグ不要（早期リターン可能）

        深い設計:
        - apply_tag()の早期リターン判定用
        - タグ不要関数は即座にスキップ（パフォーマンス）
        """
        return (StringTagPolicy.is_string_handle(func_name) or
                StringTagPolicy.is_string_pointer(func_name))

    # ========================================
    # Tag Application（Unified Entry Point）
    # ========================================

    @staticmethod
    def apply_tag(func_name: str, dst_vid: Optional[int], resolver) -> None:
        """
        タグ自動付与（統一エントリポイント）

        Args:
            func_name: 外部関数名
            dst_vid: 返り値の格納先MIR値ID（Noneなら何もしない）
            resolver: Resolver instance（タグ格納先）

        副作用:
        - resolver.mark_string(dst_vid) 呼び出し
        - resolver.string_ptrs.add(dst_vid) 呼び出し

        深い設計:
        - 早期リターン: タグ不要なら即座に終了
        - Fail-Fast: dst_vidがNoneならエラーではなく何もしない
        - 防御的: resolverが不正でもクラッシュしない

        使用例:
        >>> from instructions.string_tag_policy import StringTagPolicy
        >>> StringTagPolicy.apply_tag("nyash.string.concat_hh", 42, resolver)
        >>> # → resolver.mark_string(42) が呼ばれる
        """
        # 早期リターン: タグ不要
        if not StringTagPolicy.needs_any_tag(func_name):
            return

        # 早期リターン: dst_vidがNone（返り値なし）
        if dst_vid is None:
            return

        # 早期リターン: resolverがNone（テスト環境等）
        if resolver is None:
            return

        # String handle返りのタグ付与
        if StringTagPolicy.is_string_handle(func_name):
            try:
                if hasattr(resolver, 'mark_string'):
                    resolver.mark_string(int(dst_vid))
            except Exception:
                # Silent failure（Fail-Fastではなくsilent success）
                # 理由: タグ付与失敗は致命的エラーではない
                pass

        # String pointer返りのタグ付与
        elif StringTagPolicy.is_string_pointer(func_name):
            try:
                # 1. string_ptrsに登録
                if hasattr(resolver, 'string_ptrs'):
                    resolver.string_ptrs.add(int(dst_vid))

                # 2. mark_stringも付与（互換性）
                # pointer返りもstring扱いにする（比較・連結で使用可能）
                if hasattr(resolver, 'mark_string'):
                    resolver.mark_string(int(dst_vid))
            except Exception:
                # Silent failure
                pass

    # ========================================
    # Extension Point（将来拡張用）
    # ========================================

    @staticmethod
    def get_all_tagged_functions() -> frozenset:
        """
        タグ付与対象の全関数名を取得（デバッグ・ドキュメント用）

        Returns:
            タグ付与対象の関数名set

        使用例（ドキュメント自動生成）:
        >>> for func in StringTagPolicy.get_all_tagged_functions():
        ...     print(f"- {func}")
        """
        return (StringTagPolicy.STRING_HANDLE_RETURNS |
                StringTagPolicy.STRING_POINTER_RETURNS)


# ========================================
# Module-Level Convenience（後方互換）
# ========================================

def apply_string_tag(func_name: str, dst_vid: Optional[int], resolver) -> None:
    """
    モジュールレベル関数（後方互換用）

    Note: StringTagPolicy.apply_tag() の直接利用を推奨
    """
    StringTagPolicy.apply_tag(func_name, dst_vid, resolver)
