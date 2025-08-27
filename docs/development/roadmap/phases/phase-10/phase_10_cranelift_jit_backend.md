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

## 🔬 Sub-Phases (10_a .. 10_h)

各サブフェーズは「小さく立ち上げ→検証→次へ」。既存のVM/Thunk/PICを活用し、JITは同じ経路に自然合流させる。

### 10_a: JITブートストラップ（基盤＋プロファイラ）
- 目的: Cranelift JITの骨組みとホット関数検出の導線を用意。
- 具体: 
  - `JitManager`（関数プロファイル、しきい値、キャッシュ）
  - CLIF環境初期化（`Module`, `Context`, `ISA`）
  - VM統合: 関数入口でホット度チェック→JITコンパイル/呼び出し
  - 診断: `NYASH_JIT_STATS=1`（JIT件数/時間/キャッシュヒット）
- 受入: ダミー関数をJIT登録してVMから呼出可能、ログが出る

### 10_b: Lower(Core-1) – Const/Move/BinOp/Cmp/Branch/Ret
- 目的: ループや条件分岐を含む純粋関数をJIT実行可能に。
- 具体:
  - MIR値/型→CLIF型（i64/f64/i1/void）
  - Const/Copy/算術/比較/ジャンプ/分岐/return のLower
  - フレームレイアウト（VMValue最小表現）
- 受入: 算術/比較/分岐のみの関数がJITでVMより速い（小ベンチ）

### 10_c: ABI/呼出し – 関数呼び出しとフォールバック
- 目的: JIT化関数から他関数を呼ぶ/VMへ戻る道を用意。
- 具体:
  - 同一モジュール関数呼び出し（JIT→JIT／JIT→VM）
  - 引数/戻り値の受け渡し（整数/浮動/void）
  - 例外/TypeErrorはVMへバイアウト（trap→VM）
- 受入: 再帰/多段呼び出しが安定動作

### 10_d: コレクション基礎 – Array/Map ホット操作（外部呼び出し）
- 目的: 実用的ホットパス（length/get/set/push/pop）をJIT側から呼べるように。
- 具体:
  - ホスト関数テーブル（外部シンボル）で既存Rust実装を呼ぶ
  - 境界チェック/エラーはRust側に委譲、JITは薄い橋渡し
- 受入: Array操作がVM経由より高速（目安1.5–2.0×）

Status（2025-08-27）
- Param経路でのE2Eを実装（`NYASH_JIT_HOSTCALL=1`ゲート）
- 実装済みシンボル（PoC, C-ABI in Rust）:
  - `nyash.array.{len,push,get,set}` / `nyash.map.size`
- 使い方（例）:
```bash
cargo build --features cranelift-jit --release
NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EXEC=1 \
  ./target/release/nyash --backend vm examples/jit_array_param_call.nyash
NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EXEC=1 \
  ./target/release/nyash --backend vm examples/jit_map_param_call.nyash
```
Notes
- 関数パラメータに渡した配列/MapのみHostCall経由でアクセス（thread-local引数参照）
- ローカルnew値は10_eへ移管（ハンドル表PoC: u64→Arc<Box>）

### 10_e: BoxCall高速化 – Thunk/PICの直結
- 目的: Phase 9.79bの `TypeMeta{slot->thunk}` と Poly-PIC をJITにインライン。
- 具体:
  - `slot -> thunk -> target` 解決をJITで再現（ユニバーサル0..3含む）
  - `(type_id, version)` チェック（Poly-PIC 2–4件）→ヒット直呼び、ミスVM
  - バージョンミスマッチで安全にフォールバック
- 受入: BoxCallホットサイトで2×以上の高速化＋正しい無効化挙動

### 10_f: TypeOp/Ref/Weak/Barrier（最小）
- 目的: 実アプリで必要な最小限のタイプ/参照操作を埋める。
- 具体:
  - `as_bool()` 等の基本型変換
  - 参照/弱参照/バリアの最小パス（重い経路はVMへ）
- 受入: 代表的コードパスでJIT有効のままE2E成功

### 10_g: 診断/ベンチ/回帰
- 目的: 可視化と安定化。
- 具体:
  - `--vm-stats` にJIT統計統合（compile/ms, sites, cache率）
  - ベンチ更新（JIT有/無比較）とスナップショット
- 受入: CIで回帰検知可能／ドキュメント更新

### 10_h: 硬化・パフォーマンス調整
- 目的: ホットスポットの最適化とノイズ除去。
- 具体:
  - ガード配置最適化（分岐予測/ICヒット優先）
  - 不要コピー削減、ホスト呼出回数の削減
- 受入: 代表ベンチで安定して目標達成（2×以上）

## 📦 成果物（各サブフェーズ）
- 10_a: `jit/manager.rs` スケルトン、VM連携、統計ログ
- 10_b: `jit/lower/core.rs`（Const/BinOp/Cmp/Branch/Ret）＋単体テスト
- 10_c: `jit/abi.rs`（call/ret/fallback）＋再帰テスト
- 10_d: `jit/extern/collections.rs`（Array/Mapブリッジ）＋マイクロベンチ
- 10_e: `jit/inline_cache.rs`（PIC/VT連携）＋無効化テスト
- 10_f: `jit/lower/typeop_ref.rs`（最小）
- 10_g: ベンチ/統計/README更新
- 10_h: 最適化コミットと測定レポート

