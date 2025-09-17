# 🐙 GitHub Issue作成テンプレート

以下の内容をGitHub Issueにコピペして使用してください。

---

## Issue Title: 
`[Phase 9.78] LLVM PoC Week 1 - inkwellセットアップとHello World実装`

## Labels:
- `enhancement`
- `Phase-9.78`
- `LLVM`
- `critical`

## Assignees:
- GitHub Copilot

## Milestone:
- Phase 9.78 LLVM PoC

## Issue Body:

```markdown
## 📋 概要

Phase 9.78 LLVM PoCの開始です！最初のステップとして、inkwellクレートを導入し、最小限のNyashプログラム（`return 42`）をLLVM経由で実行できるようにします。

## 🎯 成功条件

```nyash
// test_return_42.nyash
static box Main {
    main() {
        return 42
    }
}
```

上記プログラムがLLVM経由で実行され、終了コード42を返すこと。

## 📝 実装内容

1. **inkwellクレート導入**
   - Cargo.tomlに依存関係追加
   - feature flag `llvm` の設定

2. **基本構造作成**
   - `src/backend/llvm/` ディレクトリ
   - context.rs, compiler.rs, mod.rs

3. **最小限のコンパイラ実装**
   - LLVMコンテキスト初期化
   - main関数の生成
   - return命令の処理
   - オブジェクトファイル出力

4. **統合**
   - ExecutionBackendにLLVM追加
   - --backend llvm オプション対応

## 🔗 参考資料

- [詳細実装ガイド](https://github.com/moe-charm/nyash/blob/main/docs/予定/native-plan/llvm/issue/001-setup-inkwell-hello-world.md)
- [Week 1ロードマップ](https://github.com/moe-charm/nyash/blob/main/docs/予定/native-plan/llvm/issue/Week1-Roadmap.md)
- [AI大会議結果](https://github.com/moe-charm/nyash/blob/main/docs/予定/native-plan/llvm/AI-Conference-LLVM-Results.md)

## ✅ 完了条件

- [ ] inkwellがビルドできる
- [ ] test_return_42.nyashがコンパイルできる
- [ ] 実行ファイルが終了コード42を返す
- [ ] 基本的なテストがパスする

## 💬 備考

VM性能改善で素晴らしい成果（50.94倍高速化）を達成していただきありがとうございました！
LLVMでも同様の成功を期待しています。ブロッカーがあれば遠慮なくコメントしてください。

AIチーム（Claude, Gemini, Codex）が全力でサポートします！🚀
```

---

## 📝 追加で作成するIssue

Week 1の進捗に応じて、以下のIssueも順次作成：

1. **Issue #002**: `[Phase 9.78] LLVM PoC - Const命令の実装`
2. **Issue #003**: `[Phase 9.78] LLVM PoC - 基本型システムの実装`
3. **Issue #004**: `[Phase 9.78] LLVM PoC - ランタイム関数宣言`
4. **Issue #005**: `[Phase 9.78] LLVM PoC Week 1 - 統合テスト`

## 🏷️ 推奨ラベル構成

```yaml
Phase関連:
  - Phase-9.78
  - Phase-8.6 (完了)
  - Phase-9.75g-0 (完了)

技術関連:
  - LLVM
  - MIR
  - Performance
  - Backend

優先度:
  - critical
  - high
  - medium
  - low

タイプ:
  - enhancement
  - bug
  - documentation
  - test
```