# Nyash言語と実行モデル——Box‑First設計と二系統実行（PyVM/LLVM）
著者: Nyash Project

要旨
NyashはBox‑First設計（Everything is Box）を採用し、birth/init/pack↔finiの対称的メモリ管理、プラグインABI、そして実行系の二系統（PyVM/LLVM）で開発・配布を成り立たせる。本稿ではPhase‑15での現実的な運用範囲として、PyVM（意味論基準）とLLVM（PHI合成・AOT/EXE）に評価を絞り、Boxモデルと実行責務の分担設計を示す。

## 1. はじめに
Boxを言語の第一級抽象として採用し、型・所有・リソース・ABIを単一のメンタルモデルで統一する。設計の簡素さを保ちながら現実的な配布物を得るため、実装コストの高い経路（JIT/Interpreter）はPhase‑15では補助に留め、PyVM/LLVMの二系統を強化した。

## 2. 言語設計（Box‑First）
- Boxモデル: 値・モジュール・リソースの統一表現
- 対称メモリ: birth/init/pack と fini による決定的解放（将来GCオン/オフ両立へ拡張可能）
- プラグインABI: TypeBox/BID‑FFI（`docs/reference/plugin-system/`）での安定相互運用
- 例外と修飾子（将来含む）: Block + Modifier の方向性（詳細は別稿）

## 3. 実行モデル（二系統）
- PyVM（基準）: 短絡やtruthy規約を含む意味論確認。仕様差検出・パリティ基盤
- LLVM（配布）: AOT/EXE生成、PHI合成、IRダンプ/トレース
- 責務分担:
  - MIRはPHI‑off（合流はエッジコピー）
  - LLVMがブロック先頭でPHI合成（typed incoming）
  - PyVMは意味論の参照実装、LLVMは配布物と性能の源泉

## 4. ケーススタディ
- 文字列/配列/MapなどのBoxメソッド（BoxCall）での一貫API
- プラグイン連携（ファイル/パス等）とABI境界の単純化
- 短絡・分岐合流でのパリティ（PyVM=意味, LLVM=PHI合成）確認

## 5. 評価計画（v0）
- パリティ: `tools/parity.sh --lhs pyvm --rhs llvmlite apps/tests/CASE.nyash`
- 性能: LLVM実行時間/起動時間/メモリ（PyVMは意味論sanity）
- トレース: `NYASH_LLVM_TRACE_PHI=1`, `NYASH_LLVM_DUMP_IR=...`

## 6. 関連研究
OOP/Actor/Capability/Plugin指向設計と実行系（LLVM/JVM/WASM）との比較。Nyashの特徴は「Box‑First × 実行責務の分離（PyVM=意味, LLVM=生成）」にある。

## 7. 結論
Box‑First設計と二系統実行の分担により、複雑性を爆発させずに言語の実働ラインを維持できた。次段階ではLoopFormやJITの再統合を段階的に検討する。

### 再現手順
- LLVMハーネス: `NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/CASE.nyash`
- パリティ: `tools/parity.sh --lhs pyvm --rhs llvmlite apps/tests/CASE.nyash`
- ビルド（PDF）: `tools/papers/build.sh b-jp`

### キーワード
Box‑First, 実行モデル, PHI合成, LLVM, PyVM, プラグインABI

