# Phase 15: Nyashセルフホスティング - 世界一美しい箱の完成

## 📋 概要

NyashでNyashコンパイラを書く、完全なセルフホスティングの実現フェーズ。
MIR 13命令の美しさを最大限に活かし、外部コンパイラ依存から完全に解放される。
**究極の目標：80,000行→20,000行（75%削減）→ さらなる最適化へ**

## 🔄 2025‑09‑26 Update（方針の明確化）
- 実行系の優先順位: LLVM は Python/llvmlite ハーネスを主経路に固定（llvm_sys 依存は前提にしない）。Rust VM/JIT は保守最小・比較用。
- パーサ: TokenCursor 統一を env ゲート下で進行。Step‑2/3（式＋主要文の薄ラッパ）完了、代表スモーク/パリティは緑。
- PHI: if/else の incoming は「実際の遷移元（exit ブロック）」を使用する規約で統一。dev 検証を追加（pred 重複/自己参照/CFG 包含）。
- ループPHI: `phi_core` に統合（IncompletePhi/スナップショット/exit PHI の責務集約）。`loop_builder.rs` は委譲化で軽量化。
- 次の主タスク: Nyash 製 JSON ライブラリ（JSON v0 DOM: parse/stringify）。完了後に Ny Executor（最小命令）へ直行。
- 既定挙動は不変。新経路はすべて env トグルで opt‑in。

## 🔄 2025‑10‑02 Update（PyVM 撤退・LLVM本流）
- 実行系の既定は Rust VM（MIR）と LLVM（llvmlite ハーネス）。
- PyVM は撤退（既定OFF）。互換確認が必要な場合のみ `--features pyvm-bridge` を明示してビルドし、`NYASH_VM_USE_PY=1` で起動する。
- パリティ検証は LLVM ハーネス基準に一本化。PyVM はローカル限定の互換用途に留める。

推奨トグル
- `NYASH_LLVM_USE_HARNESS=1`（LLVM Python ハーネス）
- `NYASH_PARSER_TOKEN_CURSOR=1`（TokenCursor 経路）
- `NYASH_JSON_PROVIDER=ny`（Ny JSON ライブラリ）
- `NYASH_SELFHOST_EXEC=1`（Ny Executor 最小経路）

## 🎯 フェーズの目的

1. **完全なセルフホスティング**: NyashコンパイラをNyashで実装
2. **外部依存の排除**: gcc/clang/MSVC不要の世界
3. **Everything is Box哲学の完成**: コンパイラもBox
4. **エコシステムの自立**: Nyashだけで完結する開発環境
5. **劇的なコード圧縮**: 75%削減で保守性・可読性の革命

## 🚀 実装戦略（2025年9月更新・改定）

### Self‑Hosting 優先（Phase‑15 基礎固め）
- 目的: Nyash製パーサ/言語機能/Bridge整合/パリティを完成させ、自己ホスト c0→c1→c1' を達成する。
- 運用:
  - Runner から `NYASH_USE_NY_COMPILER=1` を推奨（子プロセス実行→JSON v0→Bridge→MIR 実行）。
  - EXE化は任意の実験導線として維持（配布は Phase‑15 の外）。
  - PyVM は参照実行器として意味論検証に用い、パリティ監視を継続。

### [Historical] Phase 15.2: LLVM（llvmlite）安定化 + PyVM導入
（注）2025‑10‑02 の撤退により PyVM は互換モードのみ。現行は LLVM/Rust VM を主経路に統一。
- JIT/Cranelift は一時停止（古い/非対応）。Rust/inkwell は参照のみ。
- 既定のコンパイル経路は **Python/llvmlite**（harness）のみ
  - MIR(JSON) → LLVM IR → .o → NyRTリンク → EXE
  - Resolver-only / Sealed SSA / 文字列ハンドル不変 を強化
