# 🎯 CURRENT TASK - 2025年8月23日（刷新）

## ✅ 直近の完了
1. ドキュメント再編成の完了（構造刷新）
2. VM×プラグインのE2E整備（FileBox/Net）
   - FileBox: open/write/read, copyFrom(handle)（VM）
   - Net: GET/POST（VM）、404/500（Ok(Response)）、unreachable（Err(ErrorBox)）
3. VM命令カウンタ＋時間計測のCLI化（`--vm-stats`, `--vm-stats-json`）とJSON出力対応
   - サンプル/スクリプト整備（tools/run_vm_stats.sh、local_tests/vm_stats_*.nyash）
4. MIR if-merge 修正（retがphi dstを返す）＋ Verifier強化（mergeでのphi未使用検知、支配関係チェック導入）
5. VMの健全化（分岐・比較・Result）
   - Compare: Void/BoolのEq/Ne定義（順序比較はTypeError）
   - Branch条件: `BoxRef(BoolBox)→bool`／`BoxRef(VoidBox)→false`／`Integer≠0→true`
   - ResultBox: 新旧両実装への動的ディスパッチ統一（isOk/getValue/getError）
6. VMビルトイン強化（Array/Map/Socket）
   - ArrayBox/MapBox: 代表メソッドをVM統合ディスパッチで実装（push/get/set/size等）
   - SocketBox: `acceptTimeout(ms)`（void）/ `recvTimeout(ms)`（空文字）を追加
   - E2E追加: `socket_timeout_server.nyash` / `socket_timeout_client.nyash`
7. ドキュメント追加・更新
   - MIR→VMマッピング（分岐条件の動的変換、Void/Bool比較）
   - VM README（SocketBoxタイムアウト/E2E導線・HTTP Result整理）
   - 26命令ダイエット: PoCフラグと進捗追記（TypeOp/WeakRef/Barrier）
8. CI: plugins E2E ジョブ（Linux）を追加

## 🚧 次にやること（再開方針）

1) 命令セットダイエットのPoC実装（短期）
- フラグ `mir_typeop_poc` 有効時、Builderで TypeCheck/Cast → TypeOp を出力
- VMにTypeOp実行経路を追加（当面は既存と同義）
- 次段: `mir_refbarrier_unify_poc` で Weak*/Barrier 統合（Builder/VM）
- 成果物: スナップショット（flag on/off）＋ vm-statsで集計キー確認

2) VM×プラグインのE2E拡張（短期）
- HTTP: 遅延応答・大ボディの計測、到達不能時のERR安定化の再検証
- Socket: タイムアウト系の追加ケース（連続acceptTimeout/recvTimeout）
- 成果物: E2E追加と `VM_README.md` のTips追補

3) ResultBox単一路線への統合（中期）
- 新`NyashResultBox`へ統合、旧`ResultBox`は薄いラッパーとして段階移行
- 成果物: 実装整理・移行メモ・影響調査

## ▶ 実行コマンド例

計測実行:
```bash
tools/run_vm_stats.sh local_tests/vm_stats_http_ok.nyash vm_stats_ok.json
tools/run_vm_stats.sh local_tests/vm_stats_http_err.nyash vm_stats_err.json
tools/run_vm_stats.sh local_tests/vm_stats_http_404.nyash vm_stats_404.json
tools/run_vm_stats.sh local_tests/vm_stats_http_500.nyash vm_stats_500.json
```

VM×プラグインE2E:
```bash
cargo test -q --features plugins e2e_interpreter_plugin_filebox_close_void
cargo test -q --features plugins e2e_vm_plugin_filebox_close_void
```

MIRダンプ/検証:
```bash
nyash --dump-mir --mir-verbose examples/plugin_box_sample.nyash
nyash --verify examples/plugin_box_sample.nyash
```

## 🔭 26命令ターゲット（合意ドラフト）
- コア: Const / Copy / Load / Store / BinOp / UnaryOp / Compare / Jump / Branch / Phi / Return / Call / BoxCall / NewBox / ArrayGet / ArraySet / RefNew / RefGet / RefSet / Await / Print / ExternCall(最小) / TypeOp(=TypeCheck/Cast統合) / WeakRef(=WeakNew/WeakLoad統合) / Barrier(=Read/Write統合)
- メタ降格: Debug / Nop / Safepoint（ビルドモードで制御）

---
最終更新: 2025年8月23日（VM強化・E2E拡張・TypeOp PoC着手／次段はBuilder/VMマッピング）

## 🔁 再起動後の再開手順（ショート）
```bash
# 1) ビルド
cargo build --release -j32

# 2) plugins E2E（Linux）
cargo test --features plugins -q -- --nocapture

# 3) VM Stats 代表値の再取得（任意）
tools/run_vm_stats.sh local_tests/vm_stats_http_ok.nyash vm_stats_ok.json
tools/run_vm_stats.sh local_tests/vm_stats_http_err.nyash vm_stats_err.json

# 4) SocketBox タイムアウト確認（任意）
./target/release/nyash local_tests/socket_timeout_server.nyash
./target/release/nyash local_tests/socket_timeout_client.nyash
```
