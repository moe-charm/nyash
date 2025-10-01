# Phase 10.1: JIT→EXE via Plugin Box Unification

## 🎯 革新的発見：すべてはプラグインになる

### 核心的洞察
既存のプラグインシステム（BID-FFI）がすでに完全なC ABIを持っている。
これを活用することで、JIT→EXE変換が現実的に可能。

## 📊 フェーズ概要

### 目標
- ビルトインBoxをプラグイン化してC ABI統一
- JITから統一されたプラグインAPIを呼び出し
- スタティックリンクによるスタンドアロンEXE生成

### 背景
```
現在の構造:
- JIT → HostCall → Rustビルトイン（複雑）
- JIT → PluginInvoke → プラグインBox（C FFI）

統一後:
- JIT → PluginInvoke → すべてのBox（統一！）
- EXE → PluginInvoke → スタティックリンクされたBox
```

## 🚀 実装計画

### Week 1: ArrayBoxプラグイン化PoC（詳細は phase_plan.md 参照）
- ArrayBoxをプラグインとして再実装
- JITからのプラグイン呼び出しテスト
- パフォーマンス測定（HostCall vs Plugin）

### Week 2: 主要Box移行（詳細は phase_plan.md 参照）
- StringBox、IntegerBox、BoolBoxのプラグイン化
- JIT lowering層の統一（plugin_invoke経由）
- 既存HostCallとの共存メカニズム

### Week 3: 静的リンク基盤（詳細は phase_plan.md 参照）
- プラグインの`.a`ライブラリビルド
- 最小ランタイム（nyash-runtime）設計
- リンカースクリプト作成

### Week 4: EXE生成実証（詳細は phase_plan.md 参照）
- Hello Worldレベルのスタンドアロン実行
- Linux/macOSでの動作確認
- デバッグ情報とunwind対応

## 📁 ディレクトリ構造（予定）

```
plugins/
├── nyash-core-boxes/        # ビルトインBox群
│   ├── nyash-array-plugin/
│   ├── nyash-string-plugin/
│   └── nyash-integer-plugin/
├── nyash-runtime-minimal/   # 最小ランタイム
└── existing/               # 既存プラグイン
    ├── nyash-file-plugin/
    └── nyash-net-plugin/
```

## 🔗 関連資料（整備済み）

- フェーズ計画の詳細: [phase_plan.md](./phase_plan.md)
- C ABI v0 仕様（JIT/AOT/Plugin共通）: ../../../../docs/reference/abi/nyrt_c_abi_v0.md
  - 命名: `nyrt_*`（コア）/ `nyplug_{name}_*`（プラグイン）
  - 呼出規約: x86_64 SysV / aarch64 AAPCS64 / Win64
  - `*_abi_version()` で fail-fast（v0=1）

## ストリームエラー対策（長文/大出力を避ける）
- 先頭に短い要約（サマリ）を置く（本READMEの冒頭にあり）
- 詳細設計や長いコードは分割して参照（phase_plan.md / nyrt_c_abi_v0.md）
- コマンドやコードは三連バッククォートで閉じ忘れ防止

- [革新的アプローチ詳細](../../../ideas/new-features/2025-08-28-jit-exe-via-plugin-unification.md)
- [プラグインAPI仕様](../../../../reference/plugin-system/)
- [Phase 10.5: Python統合計画](../phase-10.5/) （旧10.1）
- [Phase 10.10: 前段階の成果](../phase-10/phase_10_10/)

## ⚡ 成功指標

1. **技術的検証**
   - ArrayBoxがプラグインとして動作
   - JITからの呼び出し成功
   - 性能劣化10%以内

2. **統合達成**
   - 5つ以上のビルトインBoxがプラグイン化
   - JIT lowering層の完全統一

3. **EXE生成**
   - スタンドアロン実行ファイル生成
   - 基本的なNyashプログラムの動作

## 🎉 期待される成果

- **Everything is Plugin** - 新たな設計哲学の確立
- 自己ホスティングへの現実的な道筋
- プラグインエコシステムの拡大可能性

---

*"Everything is Box → Everything is Plugin → Everything is Possible"*