- 新規: **PyVM（Python MIR VM）** を導入し、2本目の実行経路を確保（現在は撤退済み・互換モードのみ）
  - 最小命令: const/binop/compare/phi/branch/jump/ret + 最小 boxcall（Console/File/Path/String）
  - ランナー統合: `NYASH_VM_USE_PY=1` で MIR(JSON) を PyVM に渡して実行
  - 代表スモーク（esc_dirname_smoke / dep_tree_min_string）で llvmlite とパリティ確認

【Current Status — 2025‑09‑14】
- A6 受入達成: esc_dirname_smoke の PyVM↔llvmlite パリティ一致（ゲートOFF）、LLVM verifier green → .o → リンク → 実行OK。
- dep_tree_min_string: PyVM↔llvmlite パリティ一致、llvmlite 経路で `.ll verify → .o → EXE` 完走。
- 一時救済ゲート `NYASH_LLVM_ESC_JSON_FIX` は受入には未使用（OFF）。

#### PHI 取り扱い方針（Phase‑15 中）
- 現行: JSON v0 Bridge 側で If/Loop の PHI を生成（安定・緑）。
- 方針: Phase‑15 ではこのまま完成させる（変更しない）。
- 理由: LoopForm（MIR18）導入時に、逆Loweringで PHI を自動生成する案（推薦）に寄せるため。
  - PHI は「合流点での別名付け」であり、Boxの操作ではない。
  - 抽象レイヤの純度維持（Everything is Box）。
  - 実装責務の一極化（行数削減／保守性向上）。

#### IfForm（構造化 if）— Builder 内部モデル（追加）
- 目的: if/merge を構造化フォームで生成し、PHI‑off/PHI‑on の両経路で安定合流を得る。

### 🚀 **Phase 15.4: MIR Call革新（2025-09-23 NEW）**
**シャドウイングバグからの設計革命 - ChatGPT5 Pro協働成果**

#### 📋 **革新の背景**
- **発端**: PyVM無限ループ問題（ConsoleStd.print内でのprint()再帰呼び出し）
- **発見**: 根本原因は実行時文字列解決によるスコープ曖昧性
- **昇華**: ChatGPT5 Pro提案により表面修正→根本的アーキテクチャ改良へ

#### 🎯 **技術革新内容**
```rust
// ❌ 従来（問題構造）
Call { func: ValueId /* "print"文字列 */, args }

// ✅ 革新後（型付き解決）
enum Callee {
    Global(String),      // nyash.builtin.print
    Method { box_name, method, receiver },
    Value(ValueId),      // 関数値（最小限）
    Extern(String),      // C ABI
}
Call { callee: Callee, args }
```

#### 📈 **Phase 15目標への直接寄与**

1. **80k→20k行削減目標**
   - Phase 1のみ: ~1,500行削減（目標の7.5%）
   - 全Phase完了: ~4,500行削減（目標の22.5%）
   - 実行時解決ロジック・エラー処理・デバッグコードの大幅簡略化

2. **セルフホスティング安定化**
   - using system連携: built-in namespace統合
   - PyVM最適化: 型付き呼び出しによる高速化
   - LLVM最適化: 静的解決による最適化機会拡大

3. **Everything is Box哲学強化**
   - コンパイル時型安全性の確立
   - Box間の呼び出し関係の明確化
   - デバッグ・保守性の劇的向上

#### 🛡️ **実装戦略（3段階・破壊的変更なし）**
- **Phase 1**: 最小変更（2-3日）→即実装可能
- **Phase 2**: HIR導入（1-2週間）→コンパイル時解決確立
- **Phase 3**: 言語仕様統合（1ヶ月）→完全修飾名システム

