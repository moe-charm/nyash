# Mini‑VM Guide（自己ホスト最小実行器）

目的
- JSON v0 の最小命令（const/binop/compare/copy/branch/jump/ret）を Nyash だけで実装し、自己ホストの観測と検証を行う。
- 責務分離と一貫インターフェースで“箱”に切り出し、Fail‑Fast を徹底。

箱の構成（Box‑First）
- `InstructionScannerBox` — ブロック内のオブジェクト走査（escape‑aware）
- `JsonScanBox` — `[]`/`{}` の終端検出（エスケープ対応）
- `JsonFragBox` — 断片抽出（key:int / key:str）
- `ArithmeticBox` — 安全な10進演算（Add/Sub/Mul）と i64 アダプタ
- `CompareOpsBox` — 記号→種別マッピングと比較の評価（Eq/Ne/Lt/Le/Gt/Ge）
- `OpHandlersBox` — 上記ユーティリティへ薄い委譲（binop/compare/const）
- `MirVmMin` — 実行器本体（ret 政策・分岐・ジャンプ）

Ret ポリシー（Fail‑Fast）
1. 直前の compare の `dst` と一致する `value` なら、その結果を返す。
2. レジスタが数値で定義済みならその値を返す。
3. 上記いずれでもなければ `[ERROR] Undefined register ret: rN` を出し、`-1` を返す。
   - 0 とエラーを混同しないために、`-1` をエラーマーカーとして採用。

演算と比較の一元化
- Add/Sub/Mul は `ArithmeticBox` に集約。桁あふれを避けるため内部は10進文字列で実装し、公開APIは i64 で返す。
- 比較は `CompareOpsBox` に集約。`map_symbol()` と `eval()` を提供。
- `OpHandlersBox`/`MirVmMin`/`StepRunnerBox` はこれらへ委譲し、重複実装を排除。

契約・検証
- `NYASH_CHECK_CONTRACTS=1` を既定（VMレイヤ）。unborn の操作は禁止、`birth()` は冪等。
- `OperatorGuard` により演算子ボックスの再入を抑止（VMエントリで集中インターセプト）。

テスト（smokes）
- 算術/比較の正気性: `vm_arith_semantics_vm.sh`, `vm_compare_semantics_vm.sh` など。
- Mini‑VM ret: `selfhost_minivm_thin_vs_legacy_*_vm.sh`。
- JsonScan: `jsonscan_seek_array_end_vm.sh`（エスケープ対応の配列終端）。



### OperatorBox（デバッグ用）
- `apps/selfhost/vm/boxes/operator_box.hako`
- 目的: Mini‑VM/StepRunner から比較・算術を箱経由で呼び出し、ネイティブ評価とのパリティ確認や観測に使う。
- 既定: 未使用（非再入ポリシーのため、Rust VM 側の実行核からは呼び戻さない）。
- 使い方（例）:
  ```nyash
  using "apps/selfhost/vm/boxes/operator_box.hako" as OperatorBox
  print("LT=" + (""+OperatorBox.compare("Lt", 1, 2)))   // → LT=1
  print("ADD=" + (""+OperatorBox.apply2("Add", 3, 5))) // → ADD=8
  ```
