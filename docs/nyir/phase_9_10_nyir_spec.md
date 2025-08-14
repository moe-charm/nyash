# Phase 9.10: NyIR v1 仕様・フォーマット・検証器（公開IRの確立）

目的
- NyashのMIRを公開IR（NyIR v1）として凍結し、あらゆるフロントエンド/バックエンドの共通契約とする。
- 仕様・テキスト/バイナリフォーマット・厳格検証器・ツール群を整備し、移植性と一貫性を保証する。

成果物（Deliverables）
- 仕様書: `docs/nyir.md`（命令仕様/効果/未定義なし/検証ルール/等価変換指針）
- フォーマット: `.nyir`（テキスト）, `.nybc`（バイナリ）
- 検証器: `nyir-verify`（CLI/ライブラリ）
- 変換/実行ツール:
  - `nyashel -S`（Nyash→NyIRダンプ）
  - `nyir-run`（NyIRインタプリタ）
  - 参考: `nyir-ll`（NyIR→LLVM IR、Phase 10で拡張）
- Golden NyIR: `golden/*.nyir`（代表サンプルを固定、CIで全バックエンド一致を検証）

仕様の要点（NyIR v1）
- 命令セット: 25命令（Tier-0/1/2）を凍結
  - Tier-0: Const, BinOp, Compare, Branch, Jump, Phi, Call, Return
  - Tier-1: NewBox, BoxFieldLoad, BoxFieldStore, BoxCall, Safepoint, RefGet, RefSet, WeakNew, WeakLoad, WeakCheck, Send, Recv
  - Tier-2: TailCall, Adopt, Release, MemCopy, AtomicFence
- 効果（Effects）: pure / mut / io / control（再順序化規則を明文化）
- 所有フォレスト: 強参照の森（strong in-degree ≤ 1）、強循環禁止、weakは非伝播
- Weak: 失効時の挙動を決定（WeakLoad=null / WeakCheck=false）、世代タグ設計を想定
- Bus: ローカルは順序保証、リモートは at-least-once（または選択可能）
- 未定義動作なし: 各命令の事前条件/失敗時挙動を明示
- バージョニング: `nyir{major.minor}`、featureビットで拡張告知

テキスト形式（.nyir）
- 人間可読・差分レビュー向け
- 構造: moduleヘッダ / const pool / functions（blocks, instrs）
- コメント/メタデータ/featureビットを扱える簡潔な構文

バイナリ形式（.nybc）
- セクション化: header / features / const pool / functions / metadata
- エンコード: LEB128等の可変長を採用、識別子はstring table参照
- 将来の後方互換に備えた保守的設計

検証器（Verifier）
- 検査: 所有森/強循環/weak規則/効果整合/到達性/終端性/Phi入力整合
- 失敗時は明確なエラーを返し、ロード拒否
- CLI/ライブラリの二態（コンパイラ/実行系どちらからも利用）

Golden / Differential テスト
- `golden/*.nyir` を固定し、interp/vm/wasm/jit/llvm（順次）で出力一致をCIで検証
- 弱失効/分割fini/境界条件系を重点的に含める

タスク（Copilot TODO）
1) 仕様スケルトン: `docs/nyir.md` ひな形生成（命令/効果/検証/等価変換の目次）
2) `.nyir` パーサ/プリンタ（最小）
3) `.nybc` エンコーダ/デコーダ（最小）
4) `nyir-verify`（所有森/効果/Phi/到達/終端の基本チェック）
5) `nyashel -S` をNyIR出力対応に（既存MIRダンプ経路から移行）
6) Goldenサンプル作成（3〜5本）＋CIワークフロー雛形

受け入れ基準（Acceptance）
- 代表サンプルが `.nyir` で表現・検証・実行可能（`nyir-run`）
- `.nybc` 読み書き往復で等価
- CIでinterp/vm/wasmの結果一致（最小ケース）

依存/関連
- 8.5: 25命令の確定仕様
- 9.7: ExternCall/ABI（NyIRにもmethodノードor外部呼を表現）。v1ではExternCallは拡張セクションで可
- 10.x: NyIR→LLVM IR（別Phase）

リスク
- 仕様凍結の硬直化 → 拡張はfeatureビット＋拡張セクションへ
- 実装の重複 → Verifier/フォーマットは共有ライブラリ化

最終更新: 2025-08-14
