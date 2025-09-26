# LaTeX版の準備

## 📝 ファイル一覧

- `paper.tex` - 英語版LaTeX（IEEEtran形式）
- `paper-ja.md` - 日本語版Markdown（PPLなどで使用）

## 🎯 使い方

### 英語版PDF生成
```bash
# 図をPNGに変換
inkscape figures/box-first-architecture-simple.svg \
  --export-filename=figures/box-first-architecture.png --export-dpi=300

# LaTeXコンパイル
pdflatex paper.tex
bibtex paper
pdflatex paper.tex
pdflatex paper.tex
```

### 日本語版PDF生成（LuaLaTeX使用）
```bash
# paper-ja.texを作成（paper.texをベースに）
# - \usepackage{luatexja}を有効化
# - 内容を日本語版に置き換え

lualatex paper-ja.tex
```

## 📊 必要な図

1. `figures/box-first-architecture.png` - アーキテクチャ図
2. `figures/phi_bool_cfg.png` - PHIのCFG図（生成予定）

## 🎯 投稿先別の調整

### VMIL (2-3ページ)
- 現在の長さでOK
- Abstract を150語以内に

### PPL（日本語、10ページ程度）
- paper-ja.mdを拡張
- 実装詳細を追加
- 日本語での説明を充実

### Onward! Essays（5-10ページ）
- AI協調の部分を拡張
- 哲学的考察を追加

## 📚 参考文献（references.bib）

まだ作成していないので、以下を含める予定：
- Lattner2002 (LLVM)
- Wurthinger2013 (Truffle)
- 各種JIT関連論文

## ✅ チェックリスト

- [ ] 図の生成（PNG変換）
- [ ] references.bib作成
- [ ] 著者情報の記入
- [ ] ページ数の調整
- [ ] 投稿規定の確認