# JIT Events JSON (v0.1)

最小のイベントJSONスキーマ。観測の足場として、値は安定キーのみを出力します。

- 出力形態: 1行 = 1 JSON（JSONL）
- 出力先: 標準出力 or `NYASH_JIT_EVENTS_PATH` で指定したファイル
- phase分離:
  - compile: `phase: "lower"`（明示opt-in: `NYASH_JIT_EVENTS_COMPILE=1`）
  - runtime: `phase: "execute"`（既定ON可: `NYASH_JIT_EVENTS=1` または `NYASH_JIT_EVENTS_RUNTIME=1`）

## フィールド（必須）
- kind: 文字列（例: "hostcall"）
- function: 文字列（現状は "<jit>" で固定）
- phase: 文字列（"lower" | "execute"）
- id: シンボル名（例: "nyash.map.get_hh"）
- decision: 文字列（"allow" | "fallback"）
- reason: 文字列（"sig_ok" | "receiver_not_param" | "policy_denied_mutating" | "sig_mismatch" など）
- argc: 数値（観測引数数）
- arg_types: 文字列配列（例: ["Handle","I64"]）

任意フィールド（存在時のみ）
- handle: 数値（JITハンドル）
- ms: 数値（処理時間ミリ秒）

## 出力例

compile（lower）:
```json
{"kind":"hostcall","function":"<jit>","id":"nyash.map.get_hh","decision":"allow","reason":"sig_ok","argc":2,"arg_types":["Handle","Handle"],"phase":"lower"}
```

runtime（execute）:
```json
{"kind":"hostcall","function":"<jit>","id":"nyash.array.push_h","decision":"fallback","reason":"policy_denied_mutating","argc":2,"arg_types":["Handle","I64"],"phase":"execute"}
```

trap（execute 中の失敗）:
```json
{"kind":"trap","function":"<jit>","reason":"jit_execute_failed","ms":0,"phase":"execute"}
```

## 環境変数（抜粋）
- NYASH_JIT_EVENTS=1: 既定のruntime出力
- NYASH_JIT_EVENTS_COMPILE=1: compile（lower）出力
- NYASH_JIT_EVENTS_RUNTIME=1: runtime出力
- NYASH_JIT_EVENTS_PATH=path.jsonl: ファイルに追記
- NYASH_JIT_THRESHOLD（未設定時）: 観測ONで自動的に1が補われます（Runner/DebugConfigBoxが補助）

## 推奨の最小運用
- 現象確認: `NYASH_JIT_EVENTS=1`（runtimeのみ）
- 解析時のみcompile出力: `NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_EVENTS_PATH=events.jsonl`
- HostCall系の例では `NYASH_JIT_HOSTCALL=1` を明示
