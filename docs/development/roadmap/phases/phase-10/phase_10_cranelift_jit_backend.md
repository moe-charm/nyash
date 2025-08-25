# Phase 10: Cranelift JIT Backend（MIR→VM→Cranelift）

Status: Planned (Primary path for native speed)
Last Updated: 2025-08-25

## 🎯 ゴール
- 実行系の主経路を「MIR→VM」を維持しつつ、ホットパスをCraneliftでJIT化して高速化する。
- LLVM AOTは後段（Phase 11以降）の研究対象へ繰り延べ。

## 🔗 位置づけ
- これまでの案（MIR→LLVM AOT）を改め、現実的な開発速度と安定性を優先してCranelift JITを先行。
- VMとのハイブリッド実行（OSR/ホットカウントに基づくJIT）を採用。

## 📐 アーキテクチャ
```
AST → MIR → Optimizer → VM Dispatcher
                         └─(Hot)→ Cranelift JIT (fn単位)
```
- VMが命令カウント・プロファイルを集計し、しきい値超過関数をJITコンパイル。
- JIT済み関数は関数テーブルから直接呼び出し、VMはフォールバック先として維持。

## 📋 スコープ
1) 基盤
- JITマネージャ（関数プロファイル・コンパイルキャッシュ）
- Craneliftコード生成（MIR→CLIF Lower）
- 呼出しABI（Nyash VMスタック/レジスタとのブリッジ）

2) 命令カバレッジ（段階導入）
- Phase A: Const/Copy/BinOp/Compare/Jump/Branch/Return（純関数相当）
- Phase B: Call/BoxCall/ArrayGet/ArraySet（ホットパス対応）
- Phase C: TypeOp/Ref*/Weak*/Barrier（必要最小）

3) ランタイム連携
- Boxの所有・参照モデルを維持（共有/クローンの意味論を破らない）
- 例外・TypeErrorはVMの例外パスへエスケープ

## ✅ 受け入れ基準（Milestone）
- M1: 算術/比較/分岐/returnの関数がJIT化され、VMより高速に実行
- M2: Array/Mapの代表操作（get/set/push/size）がJITで安定動作
- M3: BoxCallホットパス（特にArray/Map）で有意な高速化（2×目標）
- M4: 回帰防止のベンチと`--vm-stats`連携（JITカウント/時間）

## 🪜 実装ステップ
1. JITマネージャ/関数プロファイルの導入（VM統計と統合）
2. MIR→CLIF Lower骨子（基本型/算術/比較/制御）
3. 呼出しABIブリッジ（引数/戻り値/BoxRefの表現）
4. JIT関数テーブル + VMディスパッチ切替
5. Array/Map/BoxCallのホットパス最適化
6. TypeOp/Ref/Weak/Barrierの必要最小を実装
7. ベンチ/スナップショット整備・回帰検出

## ⚠️ 依存・前提
- MIR26整合（TypeOp/WeakRef/Barrierの統合前提）
- P2PBox再設計（Phase 9.x）を先に安定化しておく（VM/プラグインE2E維持）

## 📚 参考
- Cranelift: Peepmatic/CLIF、simple_jitの最小例
- JIT/VMハイブリッド: LuaJIT/HotSpotのOSR設計

---
備考: LLVM AOTはPhase 11以降の研究路線に移行（設計ドキュメントは維持）。

