# Phase 15 推奨進行順（llvmlite+PyVM 優先・自己ホスティング最小）

更新日: 2025-09-05

## 方針（原則）

- JIT/Cranelift は停止。LLVM（llvmlite）と PyVM の2経路で前進。
- 最小自己ホスト体験を早期に成立 → ドキュメント/スモーク/CIを先に固める。
- using（名前空間）はゲート付きで段階導入。NyModulesとny_pluginsの基盤を強化。
- tmux + codex-async を使い、常時2本並走で小粒に積み上げる。

## 推奨シーケンス（概要→実施要点→完了基準）

### 1) 基盤整備（NyModules / ny_plugins / Windows正規化）

**要点:**
- NyModules 共有レジストリ導入: env.modules.set/get（または ModulesBox）
- ny_plugins のパス→名前空間導出: ルート相対、"/"→".", 拡張子 .nyash 省略、[^a-zA-Z0-9_.]→"_"
- Windowsパス: "\\"→"/" 正規化後に上記規則を適用
- 予約衝突: nyashstd.* の登録を明示拒否しログ出力

**スモーク/CI:**
- tools/modules_smoke.sh, tools/modules_winpath_smoke.sh

**完了基準:**
- env.modules.get("acme.logger") などが取得可能、LIST_ONLY/Fail-continue維持、予約拒否ログが出る。

### 2) 最小VM（PyVM）

**要点:**
- MIR(JSON) を Python VM（PyVM）で実行。最小命令 + 最小 boxcall（Console/File/Path/String）
- ランナー統合（`NYASH_VM_USE_PY=1`）→ 代表スモークが llvmlite と一致

**スモーク/CI:**
- tools/compare_harness_on_off.sh（ハーネス）、compare_vm_vs_harness.sh（PyVM vs llvmlite）

**完了基準:**
- esc_dirname_smoke / dep_tree_min_string が PyVM と llvmlite で一致。

【Status 2025‑09‑14】完了（A6 受入）。

### 3) using（ゲート付き）設計・実装（15.2/15.3）

**要点:**
- パーサフック: 'using <ns>' を受理（--enable-using / NYASH_USING=1; compat: NYASH_ENABLE_USING=1）
- リゾルバskeleton: resolve(ns) → NyModules を優先。外部/パッケージは TODO として設計のみ。
- 実行時フック: 未解決時に提案を含む診断。セッションキャッシュを導入（ny_plugins再読込で無効化）。
- using alias: 'using a.b as x' を設計→段階導入。

**スモーク/CI:**
- jit_smoke に using ケースとキャッシュケースを追加。

**完了基準:**
- フラグONで using 経路が動作し、未解決時の診断・キャッシュ挙動がテストで担保。

【Next】Ny パーサMVPと並走で段階導入（フラグ: `--enable-using` / `NYASH_USING=1`；compat: `NYASH_ENABLE_USING=1`）。

### 3.5) Nyash パーサMVP（サブセット）

**要点:**
- ステージ1: Ny→JSON v0 パイプ（最小表現）。
- ステージ2: 文/式のサブセット拡張。
- ステージ3: Ny AST→MIR JSON 直接降下（llvmlite/PyVMへ）。

**スモーク/CI:**
- `tools/ny_roundtrip_smoke.sh` / `tools/ny_parser_bridge_smoke.sh`
- `tools/parity.sh --lhs pyvm --rhs llvmlite <smoke.nyash>`（Nyパーサ経路ON）

**完了基準:**
- esc_dirname_smoke / dep_tree_min_string が Ny パーサ経路でも PyVM/llvmlite と一致（stdout/exit）。

### 4) nyash.link ミニマルリゾルバ（15.4）

**要点:**
- ファイル/相対解決 → 名前空間への写像、検索パス（nyash.toml と環境）、Windows正規化
- 未解決時は候補提示、NyModules へのフォールバック
- using alias + 診断を仕上げる

**スモーク/CI:**
- end-to-end 例（apps/）とJITスモークの拡充

**完了基準:**
- 小規模プロジェクトで using + nyash.link の基本導線がJITでE2E通る。

### 5) パフォーマンス守り（MIRマイクロ最適化 + 回帰ゲート）

**要点:**
- const-fold（リテラル・単純四則）、DCE（unreachable return/blocks）をスコープ限定で有効化
- 回帰時は NYASH_CLI_VERBOSE=1 で診断を落とす

**スモーク/CI:**
- jit_smoke に閾値付きケースを追加、CI optional stage で監視

**完了基準:**
- 主要ケースで回帰検出が機能し、JITパリティが維持される。

### 6) Boxes 高レベル移植（15.5 開始）

**要点:**
- StringBox → ArrayBox の順で表層メソッドをNyashへ移植（NyRTは最小プリミティブ維持）
- MapBox は次段で検討。ABI/churnを避けるため段階導入

**スモーク/CI:**
- 文字列/配列操作のJITスモークを追加

**完了基準:**
- 代表的な文字列/配列APIがNyash実装で安定動作し、CI常時緑。

### 7) インタープリターコアの段階移植（15.5/15.6）

**要点:**
- MIR実行ループを段階的にNyash化（動的ディスパッチで13命令処理）
- ブートストラップ: c0(Rust) → c1(Nyash) → c1'（自己再コンパイル）

**検証:**
- パリティテスト（trace_hash 等）とスモークを追加

**完了基準:**
- 自己再コンパイルループが成立し、差分トレースが安定。

### 8) YAML 自動生成（15.1 を後段にスライドして導入）

**要点:**
- boxes.yaml / externs.yaml / semantics.yaml を定義し、build.rs でコード自動生成
- まず externs/boxes の一部から段階導入 → 重複削減を早期に回収

**完了基準:**
- 重複コードが実測で大幅削減（1〜2万行級）、CI・ドキュメントに反映。

### 9) クローズアウト（各小節の都度）

- README.ja.md / AGENTS.md / docs のHOWTO・旗一覧・スモーク手順を常に最新化
- ツール類索引: tools/jit_smoke.sh, selfhost_vm_smoke.sh, modules_smoke.sh, modules_winpath_smoke.sh
- CIトグル整備: LLVM系は無効化、JIT（--features cranelift-jit）を標準経路に

## クイックコマンド（JITオンリー）

```bash
# ビルド
cargo build --release --features cranelift-jit

# 実行
./target/release/nyash --backend vm apps/selfhost-minimal/main.nyash

# スモーク
./tools/jit_smoke.sh
./tools/selfhost_vm_smoke.sh
./tools/modules_smoke.sh ; ./tools/modules_winpath_smoke.sh
```

## フラグ（抜粋）

- `--load-ny-plugins` / `NYASH_LOAD_NY_PLUGINS=1`
- `--enable-using` / `NYASH_USING=1`（compat: `NYASH_ENABLE_USING=1`）
- `NYASH_CLI_VERBOSE=1`（診断強化）

## 運用（Codex async / tmux）

- 2並走・重複回避: `CODEX_MAX_CONCURRENT=2 CODEX_DEDUP=1 CODEX_ASYNC_DETACH=1`
- 監視: `pgrep -af 'codex .* exec'` / `tail -f ~/.codex-async-work/logs/codex-*.log`
- Windowsパス/名前空間: "\\"→"/" 正規化 → ルール適用（/→., .nyash除去, sanitize）

## 備考

本シーケンスは `docs/development/roadmap/phases/phase-15/self-hosting-plan.txt` を尊重しつつ、JIT最小体験を優先させるため順序を最適化（LLVM/lld と YAML自動生成は後段へスライド）。進捗に応じて適宜見直し、CI/スモークで常時検証する。