#### 📊 **成功指標**
- [ ] シャドウイング無限再帰の完全排除
- [ ] 全既存テストの破壊なし（グリーン維持）
- [ ] MIRダンプの可読性向上
- [ ] パフォーマンス向上（実行時オーバーヘッド削減）
- [ ] using systemとの完全統合
- 規約（PHI 合流）:
  - merge 内に copy は置かない。then/else の pred へ edge_copy のみを挿入（self‑copy は No‑Op）。
  - 分岐直前に pre_if_snapshot を取得し、then/else は snapshot ベースで独立構築。merge で snapshot を基底に戻す。
  - 差分検出で“変更された変数のみ”をマージ対象にする。
- LoopForm との合成: ループ body 内に IfForm をネスト。continue は latch、break は after へ分岐（IfForm の merge preds から除外）。
- 検証: スナップショットテストで CFG/edge_copy/終端/分岐先を固定。

### Phase 15.3: NyashコンパイラMVP（次フェーズ着手）
- PyVM 安定後、Nyash製パーサ/レクサ（サブセット）と MIR ビルダを段階導入
- フラグでRustフォールバックと併存（例: `NYASH_USE_NY_COMPILER=1`）
- JIT不要、PyVM/llvmlite のパリティで正しさを担保

【Kickoff 目標（MVP）】
- ステージ1: Ny→JSON v0 パイプ（整数/文字列/加減乗除/括弧/return）。CLI: `--ny-parser-pipe` と互換のJSONを生成。
- ステージ2: 文/式サブセット拡張（local/if/loop/call/method/new/me/substring/length/lastIndexOf）。
- ステージ3: Ny AST→MIR JSON 降下（直接 llvmlite/PyVM へ渡す）。

#### Phase 15.3 — Detailed Plan（Ny compiler MVP）
- Directory layout（selfhost compiler）
  - `apps/selfhost-compiler/compiler.nyash`（CompilerBox entry; Ny→JSON v0 emit）
  - `apps/selfhost-compiler/parser/{lexer.nyash,parser.nyash,ast.nyash}`（Stage‑2 へ段階拡張）
  - `apps/selfhost-compiler/emitter/json_v0.nyash`（将来: emit 分離。MVPは inline でも可）
  - `apps/selfhost-compiler/mir/{builder.nyash,optimizer.nyash}`（将来）
  - `apps/selfhost-compiler/tests/{stage1,stage2}`（サンプルと期待JSON）

- Runner integration（安全ゲート）
  - フラグ: `NYASH_USE_NY_COMPILER=1`（既定OFF）
  - 子プロセス: `--backend vm` で selfhost compiler を起動し、stdout から JSON v0 1行を収集
  - 環境: `NYASH_JSON_ONLY=1` を子に渡して余計な出力を抑制。失敗時は静かにフォールバック

- Stage‑1（小さく積む）
  1) return / 整数 / 文字列 / 四則 / 括弧（左結合）
  2) 文分離（最小ASI）: 改行=文区切り、継続子（+ - * / . ,）やグルーピング中は継続
  3) 代表スモーク: `return 1+2*3` → JSON v0 → Bridge → MIR 実行 = 7

- Stage‑2（本命へ）
  - local / if / loop / call / method / new / var / 比較 / 論理（短絡）
  - PHI: Bridge 側の合流（If/Loop）に依存（Phase‑15中は現行維持）
  - 代表スモーク: nested if / loop 累積 / 短絡 and/or と if/loop の交錯

- Acceptance（15.3）
  - Stage‑1: 代表サンプルで JSON v0 emit → Bridge → VM/llvmlite で一致（差分なし）
  - Bootstrap: `tools/bootstrap_selfhost_smoke.sh` で c0→c1→c1' が PASS（フォールバックは許容）
  - Docs: 文分離ポリシー（改行＋最小ASI）を公開（link: reference/language/statements.md）

- Smokes / Tools（更新）
  - `tools/selfhost_compiler_smoke.sh`（入口）
  - `tools/build_compiler_exe.sh`（Selfhost Parser のEXE化）
  - `tools/ny_stage2_bridge_smoke.sh`（算術/比較/短絡/ネストif）
  - `tools/ny_parser_stage2_phi_smoke.sh`（If/Loop の PHI 合流）
  - `tools/parity.sh --lhs vm --rhs llvmlite <test.nyash>`（常時）

