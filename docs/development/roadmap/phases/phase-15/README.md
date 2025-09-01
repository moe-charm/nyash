# Phase 15: Nyashセルフホスティング - 究極の目標

## 📋 概要

NyashでNyashコンパイラを書く、完全なセルフホスティングの実現フェーズ。
内蔵Cranelift JITを活用し、外部コンパイラ依存から完全に解放される。

## 🎯 フェーズの目的

1. **完全なセルフホスティング**: NyashコンパイラをNyashで実装
2. **外部依存の排除**: gcc/clang/MSVC不要の世界
3. **Everything is Box哲学の完成**: コンパイラもBox
4. **エコシステムの自立**: Nyashだけで完結する開発環境

## 📊 主要成果物

- [ ] CompilerBox実装（Nyashコンパイラ）
- [ ] Nyashパーサー（Nyash実装）
- [ ] MIR Lowerer（Nyash実装）
- [ ] CraneliftBox（JITエンジンラッパー）
- [ ] ブートストラップ成功

## 🔧 技術的アプローチ

### 内蔵Craneliftの利点
- **軽量**: 3-5MB程度（LLVMの1/10以下）
- **JIT特化**: メモリ上での動的コンパイル
- **Rust統合**: 静的リンクで配布容易

### 実装例
```nyash
box NyashCompiler {
    init { cranelift }
    
    compile(source) {
        local ast = me.parse(source)
        local mir = me.lower(ast)
        local code = me.cranelift.compile(mir)
        return code
    }
}

// 使用例
local compiler = new CompilerBox()
local program = compiler.compile("print('Hello, Self-hosted Nyash!')")
program.run()
```

## 🔗 関連ドキュメント

- [セルフホスティング詳細計画](self-hosting-plan.txt)
- [Phase 10: Cranelift JIT](../phase-10/)
- [Phase 12.5: 最適化戦略](../phase-12.5/)

## 📅 実施時期

- **開始条件**: Phase 10-14完了後
- **推定開始**: 2026年前半
- **推定期間**: 6-8ヶ月

## 💡 期待される成果

1. **技術的証明**: 実用言語としての成熟度
2. **開発効率**: Nyashだけで開発完結
3. **教育価値**: シンプルなコンパイラ実装例
4. **コミュニティ**: 参入障壁の大幅低下

## 🌟 夢の実現

> 「コンパイラもBox、すべてがBox」

外部ツールチェーンに依存しない、真の自立したプログラミング言語へ。