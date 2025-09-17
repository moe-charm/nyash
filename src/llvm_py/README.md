# LLVM Python Backend (Experimental)

## 📝 概要
Rust/inkwellの複雑性を回避し、llvmliteを使ってシンプルに実装する実験的バックエンド。
ChatGPTが設計した`docs/design/LLVM_LAYER_OVERVIEW.md`の設計原則に従う。

## 🎯 目的
1. **検証ハーネス** - PHI/SSA構造の高速検証
2. **プロトタイプ** - 新機能の迅速な試作
3. **教育的価値** - シンプルで理解しやすい実装
4. **バックアップ** - Rustが詰まった時の代替案

## 📂 構造
```
llvm_py/
├── README.md          # このファイル
├── mir_reader.py      # MIR JSON読み込み
├── llvm_builder.py    # メインのLLVM IR生成
├── resolver.py        # Resolver API（Python版）
├── types.py          # 型変換ユーティリティ
└── test_simple.py    # 基本テスト
```

## 🚀 使い方
```bash
# MIR JSONからオブジェクトファイル生成
python src/llvm_py/llvm_builder.py input.mir.json -o output.o

# 環境変数で切り替え（将来）
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash program.nyash
```

## 📋 設計原則（LLVM_LAYER_OVERVIEWに準拠）
1. **Resolver-only reads** - 直接vmapアクセス禁止
2. **Localize at block start** - BB先頭でPHI生成
3. **Sealed SSA** - snapshot経由の配線
4. **BuilderCursor相当** - 挿入位置の厳格管理

## 🎨 実装状況
- [ ] 基本構造（MIR読み込み）
- [ ] Core-14命令の実装
- [ ] Resolver API
- [ ] LoopForm対応
- [ ] テストスイート

## 📊 予想行数
- 全体: 800-1000行
- コア実装: 300-400行

「簡単最高」の精神を体現！
