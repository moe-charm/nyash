# Phase 10.5: Core Standard (String/Array/Map) in Nyash — Rust依存の段階的削減

目的
- 現状Rust実装に依存している基本コンテナ（String/Array/Map）を、Nyashで実装したstdへ段階的に置換し、セルフホストへ近づける。
- rt/sys層（Box ABI・所有・weak・最小アロケータ、`ny_host_*`）を活用して堅牢性と性能の両立を図る。

前提
- Phase 10.2: Host API層（C-ABI `ny_host_*` / WASM `nyir_host`）
- Phase 10.3: 層の切り分け（corelang/rt/sys/std）
- Phase 10.4: Box ABI（fat ptr）とEffect→LLVM属性の方向性

範囲（MVP）
- String
  - 構造: { ptr: *u8, len: usize, cap: usize }
  - API: new, from_raw, into_raw, clone, len, is_empty, push_str, substr(view), to_utf8(view)
  - メモリ: `ny_host_alloc/realloc/free` 経由、UTF-8不変（validation optional）
- Array<T>
  - 構造: { ptr: *T, len: usize, cap: usize }
  - API: new, push, pop, get(i), set(i,v), len, reserve
  - メモリ: `ny_host_*` 経由、要素のfiniハンドリング（Box所有規則順守）
- Map<K,V>
  - 構造: ハッシュテーブル（オープンアドレス or チェイン; v0は単純で可）
  - API: new, get, set, remove, len, keys(view), values(view)
  - メモリ: `ny_host_*` 経由、キー/値の所有/weak規則順守

設計ポリシー
- 所有とfini: 再代入・スコープ終端でfiniが適切に発火すること（Everything is Box準拠）
- 互換: 現行言語表面の挙動に合わせる（差異は仕様に明記）
- 効果: mut操作の順序保持、view系はpure（読み取り）
- WASM/LLVM: ABI/ExternCallと矛盾しない（Stringの(ptr,len)は共通）

タスク（Copilot TODO）
1) stdレイアウトの骨子作成（ファイル/モジュール構成）
2) String v0実装 + 単体テスト（push_str/len/substr）
3) Array v0実装 + 単体テスト（push/get/set/len）
4) Map v0（簡易hash）+ 単体テスト（set/get/remove/len）
5) 再代入/スコープ終端でのfini挙動の統合テスト
6) ベンチ: 既存Rust実装対比の大まかな目安（悪化しない/許容範囲）
7) フェールセーフ: OOM/境界エラーの明確化（panic/Resultは設計に従う）
8) ドキュメント: stdのMVP API一覧と互換要件

受け入れ基準
- 代表サンプルがRust実装なしでString/Array/Mapを利用し動作
- 再代入・スコープ終端時にfiniが期待通り発火（ログで可視化）
- WASM/LLVMの文字列(ptr,len)取り扱いと整合（print等のExternCallで可視化）

リスク・軽減
- パフォーマンス劣化: ベンチで目視確認、ホットパス最適化は後続で実施
- メモリ安全: 所有/weak/効果規則をVerifierで補助（後続でLSP/静的解析を強化）
- 実装負債: MVP範囲を明確にし、機能追加はIssue分割

参考
- ABIドラフト: docs/予定/native-plan/box_ffi_abi.md
- NyIR: docs/nyir/spec.md
- Host API: Phase 10.2 仕様

最終更新: 2025-08-14
