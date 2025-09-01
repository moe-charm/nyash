# NyIR を共通交換フォーマットにする計画（All Languages → NyIR → All Targets）

目的（Goal）
- あらゆる言語のプログラムを NyIR（= Nyash MIR の公開版）へ落とし、そこから任意の実行形態（WASM/LLVM/VM/他言語）に変換する。
- 最適化は二の次。まずは意味保存（semantics-preserving）を最優先で実現し、可搬性と一貫性を担保する。

中核方針（Core Policy）
- NyIR Core（26命令）は基本セマンティクス凍結。ExternCallによる外部世界接続を含む。
- 拡張は NyIR-Ext（exceptions/concurrency/atomics）で言語固有機能を段階導入。
- Everything is Box哲学: 外部ライブラリもBIDによりBox統一インターフェースで利用。
- 仕様の正本は `docs/nyir/spec.md` に集約（Core＋Ext）。

必要拡張（Minimal Additions）
- 例外/アンワインド（Throw/TryBegin/TryEnd）
- 軽量並行/非同期（Spawn/Join/Await）
- アトミック（AtomicRmw/CAS + ordering）
→ 詳細は `docs/nyir/spec.md` の NyIR-Ext 参照

フロントエンド指針（Language → NyIR）
- C/C++/Rust: 既存IR（LLVM IR）経由または専用パーサでサブセットから対応
  - 例外→NyIR-Ext exceptions or エラー戻り値
  - スレッド→Spawn/Join、atomic→CAS/RMW
- Java/Kotlin: JVM bytecode から構造復元→NyIR（例外/スレッド/同期をExtへ）
- Python/JS/TS: AST→NyIR。辞書/配列/プロトタイプは標準Boxへ写像、例外/非同期はExtへ
- Go: panic/recover→exceptions、goroutine→Spawn/Join へ写像（将来）

バックエンド指針（NyIR → Target）
- WASM: 同期・非例外・非スレッドの最小路線から段階対応（Exceptions/Threads提案に合わせ拡張）
- LLVM: 例外/スレッド/アトミックが揃っているため先行実装が容易
- VM: 仕様の正しさ検証の基準（簡易実装でも良い）
- 他言語（ソース生成）: 可読性/慣用性は課題だが、機械的変換は可能（優先度低）

検証計画（Golden/Diff）
- Cサブセット→NyIR→C/WASM（例外なし・CASあり）
- Python/JSサブセット→NyIR→WASM（辞書/例外/非同期のサブセット）
- JVM系→NyIR→JVM bytecode（例外/スレッド）
- Rustサブセット→NyIR→LLVM（所有・weakの温存）
→ Golden NyIR を用い、interp/vm/wasm/llvm で出力一致をCI検証

関連リンク
- NyIR 仕様: `spec.md`
- ABI/BID: `../予定/native-plan/box_ffi_abi.md`
- 9.10 タスク（Copilot向け）: `../予定/native-plan/issues/phase_9_10_nyir_spec.md`

