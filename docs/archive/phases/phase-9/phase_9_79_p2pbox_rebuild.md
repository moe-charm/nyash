# Phase 9.79: P2PBox再設計・実装（Cranelift前に完了）

Status: Planned (Pre-Cranelift priority)
Last Updated: 2025-08-25

## 🎯 目的
Cranelift導入前にP2P通信基盤（P2PBox/IntentBox/MessageBus/Transports）を再設計・実装し、VM/インタープリター双方で安定動作させる。

## 📦 スコープ
1) モデル/API
- IntentBox（TLV/serde互換）
- MessageBus（単一共有・購読/発行・ログ）
- P2PBox（new/on/send/pack、デリゲーション整合）

2) Transports（段階導入）
- InProcess（同プロセスbus）
- WebSocket（WSクライアント/サーバ連携）
- WebRTC（将来、 signalingはOut of Scope）

3) 実行統合
- VM/InterpreterのBoxCall経由で同一API
- プラグイン/ExternCallと競合しない設計（BIDと将来統合）

## ✅ 受け入れ基準
- `p2p_spec.md` の代表ケースがVM/Interpreterで成功
- E2E: `node_a.send("bob", IntentBox(...))` が InProcess で往復確認
- `NYASH_VM_DEBUG_BOXCALL=1` でも追跡容易（ログ整備）

## 🪜 実装ステップ
前提（9.78h）: [phase_9_78h_mir_pipeline_stabilization.md](phase_9_78h_mir_pipeline_stabilization.md) の受け入れ基準を満たすこと。

1. IntentBoxの最小実装（payload: MapBox/ArrayBox）
2. MessageBus（購読/発行、ハンドラ登録）
3. P2PBox（new/on/send、packはビルトインのみ）
4. InProcessTransport（同プロセス配送）
5. WebSocketTransport PoC（非同期I/OはInterpreter側、VMはフォールバック）
6. E2E/スナップショット・ドキュメント整備

## 🔗 参考
- docs/reference/execution-backend/p2p_spec.md
- docs/guides/p2p-guide.md

---
備考: 既存 `src/boxes/p2p_box.rs` は古い設計。完全新規で実装する。