Imports/Namespace plan（15.3‑late）
- See: imports-namespace-plan.md — keep `nyash.toml` resolution in runner; accept `using` in Ny compiler as no‑op (no resolution) gated by `NYASH_USING=1` (compat: `NYASH_ENABLE_USING=1`).

- Operational switches
  - `NYASH_USE_NY_COMPILER=1`（selfhost compiler 経路ON）
  - `NYASH_JSON_ONLY=1`（子プロセスの余計な出力抑止）
- `NYASH_PLUGIN_POLICY=off`（必要に応じて子のみ最小化; compat: `NYASH_DISABLE_PLUGINS=1`）
  - 文分離: 最小ASIルール（深さ0・直前が継続子でない改行のみ終端）

- Risks / Rollback
  - 子プロセス出力がJSONでない→フォールバックで安全運用
  - 代表ケースで parity 不一致→selfhost 経路のみ切替OFF
  - 影響範囲: CLI/Runner 層の限定的変更（ゲートOFFなら既存経路と同値）

【受入（MVP）】
- `tools/ny_roundtrip_smoke.sh` 緑（Case A/B）。
- `apps/tests/esc_dirname_smoke.nyash` / `apps/selfhost/tools/dep_tree_min_string.nyash` を Ny パーサ経路で実行し、PyVM/llvmlite とパリティ一致（stdout/exit）。

#### 予告: LoopForm（MIR18）での PHI 自動化（Phase‑15 後）
- LoopForm を強化し、`loop.begin(loop_carried_values) / loop.iter / loop.branch / loop.end` の構造的情報から逆Loweringで PHI を合成。
- If/短絡についても同様に、構造ブロックから合流点を決めて PHI を自動化。
- スケジュール: Phase‑15 後（MIR18/LoopForm）で検討・実装。Phase‑15 では変更しない。

### Phase 15.4: VM層のNyash化（PyVMからの置換）
- PyVM を足場に、VMコアを Nyash 実装へ段階移植（命令サブセットから）
- 動的ディスパッチで13命令処理を目標に拡張

詳細：[セルフホスティング戦略 2025年9月版](implementation/self-hosting-strategy-2025-09.md)

---

## ♻ 再計画（Macro駆動リファクタ）

強化されたマクロ基盤（AST JSON v0 / PyVMサンドボックス / strict / timeout / golden）を前提に、Phase‑15 のセルフホスティング工程を **「前段AST正規化 → 正式MIR生成」** の二段型にリフレームする。

### ポイント（運用）
- すべてのフロント側変換（構文糖衣・derive相当・軽微なリライト・静的検証）は **MacroBoxSpec**（Nyash/PyVM）で実行（1パス固定点）。
- Rust側は **最小のコア**（パース/AST JSON生成、MIRビルド、バックエンド）に収束。
- 決定性担保: strict=1（既定）、capabilitiesは全OFF（io/net/env=false）の純粋展開。
- 観測: `--dump-expanded-ast-json` と golden 比較で安定性を担保。

### 実行順序（改定）
1) Parse → AST
2) Macro expand（1パス固定点: Built‑in(Rust)→User (Nyash/PyVM)）
3) Using/解決 → MIR → Backend（VM/LLVM/AOT）

### 直近タスクリスト（Phase‑15用）
1. Macro 前段の正式導入（完了／PoC→実運用）
   - `NYASH_MACRO_PATHS=...` でユーザーマクロ登録（ランナールート既定）
   - strict=1/timeout=2000ms（既定）
   - `--dump-expanded-ast-json` を golden として活用
2. Self‑host フロントの簡素化
   - Nyash 製パーサは **最小構文**のみサポート（Stage‑2サブセット）
   - 糖衣や軽いリライトは **MacroBox** に寄せる
