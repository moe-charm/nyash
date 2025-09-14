# Phase 15: Nyashセルフホスティング - 世界一美しい箱の完成

## 📋 概要

NyashでNyashコンパイラを書く、完全なセルフホスティングの実現フェーズ。
MIR 13命令の美しさを最大限に活かし、外部コンパイラ依存から完全に解放される。
**究極の目標：80,000行→20,000行（75%削減）→ さらなる最適化へ**

## 🎯 フェーズの目的

1. **完全なセルフホスティング**: NyashコンパイラをNyashで実装
2. **外部依存の排除**: gcc/clang/MSVC不要の世界
3. **Everything is Box哲学の完成**: コンパイラもBox
4. **エコシステムの自立**: Nyashだけで完結する開発環境
5. **劇的なコード圧縮**: 75%削減で保守性・可読性の革命

## 🚀 実装戦略（2025年9月更新・改定）

### Phase 15.2: LLVM（llvmlite）安定化 + PyVM導入
- JIT/Cranelift は一時停止（古い/非対応）。Rust/inkwell は参照のみ。
- 既定のコンパイル経路は **Python/llvmlite**（harness）のみ
  - MIR(JSON) → LLVM IR → .o → NyRTリンク → EXE
  - Resolver-only / Sealed SSA / 文字列ハンドル不変 を強化
- 新規: **PyVM（Python MIR VM）** を導入し、2本目の実行経路を確保
  - 最小命令: const/binop/compare/phi/branch/jump/ret + 最小 boxcall（Console/File/Path/String）
  - ランナー統合: `NYASH_VM_USE_PY=1` で MIR(JSON) を PyVM に渡して実行
  - 代表スモーク（esc_dirname_smoke / dep_tree_min_string）で llvmlite とパリティ確認

【Current Status — 2025‑09‑14】
- A6 受入達成: esc_dirname_smoke の PyVM↔llvmlite パリティ一致（ゲートOFF）、LLVM verifier green → .o → リンク → 実行OK。
- dep_tree_min_string: PyVM↔llvmlite パリティ一致、llvmlite 経路で `.ll verify → .o → EXE` 完走。
- 一時救済ゲート `NYASH_LLVM_ESC_JSON_FIX` は受入には未使用（OFF）。

### Phase 15.3: NyashコンパイラMVP（次フェーズ着手）
- PyVM 安定後、Nyash製パーサ/レクサ（サブセット）と MIR ビルダを段階導入
- フラグでRustフォールバックと併存（例: `NYASH_USE_NY_COMPILER=1`）
- JIT不要、PyVM/llvmlite のパリティで正しさを担保

【Kickoff 目標（MVP）】
- ステージ1: Ny→JSON v0 パイプ（整数/文字列/加減乗除/括弧/return）。CLI: `--ny-parser-pipe` と互換のJSONを生成。
- ステージ2: 文/式サブセット拡張（local/if/loop/call/method/new/me/substring/length/lastIndexOf）。
- ステージ3: Ny AST→MIR JSON 降下（直接 llvmlite/PyVM へ渡す）。

【受入（MVP）】
- `tools/ny_roundtrip_smoke.sh` 緑（Case A/B）。
- `apps/tests/esc_dirname_smoke.nyash` / `apps/selfhost/tools/dep_tree_min_string.nyash` を Ny パーサ経路で実行し、PyVM/llvmlite とパリティ一致（stdout/exit）。

### Phase 15.4: VM層のNyash化（PyVMからの置換）
- PyVM を足場に、VMコアを Nyash 実装へ段階移植（命令サブセットから）
- 動的ディスパッチで13命令処理を目標に拡張

詳細：[セルフホスティング戦略 2025年9月版](implementation/self-hosting-strategy-2025-09.md)

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
- [ ] semantics.yaml（MIR15定義）
- [ ] build.rs（自動生成システム）

### ブートストラップ
- [ ] c0→c1コンパイル成功
- [ ] c1→c1'自己コンパイル
- [ ] パリティテスト合格

## 🔧 技術的アプローチ

### MIR 13命令の革命
- **基本演算(5)**: Const, UnaryOp, BinOp, Compare, TypeOp
- **メモリ(2)**: Load, Store
- **制御(4)**: Branch, Jump, Return, Phi
- **Box(1)**: BoxCall（すべての箱操作を統合）
- **外部(1)**: ExternCall

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
