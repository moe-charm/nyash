# when構文 - パターンマッチング（将来実装予定）

## 概要
Nyashに「when構文」を導入し、より直感的で安全なエラー処理とパターンマッチングを実現する。

## 背景
- ChatGPT5提案の`returns_result = true`による段階的Result正規化
- 現在の`if res.is_ok()`パターンは冗長
- Nyashの「Everything is Box」哲学に合致した統一的な構文が必要

## 提案構文

### 基本形 - ResultBoxのパターンマッチング
```nyash
when res {
    ok(resp) -> {
        body = resp.readBody()
        print(body)
    }
    error(err) -> {
        print("Error: " + err.message())
    }
}
```

### 汎用形 - あらゆるBoxでのパターンマッチング
```nyash
when value {
    StringBox(s) -> print("文字列: " + s)
    IntegerBox(n) -> print("数値: " + n.toString())
    ArrayBox(arr) -> print("配列の長さ: " + arr.length())
    NullBox -> print("nullです")
    _ -> print("その他のBox")  // デフォルトケース
}
```

### ネスト可能
```nyash
when httpResult {
    ok(response) -> {
        when response.getStatus() {
            200 -> print("成功！")
            404 -> print("見つからない")
            _ -> print("その他のステータス")
        }
    }
    error(e) -> print("エラー: " + e)
}
```

## 実装前提条件
1. **MIRダイエット完了**（現在33個→目標20-26個）
   - 新しいパターンマッチング命令の追加余地が必要
2. **VM最適化完了**（Phase 8.6）
   - 効率的なジャンプテーブル実装が必要
3. **`returns_result = true`の段階導入**
   - Net系Boxから開始

## 実装計画
1. Phase 9後半: MIR命令追加（Match, MatchBranch等）
2. Phase 10: VM/JITでの最適化実装
3. Phase 11: 言語仕様への正式組み込み

## 利点
- **直感的**: switch-case的な馴染みやすい構文
- **型安全**: 各パターンで正しい型のメソッドが呼べる
- **網羅的**: すべてのケースをカバー可能
- **拡張性**: 将来の新Box型にも対応可能
- **Nyash哲学**: Everything is Boxに完全に合致

## 他の検討案
1. **chain構文**: `res.onSuccess({}).onError({})`
2. **try-else構文**: `try resp = res {} else err {}`
3. **?演算子**: `resp = res.unwrap?()`

これらも将来的に検討可能だが、when構文が最もNyashらしい。

---
提案日: 2025-08-20
提案者: Claude & ChatGPT5 協調開発チーム