3. Golden セットの拡充
   - `apps/tests/*` → `tools/test/golden/*` で展開後JSONの一致
   - timeout/strict/失敗cap の負例テストを追加
4. nyash.toml 設計の雛形（capabilities）
   - `[macros] paths=[…]` + `[macro_caps."path"] io/net/env=false` をドキュメント化

### 受け入れ基準（Phase‑15）
- `--dump-expanded-ast-json` の golden が緑（MacroON/OFF差分は無い／または期待通り）
- PyVM ランナー経由で `MacroBoxSpec.expand(json)` を呼べる（strict/timeout遵守）
- MIR/LLVM/VM の後段は、Macro展開済み入力で恒常緑

関連: [planning/macro_driven_replan.md](planning/macro_driven_replan.md)

---

補足: JSON v0 の扱い（互換）
- Phase‑15: Bridge で PHI を生成（現行継続）。
- MIR18（LoopForm）以降: PHI 自動化後、JSON 側の PHI は非必須（将来は除外方向）。
- 型メタ（“+”の文字列混在／文字列比較）は継続。

## 📊 主要成果物

### コンパイラコンポーネント
- [ ] CompilerBox実装（統合コンパイラ）
- [ ] Nyashパーサー（800行目標）
- [ ] MIR Lowerer（2,500行目標）
- [ ] CraneliftBox（JITエンジンラッパー）
- [ ] LinkerBox（lld内蔵リンカー統合）
- [ ] nyashrtランタイム（静的/動的ライブラリ）
- [ ] ToolchainBox（環境診断・SDK検出）

### 自動生成基盤
- [ ] boxes.yaml（Box型定義）
- [ ] externs.yaml（C ABI境界）
- [ ] semantics.yaml（MIR14定義）
- [ ] build.rs（自動生成システム）

### ブートストラップ
- [ ] c0→c1コンパイル成功
- [ ] c1→c1'自己コンパイル
- [ ] パリティテスト合格

## 🔧 技術的アプローチ

### MIR 14命令の革命
1. Const - 定数
  2. BinOp - 二項演算
  3. UnaryOp - 単項演算（復活！）
  4. Compare - 比較
  5. Jump - 無条件ジャンプ
  6. Branch - 条件分岐
  7. Return - 戻り値
  8. Phi - SSA合流
  9. Call - 関数呼び出し
  10. BoxCall - Box操作（配列/フィールド/メソッド統一！）
  11. ExternCall - 外部呼び出し
  12. TypeOp - 型操作
  13. Safepoint - GC安全点
  14. Barrier - メモリバリア
この究極のシンプルさにより、直接x86変換も現実的に！

### バックエンドの選択肢
#### 1. Cranelift + lld内蔵（保留）
- **軽量**: 3-5MB程度（LLVMの1/10以下）
- **JIT特化**: メモリ上での動的コンパイル
- **Rust統合**: 静的リンクで配布容易
- **lld内蔵**: Windows(lld-link)/Linux(ld.lld)で完全自立
- **C ABIファサード**: `ny_mir_to_obj()`で美しい境界

#### 2. 直接x86エミッタ（将来の革新的アプローチ）
- **dynasm-rs/iced-x86**: Rust内で直接アセンブリ生成
- **テンプレート・スティッチャ方式**: 2-3KBの超小型バイナリ可能
- **完全な制御**: 依存ゼロの究極形

### コード削減の秘密
- **Arc<Mutex>自動化**: 明示的ロック管理不要（-30%）
- **型システム簡略化**: 動的型付けの恩恵（-20%）
- **エラー処理統一**: Result<T,E>地獄からの解放（-15%）
- **動的ディスパッチ**: match文の大幅削減（-10%）
- **合計**: 80,000行→20,000行（75%削減）

