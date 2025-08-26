# Phase 9.79a: Unified Box Dispatch (Minimal) + P2PBox Polish

Status: Completed
Last Updated: 2025-08-26
Owner: core-runtime

## Goals
- Unify basic Box methods (toString/type/equals/clone) at the dispatcher level without breaking NyashBox trait.
- Simplify VM/Interpreter method routing with a single, predictable path for “universal” methods.
- Polish P2PBox: formalize share/clone semantics and safe unregister; finalize multi-node + async UX.

## Scope
1) Dispatcher-level unification (非侵襲)
- VM: universal methods (toString/type/equals/clone) を型分岐前に一括処理。
- Interpreter: 同様の「ユニバーサル優先→型別詳細」パターンを採用。
- NyashBoxトレイトは現状の`to_string_box/type_name/equals/clone_box/share_box`を維持。

2) P2PBox磨き（Phase 9.79 継続）
- 共有セマンティクス: share_box()はtransport/handlers/flags共有、clone_box()は新規transport（済）。
- InProcess unregister: 一致解除 or refcount で安全に戻す（暫定停止の解除）。
- Transport API: `register_intent_handler`は導入済。WS/WebRTCの下準備（薄いshim）を設計。
- E2E: on()/onOnce()/off()、self-ping、two-node ping-pong、TimeBox併用のasyncデモ。
- VM: P2Pヘルパ（getLast*/debug_*）のtoString/Console出力をInterpreterに寄せて整合。

## Out of Scope（今回はやらない）
- 全BoxにUnifiedBoxトレイトを適用する大改修（段階的移行のため見送り）。
- ビルトインBoxの完全プラグイン化（Phase 10+ 候補）。
- NyashValueの全面置換（機会見つけて漸進導入）。

## Deep Analysis Docとの整合
- まずはディスパッチャで統一（トレイト変更なし）→ 破壊的変更を避けつつ美しさを担保。
- Nyash言語の`toString/type/equals/clone`はVM/Interpreterで中央集約的にRust側APIへ橋渡し。
- 「Everything is Box」を壊さずに一貫した呼び出し体験を提供する。

## Milestones
- M1（Day 1–2）
  - VM: universal methods 前置ディスパッチ
  - Interpreter: 同様の前置ディスパッチ
  - スモーク：既存演算子/print動作の回帰なし
  - 進捗: 2025-08-26 達成（VM/Interpreterともに toString/type/equals/clone を前段で統一。cargo build 成功）
- M2（Day 3–4）
  - P2PBox unregister安全化（endpoint一致 or refcount）
  - E2E: onOnce/off 追加、two-node ping-pong 安定、asyncデモが確実に出力
- M3（Day 5）
  - VM表示整合：P2PヘルパのtoString/ConsoleをInterpreterと一致
  - Docs更新：言語ガイド/P2Pリファレンス反映

## Completion Notes (2025-08-26)
- Universal dispatch (toString/type/equals/clone): Interpreter/VMに前段実装・整合確認済み。
- P2PBox Polish:
  - InProcess unregister: endpoint一致時のみunregisterで安全化。
  - E2E: onOnce/off ユニットテスト追加、two-node ping→pong スモーク、self→selfスモーク追加。
  - 受信トレース: getLastFrom/getLastIntentName を受信時に更新。
  - 実用ミニ糖衣: IntentBoxの第2引数に MapBox/JSONBox を直接渡せるよう拡張。
- Docs: 新規リファレンス追加（P2P）/ 例追加
  - docs/reference/boxes-system/p2p_box.md
  - examples/p2p_self_ping.nyash
  - examples/p2p_ping_pong.nyash

Notes:
- 非WASM環境のTimerBoxはダミーのため、async出力の確実化はWASM側のガイドで扱う。ネイティブでは同期スモーク（self→self/二者）で安定確認。

## Next (roll-forward)
- Language sugar: Object literal → MapBox lowering（feature flag `object_literal`で段階導入）
  - Proposal: docs/ideas/improvements/2025-08-26-object-literal-sugar.md
- WASMガイドにTimer併用のasyncサンプル追記。

## リスクと対策
- VM分岐に触るリスク → 型別分岐の“前段”に追加、既存分岐はフォールバックとして維持
- unregister回りの退行 → 一致解除テスト/順次Dropテスト（clone/share/Drop順の組み合わせ）を追加

## 受け入れ基準
- VM/Interpreterの両方で toString/type/equals/clone が統一パスで動作
- P2PBox: multi-node ping-pong + onOnce/off E2Eが通り、asyncデモが確実にログ出力
- 既存スモークに回帰なし、Docs更新完了

## 備考
- UnifiedBox.dispatch_methodはPhase 10での検討項目として温存。
- NyashValueの活用はMIR/VM安定化と歩調を合わせて拡大。
