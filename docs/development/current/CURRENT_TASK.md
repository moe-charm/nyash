# 🎯 CURRENT TASK - 2025年8月23日（刷新）

## ✅ 直近の完了
1. ドキュメント再編成の完了（構造刷新）
2. VM×プラグインのE2E整備（FileBox/Net）
   - FileBox: open/write/read, copyFrom(handle)（VM）
   - Net: GET/POST（VM）、404/500（Ok(Response)）、unreachable（Err(ErrorBox)）
3. VM命令カウンタ＋時間計測のCLI化（`--vm-stats`, `--vm-stats-json`）とJSON出力対応
   - サンプル/スクリプト整備（tools/run_vm_stats.sh、local_tests/vm_stats_*.nyash）
4. MIR if-merge 修正（retがphi dstを返す）＋ Verifier強化（mergeでのphi未使用検知）
5. ドキュメント追加・更新
   - Dynamic Plugin Flow（MIR→VM→Registry→Loader→Plugin）
   - Netプラグインのエラーモデル（unreachable=Err, 404/500=Ok）
   - E2Eテスト一覧整備
6. CI: plugins E2E ジョブ（Linux）を追加

## 🚧 次にやること（再開方針）

1) MIR→VMの健全化（短期・最優先）
- マッピング表更新（Err経路・Handle戻り・Result整合を実測で反映）
- Verifierルールの拡充（use-before-def across merge を強化）
- 成果物: `docs/reference/architecture/mir-to-vm-mapping.md`（更新済・追補）

2) VM×プラグインシステムのE2E検証（短期）
- FileBox/Netを中心にケース拡張（大きいボディ、ヘッダー多数、タイムアウト等）
- 成果物: E2E追補＋`VM_README.md` に既知の制約とTipsを追記

3) 命令セットのダイエット（中期：目標26命令）
- 実測（HTTP OK/404/500/unreachable、FileBox）を反映して合意版を確定
- 統合方針（TypeOp/WeakRef/Barrierの統合、ExternCall最小化）
- 段階移行（ビルドモードでメタ降格、互換エイリアス→削除）と回帰テスト整備
- 成果物: 26命令案（合意版）＋移行計画

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
最終更新: 2025年8月23日（VM×Plugins安定・MIR修正・26命令合意ドラフト／再起動チェックポイント）

## 🔁 再起動後の再開手順（ショート）
```bash
# 1) ビルド
cargo build --release -j32

# 2) plugins E2E（Linux）
cargo test --features plugins -q -- --nocapture

# 3) VM Stats 代表値の再取得（任意）
tools/run_vm_stats.sh local_tests/vm_stats_http_ok.nyash vm_stats_ok.json
tools/run_vm_stats.sh local_tests/vm_stats_http_err.nyash vm_stats_err.json
```
