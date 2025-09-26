# Awesome Rust掲載準備

Date: 2025-08-31
Status: In Progress

## 🎯 目的
Nyashプロジェクトを[Awesome Rust](https://github.com/rust-unofficial/awesome-rust)リストに掲載し、Rustコミュニティへの認知度を向上させる。

## 📋 掲載カテゴリー候補

### 1. Development tools > Build system
- Nyashの統合ビルドシステム（インタープリター/VM/WASM/AOT）

### 2. Programming languages
- **Nyash - Everything is Box プログラミング言語** ← 最有力候補
- Rust製の新しいプログラミング言語実装として

### 3. Virtual machines
- NyashのVM実装（MIR15命令セット）

## 📝 提出文案

### オプション1（シンプル版）
```markdown
* [Nyash](https://github.com/[user]/nyash) — A Box-oriented programming language with VM/JIT/AOT backends. Everything is Box philosophy with 15-instruction MIR.
```

### オプション2（詳細版）
```markdown
* [Nyash](https://github.com/[user]/nyash) [[nyash](https://crates.io/crates/nyash)] — Everything is Box programming language featuring unified object model, multi-backend execution (Interpreter/VM/WASM/AOT), and revolutionary 15-instruction MIR design. Built for P2P mesh networking and distributed computing.
```

### オプション3（技術重視版）
```markdown
* [Nyash](https://github.com/[user]/nyash) — Modern programming language with Box-based unified type system, featuring high-performance VM with JIT compilation, WASM target, and upcoming LLVM backend. Designed for simplicity without sacrificing performance.
```

## ✅ 掲載前チェックリスト

### 必須項目
- [ ] GitHubリポジトリが公開されている
- [ ] READMEが充実している（英語）
- [ ] ライセンスが明記されている
- [ ] ビルド手順が明確
- [ ] 基本的な使用例がある

### 推奨項目
- [ ] CIが設定されている（GitHub Actions等）
- [ ] ドキュメントが整備されている
- [ ] サンプルプログラムがある
- [ ] crates.ioに公開されている
- [ ] バージョン1.0以上（または明確なロードマップ）

## 🚀 提出手順

1. **リポジトリ準備**
   - README.mdを英語化/改善
   - サンプルコードを追加
   - CI/CDを設定

2. **PR作成**
   - Awesome Rustをfork
   - 適切なセクションに追加
   - アルファベット順を守る
   - PRテンプレートに従う

3. **フォローアップ**
   - レビューコメントに対応
   - 必要に応じて説明追加

## 📊 現在の準備状況

### ✅ 完了
- 基本的な言語実装
- VM実装（13.5倍高速化達成）
- MIR設計（15命令に削減）
- ドキュメント構造

### 🚧 作業中
- README.mdの英語化
- サンプルプログラムの整理
- CI/CDの設定

### ❌ 未着手
- crates.io公開
- ロゴ/ブランディング
- Webサイト

## 🎨 プロジェクト説明の改善案

### 現在のREADME冒頭
```
Nyashプログラミング言語 - Everything is Box
```

### 改善案（英語版）
```markdown
# Nyash Programming Language

A modern programming language where Everything is Box - unified object model with high-performance execution.

## Features
- 🎁 **Everything is Box**: Unified object model for all values
- ⚡ **Multi-backend**: Interpreter, VM (13.5x faster), WASM, AOT
- 🚀 **15-instruction MIR**: Revolutionary minimal instruction set
- 🔧 **Plugin System**: Extensible architecture
- 🌐 **P2P Ready**: Built for distributed computing

## Quick Start
```nyash
// Everything is a Box
local greeting = new StringBox("Hello, Nyash!")
print(greeting)

// User-defined Boxes
box Person {
    init { name, age }
    
    birth(name) {
        me.name = name
        me.age = 0
    }
}

local alice = new Person("Alice")
```
```

## 📅 タイムライン

### Phase 1（現在）
- README改善
- サンプル整理
- 基本的なCI設定

### Phase 2（LLVM実装後）
- crates.io公開
- 正式なv1.0リリース
- Awesome Rust提出

### Phase 3（採用後）
- コミュニティフィードバック対応
- ドキュメント拡充
- エコシステム構築

## 🔗 関連リンク
- [Awesome Rust](https://github.com/rust-unofficial/awesome-rust)
- [提出ガイドライン](https://github.com/rust-unofficial/awesome-rust/blob/main/CONTRIBUTING.md)
- [他の言語実装例](https://github.com/rust-unofficial/awesome-rust#programming-languages)