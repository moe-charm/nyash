# Phase 12 Archive - AIたちの誤解の記録

## 🚨 重要：このフォルダの内容について

このフォルダには、Phase 12の議論で生まれた**誤解に基づいたドキュメント**が保存されています。

### なぜ誤解が生まれたのか

AIたちは「プラグイン」という言葉から、以下のような複雑な仕組みが必要だと考えました：
- トランスパイル（Nyash→C）
- 埋め込みVM
- 特別なABI変換
- JIT/AOT統合

### 実際の真実

**Nyashスクリプト = 普通のユーザー定義Box**

```nyash
# これで十分！特別な仕組みは不要
box MyPlugin {
    init {
        me.file = new FileBox()  # C ABIプラグイン使える
    }
    process(data) {
        return me.file.read(data)
    }
}
```

### 教訓として

これらのドキュメントは、以下の教訓を示すために残しています：
1. シンプルな解決策を見逃してはいけない
2. 技術用語に惑わされてはいけない
3. Everything is Boxの哲学を忘れてはいけない

## 📁 アーカイブ内容

- `CRITICAL-ISSUE.md` - 存在しない問題を解決しようとした記録
- `01_roadmap_final.md` - 不要なトランスパイル実装計画
- `02_spec_embedded_vm.md` - 不要な埋め込みVM仕様
- `03_spec_box_arguments.md` - 不要なBox受け渡し仕様
- その他、AI会議での誤解に基づく提案

---

*「時に、最も賢い解決策は何もしないことである」*