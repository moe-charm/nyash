# エントリーポイント規約（Hakorune）

目的: 入門者にやさしく、明確で予測可能な起動点を提供する。

結論（既定ポリシー）
- 既定のエントリは Strict: `Main.main` のみ。
- 推奨の書き方は「flow Main」。静的状態が必要な場合のみ「static box Main」。

許可される2つの形
1) flow Main（推奨）
```
flow Main {
  main() {
    // … your code …
    return 0
  }
}
```

2) static box Main（例外的に状態が必要な場合）
```
static box Main {
  // 例: 定数テーブルや少量のキャッシュ
  const TABLE = 42

  main() {
    // … your code …
    return 0
  }
}
```

禁止／非推奨
- トップレベル `main()` はエントリとして扱いません（Strict）。
- `flow` 以外の名前（例: `flow App { main() }`）はエントリになりません。
  - その場合は `flow Main` にリネームするか、（将来）CLI の `--entry` を利用してください。

flow と static の違い（要約）
- flow: フィールドなし、`me`・`birth/fini` なし、純関数グループ。入門・標準エントリに最適。
- static: フィールドあり（最小限）、`me` 不可。共有の小さな静的状態が必要なときのみ選択。

エラー時のガイド
- `Main.main` が見つからない場合はエラーになります。
- メッセージに候補（`*.main`）が列挙されます。`flow Main` へ改名するか、（将来）`--entry` を指定してください。

備考（移行）
- 旧ドキュメント/実装の一部ではトップレベル `main` を受理している場合がありますが、廃止方向です。Strict ポリシーへ順次統一します。