### 実装例
```nyash
// 80,000行のRust実装が20,000行のNyashに！
box NyashCompiler {
    parser: ParserBox
    lowerer: LowererBox
    backend: BackendBox
    
    birth() {
        me.parser = new ParserBox()
        me.lowerer = new LowererBox()
        me.backend = new BackendBox()
    }
    
    compile(source) {
        local ast = me.parser.parse(source)
        local mir = me.lowerer.lower(ast)
        return me.backend.generate(mir)
    }
}

// MIR実行器も動的ディスパッチで簡潔に
box MirExecutor {
    values: MapBox
    
    birth() {
        me.values = new MapBox()
    }
    
    execute(inst) { return me[inst.type](inst) }
    Const(inst) { me.values[inst.result] = inst.value }
    BinOp(inst) { /* 実装 */ }
}

// lld内蔵リンカー（ChatGPT5協議）
box LinkerBox {
    platform: PlatformBox
    lld_path: StringBox
    libraries: ArrayBox
    
    birth(platform) {
        me.platform = platform
        me.lld_path = platform.findLldPath()
        me.libraries = new ArrayBox()
    }
    
    link(objects, output) {
        local cmd = me.build_command(objects, output)
        return me.platform.execute(cmd)
    }
}
```

### テンプレート・スティッチャ方式（革新的アプローチ）
```nyash
// 各MIR命令を共通スタブとして実装
box TemplateStitcher {
    init { stubs }
    
    constructor() {
        me.stubs = new MapBox()
        // 各命令の共通実装をスタブとして登録
        me.stubs.set("Const", 0x1000)      // スタブアドレス
        me.stubs.set("BinOp", 0x1100)
        me.stubs.set("BoxCall", 0x1200)
        // ... 13命令分のスタブ
    }
    
    generate(mir) {
        local jumps = new ArrayBox()
        
        // プログラムはスタブ間のジャンプ列に！
        for inst in mir.instructions {
            jumps.push("jmp " + me.stubs.get(inst.type))
        }
        
        return jumps  // 超小型バイナリ！
    }
}
```

## 🔗 EXEファイル生成・リンク戦略

### 統合ツールチェーン（現状）
```bash
nyash build main.ny --backend=llvm --emit exe -o program.exe   # llvmlite/harness 経路
NYASH_VM_USE_PY=1 nyash run main.ny --backend=vm               # PyVM（MIR JSON を実行）
```

### 実装戦略

#### LLVM バックエンド（優先・llvmlite）
1. **MIR→LLVM IR**: MIR13をLLVM IRに変換（✅ 実装済み）
2. **LLVM IR→Object**: ネイティブオブジェクトファイル生成（✅ 実装済み）
3. **Python/llvmlite実装**: Resolver patternでSSA安全性確保（✅ 実証済み）
4. **Object→EXE**: リンカー統合でEXE作成（🚀 実装中）
5. **独立コンパイラ**: `nyash-llvm-compiler` crateとして分離（📝 計画中）

詳細は[**LLVM EXE生成戦略**](implementation/llvm-exe-strategy.md)を参照。

#### Cranelift バックエンド（保留）
1. **MIR→Cranelift**: MIR13をCranelift IRに変換
2. **Cranelift→Object**: ネイティブオブジェクトファイル生成（.o/.obj）
3. **lld内蔵リンク**: lld-link（Win）/ld.lld（Linux）でEXE作成
4. **nyashrtランタイム**: 静的/動的リンク選択可能

### C ABI境界設計
```c
// 最小限の美しいインターフェース
ny_mir_to_obj(mir_bin, target_triple) -> obj_bytes
ny_mir_jit_entry(mir_bin) -> exit_code
ny_free_buf(buffer)
```

詳細は[**自己ホスティングlld戦略**](implementation/lld-strategy.md)を参照。

## 🔗 関連ドキュメント

### 📂 実装関連（implementationフォルダ）
- [🚀 LLVM EXE生成戦略](implementation/llvm-exe-strategy.md)（NEW）
- [🚀 自己ホスティングlld戦略](implementation/lld-strategy.md)（Cranelift版）
- [🧱 箱積み上げ準備メモ](implementation/box-stacking.md)
- [🏗️ アーキテクチャ詳細](implementation/architecture.md)

