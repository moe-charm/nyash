# JIT論文プロジェクト - Box-First設計

## 📝 メイン論文（ChatGPT5作）

### 現在の原稿
- **[00-README.md](00-README.md)** - プロジェクト概要
- **[01-abstract.md](01-abstract.md)** - アブストラクト（日英）
- **[02-paper-draft.md](02-paper-draft.md)** - 本文（2-3ページ短編）✨
- **[03-figures-notes.md](03-figures-notes.md)** - 図の作成メモ
- **[box-acceleration-chatgpt5.md](box-acceleration-chatgpt5.md)** - 箱理論によるJIT開発加速事例（2025-08-29）🆕

### 論文の特徴
- **タイトル**: Box-First JIT: Decoupled, Probe-Driven JIT Enablement in Nyash within 24 Hours
- **長さ**: 2-3ページ（ワークショップ/ポスター向け）
- **切り口**: AI支援開発での「力づく最適化を避ける」方法論
- **キーワード**: 可視・可逆・切替可能

## 🎯 次のステップ

1. **DOT図の生成**
```bash
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_PHI_MIN=1 \
NYASH_JIT_DOT=tmp/phi_bool.dot \
./target/release/nyash --backend vm examples/phi_bool_merge.nyash

dot -Tpng tmp/phi_bool.dot -o figures/phi_bool_cfg.png
```

2. **図の準備**
- Timeline図（24時間の実装フロー）
- Box構造図（設定/境界/観測）
- CFG可視化（phi_bool_merge）
- 性能グラフ（1.06-1.40倍）

3. **投稿先検討**
- PPL 2026（日本語OK）
- Onward! Essays（新視点歓迎）
- PX Workshop（開発体験重視）

## 📁 アーカイブ

過去の草稿や分析は[archives/](archives/)フォルダに保管されています：
- 初期ドラフト
- Gemini先生との相談記録
- 各種分析文書
- ベンチマーク詳細

## 🚀 実装デモ

論文で引用されているデモ：
- `examples/phi_bool_merge.nyash` - Boolean PHIマージ
- `examples/mix_num_bool_promote.nyash` - 型昇格デモ

使用例：
```bash
# b1パスのデモ実行
./target/release/nyash --backend vm --jit-exec --jit-threshold 1 \
  --jit-phi-min examples/phi_bool_merge.nyash
```