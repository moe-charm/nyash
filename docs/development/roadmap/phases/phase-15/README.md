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

## 📊 主要成果物

### コンパイラコンポーネント
- [ ] CompilerBox実装（統合コンパイラ）
- [ ] Nyashパーサー（800行目標）
- [ ] MIR Lowerer（2,500行目標）
- [ ] CraneliftBox（JITエンジンラッパー）
- [ ] LinkerBox（リンカー統合）

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
#### 1. Cranelift（現在の主力）
- **軽量**: 3-5MB程度（LLVMの1/10以下）
- **JIT特化**: メモリ上での動的コンパイル
- **Rust統合**: 静的リンクで配布容易

#### 2. 直接x86エミッタ（革新的アプローチ）
- **dynasm-rs/iced-x86**: Rust内で直接アセンブリ生成
- **テンプレート・スティッチャ方式**: 2-3KBの超小型バイナリ可能
- **完全な制御**: 依存ゼロの究極形

### コード削減の秘密
- **Arc<Mutex>自動化**: 明示的ロック管理不要（-30%）
- **型システム簡略化**: 動的型付けの恩恵（-20%）
- **エラー処理統一**: Result<T,E>地獄からの解放（-15%）
- **動的ディスパッチ**: match文の大幅削減（-10%）

### 実装例
```nyash
// 80,000行のRust実装が20,000行のNyashに！
box NyashCompiler {
    init { parser, lowerer, backend }
    
    compile(source) {
        local ast = me.parser.parse(source)
        local mir = me.lowerer.lower(ast)
        return me.backend.generate(mir)
    }
}

// MIR実行器も動的ディスパッチで簡潔に
box MirExecutor {
    execute(inst) { return me[inst.type](inst) }
    Const(inst) { me.values[inst.result] = inst.value }
    BinOp(inst) { /* 実装 */ }
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

### 段階的アプローチ
1. **Phase 1**: 外部リンカー（lld/gcc）利用
2. **Phase 2**: lld内蔵で配布容易化
3. **Phase 3**: ミニリンカー自作（究極の自立）

### C ABI境界設計
- **プレフィクス**: `ny_v1_*`で統一
- **呼出規約**: Windows(fastcall) / Linux(sysv_amd64)
- **必須関数**: `ny_init()`, `ny_fini()`
- **型マッピング**: `ny_handle=uint64_t`

## 🔗 関連ドキュメント

- [セルフホスティング詳細計画](self-hosting-plan.txt)
- [技術的実装詳細](technical-details.md)
- [Phase 10: Cranelift JIT](../phase-10/)
- [Phase 12.5: 最適化戦略](../phase-12.5/)

## 📅 実施時期

- **開始条件**: Phase 10-14完了後
- **推定開始**: 2026年前半
- **推定期間**: 6-8ヶ月
- **早期着手**: YAML自動生成は今すぐ開始可能

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
- すべてがBox！

**世界一美しい箱は、自分自身さえも美しく包み込む**