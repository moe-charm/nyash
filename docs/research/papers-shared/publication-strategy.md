# 📚 Nyash論文の出版戦略

## 🎯 推奨ルート（段階的アプローチ）

### Phase 1: プレプリント公開（1ヶ月以内）
1. **arXiv投稿**
   - カテゴリ: cs.PL (Programming Languages)
   - 必要なもの：.edu メールアドレス（なければendorserを探す）
   - 利点：即座に世界公開、引用可能、フィードバック獲得

2. **GitHub公開**
   - リポジトリ内に`papers/`フォルダ作成
   - PDFとソースコード同梱
   - Twitterで宣伝

### Phase 2: カジュアル発表（3ヶ月以内）
1. **PPL 2026**（2026年3月）
   - 締切：通常12月
   - 10ページ程度でOK
   - 日本語可能

2. **JSSST 2025**（日本ソフトウェア科学会大会）
   - 締切：通常6月
   - ポスター発表もあり

### Phase 3: 国際会議挑戦（6ヶ月〜1年）
1. **Onward! 2025** (SPLASH併設)
   - 締切：通常4月
   - 新しいアイデア大歓迎
   - 採択率高め（30-40%）

2. **Programming 2026**
   - 締切：通常10月
   - 建設的査読プロセス
   - オープンアクセス

### Phase 4: トップ会議（1年以上）
- 実装とデータを充実させてから
- PLDI or OOPSLA

## 💰 費用の現実

### 無料〜安価
- arXiv: 完全無料
- GitHub: 完全無料
- 国内研究会: 5,000〜10,000円（非会員）

### 高額
- 国際会議参加: 10〜50万円（渡航費込み）
- ただし**論文投稿自体は無料**（採択後の登録料は必要）

## 🔧 実践的アドバイス

### 1. まずarXivに出す
```bash
# 論文をPDF化
pdflatex paper.tex

# arXivにアップロード
# タイトル例：
"Box-Oriented JIT: A Fault-Tolerant Architecture 
 for Language Runtime Construction"
```

### 2. Twitterで宣伝
```
New paper: "Box-Oriented JIT" - a simple yet powerful
approach to building fault-tolerant language runtimes.

We implemented it in Nyash and achieved 100% panic recovery!

Paper: arxiv.org/abs/xxxx.xxxxx
Code: github.com/nyash/jit

#PLDesign #JIT #Nyash
```

### 3. フィードバックを集める
- Reddit r/ProgrammingLanguages
- Hacker News
- 日本なら Qiita/Zenn で解説記事

### 4. 改善して会議投稿
- フィードバックを反映
- データを追加
- 適切な会議を選んで投稿

## 🎯 なぜこの順序か

1. **arXiv** = 優先権確保、世界に公開
2. **国内発表** = 練習、日本語で議論
3. **Onward!** = 国際デビュー、新アイデア歓迎
4. **PLDI/OOPSLA** = 最終目標、キャリアに箔

## 📌 重要な真実

**会員にならなくても論文は出せる！**

- 国際会議は投稿無料
- arXivは完全無料
- 採択されてから参加を考えればOK

**まず書いて、出して、反応を見る** - これが一番大事にゃ！🐱📝