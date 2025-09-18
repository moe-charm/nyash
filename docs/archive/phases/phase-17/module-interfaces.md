# Phase 17 接続モデル（最小）: JSON NDJSON over stdio, exe↔exe

目的: まずは「簡単・見える・壊れない」。単一の接続モデル（JSONを1行ずつ、標準入出力でやり取り）で、エンジンを別プロセスの小さな実行ファイルとして積み上げる。

## 方針
- 1接続モデルのみ: NDJSON（1メッセージ=1行のJSON）。
- 1トランスポートのみ: 標準入出力（子プロセス起動）。パイプ/ソケットは後回し。
- 1エンジン=1 exe: `nyash-engine-core13`, `nyash-engine-loop`など。役割ごとに小さく分割。
- 可視化容易: すべてのトレースをNDJSONで出力し、そのまま保存・比較・変換（jq/簡易スクリプト）。

## メッセージ共通フィールド
- `op`: 操作名（例: "load_module", "call", "trace_sub"）
- `id`: リクエストID（数値/文字列）。応答に同じ`id`を返す。
- `schema`: プロトコル版（整数, 例: 1）。
- `ts`: 送信者時刻（オプション, ns/μs/ms表現は自由）。

応答の共通:
- `ok`: true/false
- `id`: リクエストと同一
- `err`: 失敗時のみ `{code, message, data?}`

## 操作一覧（最小）
1) load_module
- 要求: `{op:"load_module", id:1, schema:1, ir:"core13|loop", format:"json|bin", bytes:"<base64>"}`
- 応答: `{ok:true, id:1, module_id:1, features:["interp","trace"], ir:"core13"}`

2) call
- 要求: `{op:"call", id:2, module_id:1, func:"main", args:[1,2,3], timeout_ms:5000?}`
- 応答: `{ok:true, id:2, value:3, events_dropped:0}`

3) trace_sub（トレース購読）
- 要求: `{op:"trace_sub", id:3, mask:["EnterFunc","ExitFunc","LoopIter","Branch","ExternCall"], flush_ms:50?}`
- 応答: `{ok:true, id:3}`
- イベントストリーム: 別行で `{"event":"EnterFunc", "func":"main", ...}` を逐次出力（応答行とは独立）

4) unload（任意）
- 要求: `{op:"unload", id:4, module_id:1}`
- 応答: `{ok:true, id:4}`

5) ping（ヘルスチェック）
- 要求: `{op:"ping", id:5}`
- 応答: `{ok:true, id:5, now: 1725600000}`

エラー例:
- 応答: `{ok:false, id:2, err:{code:"E_NO_FUNC", message:"function not found", data:{func:"main"}}}`

## トレースイベント（NDJSON）
- 例: `{"event":"EnterFunc","func":"main","args":[1],"ts":...}`
- 最小セット: `EnterFunc, ExitFunc, Block, PhiMerge, LoopIter, Branch, ExternCall, Safepoint, Barrier`
- フィールドの原則: `event`必須、その他は柔軟に拡張し未知キーは無視可能。

## エンジンexeの約束
- 起動直後にバナー等を出力しない（標準出力は完全にプロトコル専用）。
- 標準エラーは自由（ログ用）。
- 読み込みは行単位でJSONをパース、応答は必ず1行。
- イベントは応答と独立行で流してよい（trace_sub済みのとき）。
- 大きなIRバイナリは `bytes` を base64。JSON IRはそのまま文字列可。

## クライアント側（nyash-cli）
- `nyash run --engine=remote --exe ./nyash-engine-core13 --ir=core13 -- program.ny`
- 実装は単純: 子プロセスspawn → stdin/stdoutにNDJSON → `load_module`→`call`。
- `--trace`で `trace_sub` を投げてNDJSONログをファイル保存（そのまま可視化可能）。

## 可視化（最小）
- NDJSON→Markdown表: `jq -r` で列抽出, `|`区切りでテーブル化。
- NDJSON→CSV: `jq -r '[.event, .func, .block, .ts] | @csv'`。
- 差分: 2つのトレースを event+順序でzip比較。簡易Python/nyashツールは後続で追加可。

## バージョニングと互換
- `schema` は整数で宣言、互換性は「追加のみ」原則。意味変更や削除は `schema+1`。
- 不明キーは無視。必須キー欠落は `E_SCHEMA`。

## タイムアウト/健全性
- `call.timeout_ms` を受付。内部でキャンセル/中断可能な範囲で対応。
- ハートビート: `ping` を定期送信して固まり検出。

## セキュリティ
- ローカル実行のみを想定（最小）。ネットワーク露出はしない。
- 外部からのファイルアクセス/実行はプロトコルに含めない（エンジンはIR実行に限定）。

## 最小試験手順（手動）
1) エンジンを起動（手動で別ターミナル）。`cat`でNDJSONを流せるならなお良い。
2) `load_module` を送信（小さなIR）。
3) `trace_sub` を送信。
4) `call` を送信 → 応答とイベント列を保存。
5) jqでCSV化して目視。

## 将来拡張（後回し）
- 長期接続（ソケット/名前付きパイプ）
- バイナリMessagePack/bincode実装（速度重視）
- 圧縮（gzip）
- ストリーム再同期（途中から読み直し）
