# 🧱 Nyash開発哲学: 「箱を作って下に積む」

作成日: 2025-09-02
ステータス: 運用指針（安定）

## 目的
機能を最短で安全に通しつつ、後戻り可能性と可観測性を最大化するための実践原則。Nyashは「Everything is Box」。開発も同様に、まず箱（境界・足場）を作り、下に積んでいく。

## 中核原則（Box-First）
- 境界一本化: 変換・正規化は境界箱（Boundary/Registry/Semantics）1箇所に集約。
- 下に積む: 上位機能の前に、受け皿となる下位の箱（ABI/Registry/Hostcall/Semantics）を先に用意。
- 小さい箱: API面は最小・明確に。用途が広がれば能力(capability)を後置きで追加。
- いつでも戻せる: env/feature/Box設定で切替可能。フォールバック経路を常設し破壊的変更を避ける。
- 可観測性先行: JSON/DOT/タグ付きログなどの観測を同時に追加し、振る舞いを記録・比較可能にする。
- 明示性優先: 暗黙の魔法を排除（by-id固定、explicit override/from、strictトグル）。

## 積む順序（5ステップ）
1) 境界箱を置く: Semantics/ABI/HandleRegistry/Hostcall/Config のいずれかに着地点を用意。
2) no-op足場: 失敗しない実装（no-op/同期get/仮戻り値0）でまず通す。
3) 小さく通す: ゴールデン3件（成功/失敗/境界）で tri-backend を最小通過。
4) 観測を入れる: Result行・stats.jsonl・CFG DOT・TRACE env を追加（デフォルト静か、必要時のみON）。
5) 厳密化: 型/戻り/エラーを段階で厳密化、フォールバックを削り strict を既定へ寄せる。

## 具体適用（現行ライン）
- Semantics層: 加算/比較/文字列化の正規化をVM/JIT/Interpreterで共有。
- 単一出口: returnは必ずretブロックに集約。PHIはローカルへ実体化しReturnはlocal load。
- Handle-First + by-id: PluginInvokeは常に a0=handle / method_id 固定。TLVでプリミティブ化。
- Await/Future: まず同期解決で安全着地→Cancellation/Timeout/Err統一を段階導入。
- Safepoint: checkpointはglobal_hooks経由でGC/スケジューラ連携（no-op→実装）。

## アンチパターン（避けること）
- バックエンド横串: VM/JIT/LLVMが互いを直接知る配線。
- 境界分散: 値変換やポリシーが複数箇所に散らばる。
- 先に最適化: 観測や足場なしで高速化のみを入れる。
- 暗黙フォールバック: 失敗を隠して通す（原因が観測できない）。
- 仕様の局所実装: 便宜的な特例 if/else を増やす（規約化せずに拡散）。

## 成功判定（DoD）
- tri-backend一致: Script/VM/JIT（必要に応じAOT）でResult系の一致。
- 観測可: stats/TRACE/DOTが残り、回帰比較が可能。
- リバータブル: env/feature/Box設定で旧経路へ即時切替可能。
- 文書化: 追加箱/APIの概要と使用例をdocsへ追補（最小）。

## 付録: 代表Box一覧（足場）
- SemanticsBox: coerce/compare/arith/concat の正規化
- HandleRegistry: ハンドル↔実体の一元管理
- InvokeHost: by-id呼び出し（固定長/可変長、TLV）
- JitConfigBox: 環境設定の集約窓口
- ObservabilityBox: stats.jsonl/CFG DOT/traceの管理

この順序と原則で、壊さず、早く、何度でもやり直せる。