### 📅 計画関連（planningフォルダ）
- [📋 推奨実装順序](planning/sequence.md)
- [🔧 準備作業まとめ](planning/preparation.md)

### 🔧 実行チェックリスト
- [ROADMAP.md](ROADMAP.md) - 進捗管理用チェックリスト

### ✅ クイックスモーク（現状）
- PyVM↔llvmlite パリティ: `tools/parity.sh --lhs pyvm --rhs llvmlite apps/tests/esc_dirname_smoke.nyash`
- dep_tree（ハーネスON）: `NYASH_LLVM_FEATURE=llvm ./tools/build_llvm.sh apps/selfhost/tools/dep_tree_min_string.nyash -o app_dep && ./app_dep`
- Selfhost Parser EXE: `tools/build_compiler_exe.sh && (cd dist/nyash_compiler && ./nyash_compiler tmp/sample.nyash > sample.json)`
- JSON v0 bridge spec: `docs/reference/ir/json_v0.md`
- Stage‑2 smokes: `tools/ny_stage2_bridge_smoke.sh`, `tools/ny_parser_stage2_phi_smoke.sh`, `tools/ny_me_dummy_smoke.sh`

WSL Quickstart
- See: `docs/guides/exe-first-wsl.md`（依存の導入→Parser EXE バンドル→スモークの順）

### 📚 関連フェーズ
- [Phase 10: Cranelift JIT](../phase-10/)
- [Phase 12.5: 最適化戦略](../phase-12.5/)
- [Phase 12.7: ANCP圧縮](../phase-12.7/)
- [Phase 15.1: AOT計画](phase-15.1/)

## 📅 実施時期（修正版）

- **現在進行中**（2025年9月）
  - Python/llvmlite（既定）／Craneliftは停止
  - PyVM（Python MIR VM）導入・代表スモークで llvmlite とパリティ確認
- **Phase 15.2**: llvmlite安定化 + PyVM最小完成（2025年9-10月）
- **Phase 15.3**: NyashコンパイラMVP（2025年11-12月）
- **Phase 15.4**: VM層Nyash化（2026年1-3月）
- **Phase 15.5**: ABI移行（LLVM完成後、必要に応じて）

## 💡 期待される成果

1. **技術的証明**: 実用言語としての成熟度
2. **開発効率**: Nyashだけで開発完結
3. **教育価値**: 15,000行で読破可能なコンパイラ
4. **コミュニティ**: 参入障壁の大幅低下
5. **保守性革命**: 75%削減で誰でも改造可能

## 🌟 夢の実現

> 「コンパイラもBox、リンカーもBox、すべてがBox」
> 「71,000行→15,000行、これが革命」

外部ツールチェーンに依存しない、真の自立したプログラミング言語へ。

### 数値で見る革命
- **現在**: 80,000行（Rust実装）
- **第一目標**: 20,000行（Nyashセルフホスティング、**75%削減**）
- **究極の夢**: さらなる最適化でより小さく！
- **MIR命令数**: たった13個で全機能実現
- **理解容易性**: 週末で読破可能なコンパイラ
- **バイナリサイズ**: テンプレート方式なら2-3KBも可能
- **教育的価値**: 世界一美しく、世界一小さい実用コンパイラ

### 🌟 Everything is Boxの究極形
- コンパイラもBox
- リンカーもBox  
- アセンブラもBox
- プラグインもBox（.so/.o/.a全方向対応）
- すべてがBox！

**世界一美しい箱は、自分自身さえも美しく包み込む**

### 🚀 次のマイルストーン
- ✅ LLVM dominance違反解決（Resolver pattern）
- 🚀 Python/llvmliteでEXE生成パイプライン完成
- 📝 nyash-llvm-compiler分離設計
- 📝 NyashパーサーMVP実装開始
