# 付録B: 効果注釈と Safepoint/Barrier 最小規約

## Effect Mask（最小分類）
- Pure: Const/BinOp/Compare/Phi（参照透過）
- Call: ユーザー関数呼出（副作用不定、配置に応じてSafepoint許容）
- BoxCall: Boxメッセージ呼出（副作用あり得る、Barrier要否は対象に依存）
- Extern: ランタイム/プラグイン呼出（FFI境界）
- MemRead/MemWrite: メモリ読み/書き（Core‑13ではBarrierで抽象化）
- Safepoint: 安全点（GC/割込み協調用、停止可能点）
- Barrier: 読み/書きバリア（世代間/弱参照などの最小表現）

注: 実装では ExternCall/BoxCall に対し、必要最小の Barrier を付与（下位最適化で除去可能）。

## Safepoint 挿入の指針（最小）
- ループヘッダ: 長期ループの先頭に1箇所（LoopFormなら loop.begin 直後 or ヘッダブロック）
- 長期待機/FFI後: ブロッキングや長期外部呼出の復帰箇所
- コンパクション/停止要求: ランタイムのポリシーに従い、明示Safepointを挿入

## Barrier の指針（最小）
- 書き込み: 参照フィールド書換時（世代間/弱参照の一貫性確保）
- 読み出し: 弱参照/エポック制御の必要時のみ
- 下位最適化: 隣接Barrierの併合/削除を許可

---

この最小規約は、Core‑13 の命令が少ないほど効果が曖昧にならないよう、注釈で差を表現する思想に基づく。