## 🧩 既存資産との連携（重要）
- Thunk: Phase 9.79b.3の `TypeMeta{thunks}` をJIT直呼びターゲットとして使用
- Poly-PIC: VMの構造をJITに投影（同じキー `(label, version)` を使用）
- Versioning: `cache_versions` のイベントに同期してIC無効化

## 🎯 マイルストーン再定義
- M1: 10_a + 10_b 合格（Core関数のJIT実行）
- M2: 10_c + 10_d 合格（関数呼出/Arrayホット操作）
- M3: 10_e 合格（BoxCallホットパス2×）
- M4: 10_g + 10_h 合格（ベンチ/統計/硬化）

## ⚠️ リスクと緩和
- ABI複雑化: まず整数/浮動/voidに限定し、BoxRefはホスト関数へブリッジ
- 最適化過剰: 常にVMフォールバックを保持、ガード失敗で安全に戻す
- デバッグ困難: CLIFダンプ/CFG表示/`NYASH_JIT_STATS`で観測

## 🐛 発見した問題点（2025-08-27 ベンチマーク実行時）

### 1. インタープリター性能問題
- **問題**: 10万回のループで2分以上かかりタイムアウト
- **原因**: `unwrap_instance`のデバッグログが大量出力（毎回の演算でInstanceBoxチェック）
- **目標**: 10万回ループを数秒で完了
- **対策**: 
  - デバッグログの条件付き出力化
  - 基本演算の高速化（IntegerBoxの直接操作最適化）

### 2. VMの変数管理エラー
- **問題**: `Invalid value: Value %47 not set` - simple_add_loop内の変数zが正しく管理されていない
- **原因**: MIR生成時の変数スコープ管理の不具合
- **対策**: MirBuilderの変数トラッキング改善

### 3. Box APIの成熟度
- **問題**: TimeBoxにelapsed()/reset()メソッドがインタープリターから呼べない
- **原因**: Boxメソッドの登録システム未完成
- **対策**: 
  - Boxメソッドの統一的登録システム実装
  - インタープリターとVMのメソッド解決統一

### 4. ベンチマーク基盤
- **問題**: Nyashスクリプトでの正確な時間計測が困難
- **根本原因**: TimeBoxのelapsed()/reset()メソッドがインタープリターから呼べない（Box API問題と同じ）
- **対策**: Box APIの成熟度問題（問題3）が解決すれば自動的に解決

### 改善優先度
1. **高**: インタープリター性能問題（基本機能の使い物にならない）
2. **中**: VM変数管理（JIT最適化の前提）
3. **中**: Box APIの成熟度（ベンチマーク基盤も同時解決）

## 🚀 Phase 10.9: Cranelift AOT Backend（追加計画）

Status: Future (JIT実装の上乗せで実現可能)

### 概要
JIT実装（10_a～10_h）で構築したMIR→CLIF変換基盤をそのまま活用し、事前コンパイル（AOT）によるスタンドアロン実行ファイル生成を実現する。

### 利点
- **コード再利用**: JITと同じLowerCore実装を使用（追加実装最小）
- **非同期完全サポート**: WASMの制限なし、ネイティブ非同期可能
- **最高性能**: ネイティブコード直接実行（起動時コンパイル不要）
- **デバッグ容易**: gdb/lldb等のネイティブデバッガ使用可能

### 実装ステップ案
```
10.9a: ObjectModule統合
├── cranelift-objectモジュール導入
├── CLIF→オブジェクトファイル生成
└── 既存のLowerCore再利用

10.9b: ランタイムライブラリ
├── Nyash標準Box群の静的リンク版作成
├── プラグインの静的埋め込み対応
└── 最小ランタイム（GC hooks等）分離

10.9c: リンカー統合
├── cc/ldによる実行ファイル生成
├── プラットフォーム別設定
└── デバッグシンボル生成

10.9d: クロスコンパイル
├── 複数ターゲット（x86_64/aarch64/wasm32）
├── ターゲット別最適化
└── CI/CDでのマルチプラットフォームビルド
```

### 使用イメージ
```bash
# ネイティブ実行ファイル生成
./target/release/nyash --compile-native program.nyash -o program
./program  # スタンドアロン実行！

# クロスコンパイル
./target/release/nyash --compile-native --target x86_64-pc-windows-msvc program.nyash -o program.exe
./target/release/nyash --compile-native --target aarch64-apple-darwin program.nyash -o program.mac
```

### 技術的詳細
- **共通基盤**: `LowerCore`のemit処理はJIT/AOT両対応
- **差分実装**: JITは`JITModule`、AOTは`ObjectModule`を使用
- **リンク方式**: 静的リンク優先（配布容易性重視）
- **サイズ最適化**: LTO/strip対応で実行ファイルサイズ削減

### WASM AOTとの比較
| 特性 | Cranelift AOT | WASM AOT |
|------|--------------|----------|
| 非同期 | ✅ 完全対応 | ❌ 制限あり |
| 実行速度 | 最速 | 速い |
| ファイルサイズ | 大（MB級） | 小（KB級） |
| ポータビリティ | プラットフォーム別 | 高い |
| デバッグ | ネイティブツール | 限定的 |

### 想定スケジュール
- Phase 10（JIT）安定化後に着手
- 実装期間: 2-3週間（JIT基盤の再利用により短期実現可能）
- 初期ターゲット: Linux x86_64、その後Windows/macOS対応
