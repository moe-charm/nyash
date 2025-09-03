# 論文分担戦略 - MIR13時代の2本構成

Date: 2025-09-03
Status: Active Strategy

## 📊 2本の論文の違い（一目でわかる表）

| 観点 | Paper A（実装の幅） | Paper B（設計の深さ） |
|------|-------------------|-------------------|
| **タイトル** | From Interpreter to Native GUI Apps | The Simple Lifecycle Philosophy |
| **焦点** | 何ができるか（WHAT） | なぜできるか（WHY） |
| **読者** | システム研究者 | 言語設計者 |
| **データ** | 性能・サイズ・実行時間 | 設計メトリクス・学習曲線 |
| **評価** | 5つの実行形態の比較 | API一貫性・拡張性 |
| **投稿先** | ASPLOS, CGO, OSDI | PLDI, OOPSLA, Onward! |
| **ページ数** | 12-14ページ | 10-12ページ |

## 🎯 なぜこの分け方が良いのか

### 1. メッセージの明確化
- **Paper A**: 「13命令でこんなにできる！」（驚き）
- **Paper B**: 「なぜ13で済むのか」（洞察）

### 2. 評価の適切性
- **Paper A**: 定量的評価（ベンチマーク）
- **Paper B**: 定性的評価（設計の美しさ）

### 3. 読者層の最適化
- **Paper A**: 実装したい人向け
- **Paper B**: 設計を学びたい人向け

## 📝 具体的な内容分担

### Paper A に入れるもの
- ✅ MIR13の命令リスト
- ✅ 5つの実行形態の実装詳細
- ✅ 性能ベンチマーク結果
- ✅ 実アプリのスクリーンショット
- ✅ バイナリサイズ比較
- ✅ コンパイル時間測定
- ❌ Box哲学の詳細（軽く触れる程度）
- ❌ ライフサイクル設計論

### Paper B に入れるもの
- ✅ Everything is Box哲学
- ✅ ライフサイクルの3段階
- ✅ なぜ13命令で十分かの説明
- ✅ User/Plugin/Builtinの統一性
- ✅ 学習曲線の分析
- ✅ コード削減率（80k→20k）
- ❌ 詳細な性能データ
- ❌ 実装のテクニカルな話

## 🔄 相互参照の仕方

### Paper A での言及
```
"The simplicity of MIR13 is enabled by the Everything-is-Box 
philosophy, which we explore in depth in [Paper B]."
```

### Paper B での言及
```
"The practical implications of this design are demonstrated 
in [Paper A], where we show implementations ranging from 
interpreters to GUI applications."
```

## 📅 執筆スケジュール

### Phase 1: Paper A 優先（9-10月）
1. Week 1-2: データ収集・ベンチマーク
2. Week 3-4: 執筆・図表作成
3. Week 5: レビュー・推敲

### Phase 2: Paper B 熟成（11-12月）
1. Month 1: 設計分析・メトリクス収集
2. Month 2: 執筆・ケーススタディ

### 理由
- Paper Aは実装済みでデータがある
- Paper Bは使用経験を積んでから書く方が深い

## 💡 書き方のコツ

### Paper A（実装論文）のコツ
1. **デモ重視**: スクリーンショット多用
2. **数値で語る**: グラフ・表を活用
3. **再現性**: ソースコード公開を明記

### Paper B（設計論文）のコツ
1. **図解重視**: 概念図・アーキテクチャ図
2. **例で語る**: 具体的なコード例
3. **比較で語る**: 他言語との違い

## 🎯 最終目標

### 短期（2025年）
- Paper A → ASPLOS/CGO採択
- Paper B → PLDI/OOPSLA投稿

### 中期（2026年）
- 両論文の内容を統合した招待講演
- チュートリアル・ワークショップ開催

### 長期（2027年以降）
- 書籍化："The Nyash Way: Simplicity in Language Design"
- 教育カリキュラムへの採用

## 🌟 成功の鍵

**Paper A**: 実装の**幅広さ**で驚かせる
**Paper B**: 設計の**シンプルさ**で感動させる

2本合わせて、Nyashの革新性を完全に伝える！