# 論文B アウトライン（Nyash 言語と実行モデル）

## 0. タイトル候補
- Nyash: A Box‑First Programming Language with Symmetric Memory and Unified Execution
- Box‑First Language Design and a Multi‑Backend Execution Model

## 1. 問題設定 / 貢献
- 問題: 言語の抽象モデル（Box）と実行基盤（VM/JIT/AOT/Bridge）を矛盾なく共存させる設計。
- 核となる貢献:
  - 対称メモリ管理（birth/init/pack ↔ fini）の設計と実装方針。
  - Box 第一級の ABI とプラグイン統合（TypeBox, BID‑FFI）。
  - 多層実行（Interpreter/VM/JIT/AOT/WASM）を貫く一貫 API。
  - 実アプリ事例（P2P/GUI/Plugin）による実用性の検証。

## 2. 言語設計
- Everything is Box（値・モジュール・リソースを単一モデルで表現）。
- メソッド後置例外処理（参考: paper‑m）の位置づけ（本論では設計原理に触れるのみ）。
- Namespace/using/型正規化の方針（MVP レベル）。

## 3. 実行モデル
- Bridge/Runner と各バックエンドの役割（Phase‑15 ポリシー）。
- PyVM を意味論リファレンス、llvmlite を規範とする運用。
- 短絡（&&/||）や PHI の意味論合意を跨いだ一貫性維持策。

## 4. 実装事例 / ケーススタディ
- P2P Intent モデルのサンプル（NyashCoin 等）。
- Plugin Store / Box 間相互運用。
- GUI/Web（EguiBox / WebCanvasBox）。

## 5. 評価設計
- 表現力/学習容易性/開発効率の定性的比較。
- 実行バックエンドの性能・起動時間・メモリ比較。
- 実アプリの応答性・安定性。

## 6. 関連研究
- OOP/Actor/Capability/Component/Plugin 設計との比較。
- 実行系（WASM/LLVM/JVM/BEAM）との立ち位置整理。

## 7. まとめ
- Box‑First 設計の実用性と拡張余地。

---

## 執筆メモ（短期タスク）
- birth/fini の API と実装例を図示（リソース生存域）。
- P2P Intent のメッセージ流れ図・簡易ベンチ（往復遅延）。
- バックエンド切替のデモ（同一ソース → VM/JIT/AOT）。
