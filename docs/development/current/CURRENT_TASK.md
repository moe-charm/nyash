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
7. E2E拡張（Net/Socket）
   - HTTP: 大ボディ取得クライアント `local_tests/http_big_body_client.nyash`
   - Socket: 反復タイムアウト検証 `local_tests/socket_repeated_timeouts.nyash`
   - インタープリタ: SocketBoxの `acceptTimeout/recvTimeout` を結線
8. VM/MIRの健全化（Builder/VM）
   - Compare拡張: Float/Int-Float混在をサポート（Eq/Ne/Lt/Le/Gt/Ge）
   - TypeOp(Check)最小意味論実装（Integer/Float/Bool/String/Void/Box名）
   - ArrayGet/ArraySet（VM）本実装（ArrayBox.get/setへ橋渡し）
   - Array/Mapをidentity扱い（clone_or_shareがshareを選択）
   - BoxCallにArrayBox fast-path（BoxRefからget/set直呼び）
   - me参照の安定化（fallback時に一度だけConstを発行しvariable_mapに保持）
   - デバッグ: `NYASH_VM_DEBUG_BOXCALL=1` でBoxCallの受け手/引数/経路/結果型を標準エラーに出力
9. ドキュメント追加・更新
   - MIR→VMマッピング（分岐条件の動的変換、Void/Bool比較）
   - VM README（SocketBoxタイムアウト/E2E導線・HTTP Result整理）
   - 26命令ダイエット: PoCフラグと進捗追記（TypeOp/WeakRef/Barrier）
10. CI: plugins E2E ジョブ（Linux）を追加

## 🚧 次にやること（再開方針）

1) 命令セットダイエットのPoC実装（短期）
   - 現状: VMに `TypeOp/WeakRef/Barrier` 実行経路（等価）とPrinter対応。Builderに補助APIを追加済（未置換）。
   - 次: Builder内の該当箇所を補助APIに置換（flag onで新命令を吐く／offで従来どおり）
   - 旗: `mir_typeop_poc`（TypeCheck/Cast→TypeOp）、`mir_refbarrier_unify_poc`（Weak*/Barrier→統合）
   - 成果物: スナップショット（flag on/off）＋ vm-statsのキー確認（TypeOp/WeakRef/Barrier）

2) VM×プラグインのE2E拡張（短期）
   - HTTP: 遅延応答・大ボディの計測、到達不能時のERR安定化の再検証（代表は追加済）
   - Socket: 反復タイムアウトの追加ケース（代表は追加済）
   - 成果物: 必要に応じてE2E追補と `VM_README.md` のTips更新

3) ResultBox単一路線への統合（中期）
- 新`NyashResultBox`へ統合、旧`ResultBox`は薄いラッパーとして段階移行
- 成果物: 実装整理・移行メモ・影響調査

4) Array系の本実装（必要時・中期）
   - VMの `ArrayGet/ArraySet` 実装済み。BoxCall fast-pathの整合性と回帰テストを充実

5) BoxCall高速化（性能段階）
- vm-statsでホットなBoxCallの高速化（命令セット統合より効果大の可能性）

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
最終更新: 2025年8月23日（VM強化・E2E拡張・me参照安定化・TypeOp/WeakRef/Barrier PoC完了／次段はBuilder置換とスナップショット）

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

# 5) 反復タイムアウト確認（任意）
./target/release/nyash local_tests/socket_repeated_timeouts.nyash

# 6) HTTP 大ボディ確認（任意）
./target/release/nyash local_tests/http_big_body_client.nyash

# 7) VM BoxCall デバッグ（任意）
NYASH_VM_DEBUG_BOXCALL=1 ./target/release/nyash --backend vm local_tests/test_vm_array_getset.nyash
```
