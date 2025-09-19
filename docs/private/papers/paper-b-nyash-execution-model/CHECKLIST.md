# 論文B チェックリスト（Nyash 言語と実行モデル）

## スコープ確定
- [ ] 言語コアの対象範囲（構文/型/Box/Plugin/実行系）を明示
- [ ] birth/init/pack/fini の役割分担を定義
- [ ] 実行バックエンド間の共通 API と相違点を表で整理

## 実証/事例
- [ ] P2P Intent サンプル（送受・同期・検証）
- [ ] Plugin Store デモ（動的ロード/安全策）
- [ ] GUI/Web 例（EguiBox/WebCanvasBox）

## 再現性と評価
- [ ] バックエンド切替の同一入力 → 同一出力デモ
- [ ] 性能・起動時間・メモリの比較表
- [ ] 参考実装（サンプルコード）を figures/examples と併記

## 図表
- [ ] Box 階層/ABI/メモリ生存域（birth→fini）
- [ ] 実行経路の切替図（Bridge→VM/JIT/AOT/WASM）
- [ ] Intent モデルの時系列図

## 原稿
- [ ] Abstract（JP/EN）
- [ ] 本文（JP/EN）: main‑paper‑jp.md / main‑paper.md
- [ ] 関連研究（JVM/BEAM/Actor/Capability/Plugin）

## 生成物
- [ ] `tools/papers/build.sh b-jp` / `b-en` 成功
- [ ] `docs/private/out/paper-b-*.pdf` 出力を確認

## 提出準備
- [ ] arXiv/会議フォーマット整合
- [ ] 参考文献整備

---

メモ: 言語仕様の一次ソースは `docs/reference/` を規範にし、papers 配下の参照は重複を避ける（リンク推奨）。
