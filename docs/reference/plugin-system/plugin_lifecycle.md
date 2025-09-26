# Plugin Lifecycle and Box RAII

最終更新: 2025-08-22

## 概要
NyashのBoxには「ユーザー定義Box」「ビルトインBox」「プラグインBox」があります。いずれもRAII（取得した資源は所有者の寿命で解放）に従いますが、プラグインBoxは共有やシングルトン運用があるため、追加ルールがあります。

## 共通ライフサイクル（ユーザー/ビルトイン/プラグイン）
- インスタンスの寿命が尽きると、強参照フィールド（public/private）に対し順に `fini()` が呼ばれ解放（weak はスキップ）
- `local` 変数のスコープを抜けると、そのスコープで生成されたインスタンスは解放対象
- 明示的に `fini()` が呼ばれた場合も同様に後処理を実施

補足:
- これらは Nyash のスコープトラッカにより実施されます
- 解放順は生成の逆順（LIFO）で、カスケード `fini` を保証します

## プラグインBoxの特則（シングルトン）
- シングルトン（`nyash.toml`）
  - プラグインのBox型は `singleton = true` を宣言可能
  - ローダが起動時に `birth()` し、以後は同一ハンドルを共有して返却
  - シャットダウン時（`shutdown_plugins_v2()` など）に一括 `fini()` されます

補足:
- Nyashは参照カウントを採用しません。解放は「スコープ終了」または「明示的`fini`」のみで決まります（自己責任モデル）。
- プラグインBoxも同じルールです。スコープ終了時に`fini`され、以後の利用はエラー（Use after fini）。
- 長寿命が必要なケースは「シングルトン」で運用してください（個別のBoxに特例は設けない）。

### 例: `nyash.toml` 抜粋
```toml
[libraries."libnyash_counter_plugin.so".CounterBox]
type_id = 7
singleton = true
```

## Net Plugin（HTTP/TCP）運用メモ
- ログ
  - `NYASH_NET_LOG=1` で有効化、`NYASH_NET_LOG_FILE=net_plugin.log` 出力先
- 並列実行とポート
  - E2Eや並列CIではポート競合を避けるため、テスト毎にポートを明示（例: 8080, 8081, ...）
  - サーバ終了タイミング（`stop()`/スコープ終了）とクライアント接続の順序に注意

## ベストプラクティス
- ユーザー/ビルトインBox
  - フィールドの weak 指定（循環参照の解消）を活用
  - 必要に応じて明示 `fini()` を呼び、高価な資源（ファイル/ソケット等）を早期解放
- プラグインBox
  - シングルトン化が望ましい長寿命資源（サーバ、デバイス）に `singleton = true`
  - 複数スコープで共有される可能性がある値は、スコープ終了時に自動 `fini` されないことを前提に設計
  - 終了前に `shutdown_plugins_v2()` を呼ぶと単一箇所で確実に `fini` を実行可能

## 実装参照
- スコープ追跡: `src/scope_tracker.rs`（スコープ終了時の `fini` 呼出し、プラグインBox自動 `fini` 回避）
- プラグインローダ: `src/runtime/plugin_loader_v2.rs`（シングルトン生成・保持・シャットダウン、`PluginHandleInner::drop` の `fini`）
