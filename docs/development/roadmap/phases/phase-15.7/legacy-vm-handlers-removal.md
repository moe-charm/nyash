# VM Convenience Handlers — Deprecation & Removal Plan (Phase 15.7)

Status: planned; Scope: src/backend/mir_interpreter/handlers/*（VM固有）

## 対象（撤退候補）
- String ハンドラ: `handlers/boxes_string.rs`
  - length/len/size, indexOf/lastIndexOf, substring, concat, stringify, isEmpty, charAt
- Array ハンドラ: `handlers/boxes_array.rs`
  - birth, push, len|length|size, get, set, toJSON, isEmpty
- Map ハンドラ: `handlers/boxes_map.rs`
  - birth, set/get/has/delete/size/keys/values/toJSON, isEmpty
- BoxCall fast‑path: `handlers/calls/box_call.rs`
  - Map/Array fast‑pathの部分は段階的に削除（Plugin 経路へ移管）

## 撤退順（小さく安全に）
1. String convenience（Plugin に同等実装がある範囲）
   - 依存度: 低〜中（selfhost では最小使用）
   - 入口: Plugin での StringBox vtable を優先、VM ハンドラは統合削除
2. Map convenience（get/set/has/size/keys/values）
   - 依存度: 中（selfhost-vm が参照）
   - 先行: 返り値と miss 仕様は既に統一（get→null、set/clear→null）
   - 移行: Plugin 優先、VM ハンドラは段階削除
3. Array convenience（len/size/get/set/push/toJSON）
   - 依存度: 中〜高（テストが多い）
   - 移行: Plugin での実装優先を確認後、VM ハンドラを削除
4. BoxCall fast‑path の整理
   - Plugin/Kernel の vtable/extern 経路が安定後に削除

## 影響範囲
- Runner: 既定 Plugins OFF のため quick は VM 便宜ハンドラへの依存が残る
  - 削除は Plugin ON（auto/force）環境で先に検証→段階的に quick 側を切替
- Selfhost: Pipeline/VM ミニ器は最小APIで動作。順守: quick 緑維持

## ガード/受け入れ
- quick 全緑（VM/LLVM 代表）
- plugin-on での同等スモークが緑（Plugin優先）
- 撤退ごとに M1/M2/M3 の自己ホストスモークが緑

## 備考
- 長期: CoreBoxの Kernel 側残骸は Null/Missing など最小系に限定
- 文字列エラー判定（"Key not found:") は全面廃止（null チェックへ統一）
