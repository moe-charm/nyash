# 🎯 CURRENT TASK - 2025年8月23日（刷新）

## ✅ 直近の完了
1. ドキュメント再編成の完了（構造刷新）
2. プラグインBox（FileBox）基本実装とインタープリター統合
3. VM命令カウンタ＋時間計測のCLI化（`--vm-stats`, `--vm-stats-json`）とJSON出力対応

## 🚧 次にやること（再開方針）

1) MIR→VMの健全化（短期・最優先）
- 現行MIR→VMのマッピング表を作成（欠落/冗長/重複を可視化）
- サンプル/テストをVMで実行し、差分ログ（例外系・returns_result）を確認
- 成果物: `docs/reference/architecture/mir-to-vm-mapping.md`（暫定）

2) VM×プラグインシステムのE2E検証（短期）
- `tests/e2e_plugin_filebox.rs` をVMでも通す（`--features plugins`）
- ケース: `new/close`, `open/read/write`, `copyFrom(handle)`、デリゲーション from Parent
- 成果物: テストグリーン＋既知の制約を `VM_README.md` に明記

3) 命令セットのダイエット（中期：目標26命令）
- 実行統計（`--vm-stats --vm-stats-json`）でホット命令を特定
- 統合方針（例: TypeCheck/Castの整理、Array/Ref周りの集約、ExternCall→BoxCall移行）
- 段階移行（互換エイリアス→削除）と回帰テスト整備
- 成果物: 26命令案ドラフト＋移行計画

## ▶ 実行コマンド例

計測実行:
```bash
nyash --backend vm --vm-stats --vm-stats-json local_tests/test_hello.nyash > vm_stats.json
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

## 🔭 26命令ターゲット（ドラフトの方向性）
コア（候補）: Const / Copy / Load / Store / BinOp / UnaryOp / Compare / Jump / Branch / Phi / Call / BoxCall / NewBox / ArrayGet / ArraySet / RefNew / RefGet / RefSet / WeakNew / WeakLoad / BarrierRead / BarrierWrite / Return / Print or ExternCall(→BoxCall集約) + 2枠（例外/await系のどちらか）

補助: Debug/Nop/Safepointはビルドモードで有効化（命令としては非中核に降格）

---
最終更新: 2025年8月23日（MIR/VM再フォーカス、26命令ダイエットへ）
