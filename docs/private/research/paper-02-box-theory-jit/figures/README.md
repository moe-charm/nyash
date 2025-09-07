# 論文用図表

## 📊 図一覧

### 1. Box-First Architecture
- `box-first-architecture-simple.svg` - シンプル版アーキテクチャ図（新規作成）
- `box_first_architecture.svg` - ChatGPT5作成のオリジナル（Matplotlib生成）

### 2. CFG/PHI可視化（生成予定）
- `phi_bool_cfg.png` - phi_bool_mergeのCFG図
- `mix_promote_cfg.png` - 型昇格のCFG図

### 3. 性能グラフ（作成予定）
- `performance_comparison.png` - VM vs JIT性能比較
- `compile_time_tradeoff.png` - コンパイル時間トレードオフ

## 🎨 図の生成方法

### CFG図の生成
```bash
# 1. DOTファイル生成
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_PHI_MIN=1 \
NYASH_JIT_DOT=tmp/phi_bool.dot \
./target/release/nyash --backend vm examples/phi_bool_merge.nyash

# 2. PNG変換
dot -Tpng tmp/phi_bool.dot -o figures/phi_bool_cfg.png
```

### SVGからPNG変換
```bash
# Inkscapeを使用
inkscape box-first-architecture-simple.svg --export-png=box-first-architecture.png --export-dpi=300
```

## 📝 論文での使用

### Figure 1: Box-First Architecture
```latex
\begin{figure}[h]
\centering
\includegraphics[width=0.8\textwidth]{figures/box-first-architecture.png}
\caption{Box-First JIT Architecture. Three boxes (Configuration, Boundary, 
         Observability) enable safe and reversible JIT implementation.}
\label{fig:architecture}
\end{figure}
```

### Figure 2: Boolean PHI Merge
```latex
\begin{figure}[h]
\centering
\includegraphics[width=0.6\textwidth]{figures/phi_bool_cfg.png}
\caption{CFG visualization of boolean PHI merge with b1 internal path.}
\label{fig:phi-bool}
\end{figure}
```