# 図表ディレクトリ

このディレクトリには、AI二重化開発モデルを説明する図表を配置します。

## 必要な図表

### 1. ai_dual_mode_flow.svg
AI二重化モデルのフロー図

```
┌─────────────┐     ┌─────────────┐
│  俯瞰AI     │     │  実装AI     │
│ (Architect) │     │(Implementer)│
└─────┬───────┘     └─────┬───────┘
      │                    │
      ├────────┬───────────┤
              ▼
        ┌─────────────┐
        │   人間      │
        │(Integrator)│
        └─────────────┘
```

要素：
- 各AIの役割と入出力
- 人間の統合判断ポイント
- フィードバックループ

### 2. development_speed.svg
開発速度の比較グラフ

```
時間（時間）
10 ┤ ■■■■■■■■■■ 従来モデル
 8 ┤ ■■■■■■■■
 6 ┤ ■■■■■■
 4 ┤ ■■■■
 2 ┤ ■■
 0 ┤ ▬ AI二重化モデル（0.3時間）
   └─────────────────────
```

### 3. problem_solving_flow.svg
問題解決フローの可視化

```
観測: argc==0
    ↓
俯瞰AI: MIR引数配線が問題
    ↓
実装AI: BoxCallにargs追加
    ↓
検証: argc==1 ✓
```

### 4. box_hierarchy.svg
箱理論の階層構造

```
Everything is Box
├─ データ箱
│   ├─ StringBox
│   ├─ IntegerBox
│   └─ MathBox
├─ 機能箱
│   ├─ JitConfigBox
│   └─ JitEventsBox
└─ AI役割箱
    ├─ 俯瞰Box
    └─ 実装Box
```

### 5. observable_metrics.svg
観測可能性のメトリクス

```
{
  "event": "hostcall",
  "argc": 0,        ← 問題の指標
  "method": "sin",
  "decision": "sig_mismatch"
}
```

### 6. knowledge_creation_rate.svg
知識創造速度のグラフ

```
論文ネタ/日
5 ┤ ★ AI二重化モデル
4 ┤ ★
3 ┤ ★
2 ┤ ★
1 ┤ ★ ● 従来モデル
0 └─────────────────
```

## 作成方法

これらの図表は以下のツールで作成可能：
- draw.io / diagrams.net（フロー図）
- matplotlib / plotly（グラフ）
- graphviz（階層構造図）

## 配色ガイド

- 俯瞰AI: 青系（#2196F3）
- 実装AI: 緑系（#4CAF50）
- 人間: オレンジ系（#FF9800）
- 問題: 赤系（#F44336）
- 解決: 緑系（#8BC34A）