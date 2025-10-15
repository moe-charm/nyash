# セルフホストコンパイラー 重複コードヒートマップ

**視覚的分析レポート** - 2025-10-12

---

## 🔥 重複度ヒートマップ

### 重複スコア (影響ファイル数 × 重複行数)

```
┌──────────────────────────────────────────────────────────────┐
│ 重複パターン                影響度  重複行  スコア  優先度 │
├──────────────────────────────────────────────────────────────┤
│ 🔴 substring操作            30files × 8行  = 240   P0    │
│ 🔴 index_of系               22files × 12行 = 264   P0    │
│ 🔴 数値変換 (to_int/i2s)    22files × 10行 = 220   P0    │
│ 🟠 trim/skip_ws             10files × 10行 = 100   P1    │
│ 🟠 escape/quote             5files  × 15行 = 75    P1    │
│ 🟡 JSON read系              3files  × 40行 = 120   P2    │
│ 🟡 MapBox生アクセス         23files × 3行  = 69    P2    │
│ 🟢 ConsoleBox生成           1file   × 3行  = 3     P3    │
└──────────────────────────────────────────────────────────────┘

凡例: 🔴超高 🟠高 🟡中 🟢低
```

---

## 📊 ファイルサイズ分布と重複密度

```
ファイルサイズ (行数) vs 重複密度 (推定重複行率)

600│ JsonProgramBox (531行, 30%重複)
   │     ┃
500│     ┃
   │     ┃
400│     ┃
   │     ┃
300│     ┃  ParserExprBox (353行, 20%)
   │     ┃        ┃
   │     ┃        ┃
200│     ┃        ┃  MirEmitterBox (230行, 25%)
   │     ┃        ┃        ┃
   │     ┃        ┃        ┃  ParserStmtBox (200行, 20%)
100│     ┃        ┃        ┃        ┃
   │█████████████████████████████████ (他60+ファイル)
  0└───────────────────────────────────────────────────────→
   0%              重複密度                          50%

重複密度 = 他ファイルにも存在する機能の行数割合
```

---

## 🎯 削減ポテンシャル マップ

```
                    実装工数
                      ↑
                 高   │
                      │
         JsonUtilsBox │  StringUtilsBox
              🟠      │      🔴
        (50-80行削減) │ (220-370行削減)
                      │
                      │
      DebugBox 🟢     │  MapHelpersBox 🟡
    (5-10行削減)      │  (50-80行削減)
                      │
                 低   │
            ─────────┼─────────→
                 低  削減効果  高

凡例:
🔴 P0 (超優先): 高効果・低工数
🟠 P1 (優先):   高効果・中工数
🟡 P2 (中):     中効果・中工数
🟢 P3 (低):     低効果・低工数
```

---

## 📈 Everything is Box 準拠度の内訳

```
カテゴリ別準拠度 (0-100%)

ユーティリティ箱化   ████████████░░░░░░░░ 60%
                     ↑ 文字列操作の分散が主因

emit系の箱化        █████████████████░░░ 85%
                     ↑ 良好な設計！

データ構造の箱化     ████████████████░░░░ 80%
                     ↑ MapHelpersBox採用率向上が鍵

デバッグ系の箱化     ██████░░░░░░░░░░░░░░ 30%
                     ↑ DebugBox採用促進が必要

パイプライン統合     ██████████████████░░ 90%
                     ↑ UsingResolverBoxの成功例

─────────────────────────────────────────────
総合スコア           █████████████░░░░░░░ 65%
```

---

## 🔍 重複コード分布マップ

### ディレクトリ別の重複スコア

```
apps/selfhost-compiler/
├── pipeline_v2/ (36files)       重複スコア: ★★★★☆ (高)
│   ├── pipeline.hako (556行)   │ 数値変換: 10箇所
│   ├── using_resolver.hako     │ 文字列操作: 15箇所
│   └── ...                      │ MapBox生アクセス: 20箇所
│
├── boxes/ (20+files)            重複スコア: ★★★★★ (最高)
│   ├── json_program_box.hako   │ 文字列操作: 20箇所
│   │   (531行, 最大)           │ JSON操作: 独自実装大量
│   ├── mir_emitter_box.hako    │ 数値変換: 5箇所
│   └── parser/ (15files)       │ 文字列操作: 合計30箇所
│       ├── scan/ (5files)      │   ↑ 超高重複地帯！
│       ├── expr/ (4files)      │
│       ├── stmt/ (4files)      │
│       └── using/ (2files)     │
│
├── common/ (5files)             重複スコア: ☆☆☆☆☆ (統合済み)
│   ├── *_emit_box.hako (4)     │ ✅ 良好な箱化
│   └── (string_helpers.hakoは別ディレクトリ)
│
└── builder/ (3files)            重複スコア: ★★☆☆☆ (中)
    ├── ssa/local.hako          │ 数値変換: 3箇所
    └── ssa/cond_inserter.hako  │ 文字列操作: 5箇所
```

---

## 🚀 段階的削減計画

### フェーズごとの削減イメージ

```
現在: 5,733行
│
│ Phase 1-Week1: StringUtilsBox統合
├─ 削減: 220-370行
│  ├─ JsonProgramBox: 531 → 450行 (▼81行)
│  ├─ ParserStringUtils: 83 → 20行 (▼63行)
│  ├─ MirEmitterBox: 230 → 200行 (▼30行)
│  └─ 他27files: 合計▼46-196行
│
├─ 5,363-5,513行
│
│ Phase 1-Week2: JsonUtilsBox抽出
├─ 削減: 50-80行
│  ├─ JsonProgramBox: 450 → 330行 (▼120行移動)
│  │  新規JsonUtilsBox: +150行
│  │  純削減: ▼50-80行 (重複排除)
│
├─ 5,283-5,463行
│
│ Phase 2: MapHelpersBox拡張
├─ 削減: 50-80行
│  └─ 23files: nullチェック統一化
│
├─ 5,203-5,413行
│
│ Phase 3: DebugBox改善
└─ 削減: 5-10行
   └─ ConsoleBox再生成削除

最終: 5,193-5,408行
削減率: 5.7-9.4%
```

---

## 🎯 クイックウィン ターゲット

### 即座に着手可能な5ファイル (Phase 1-Week1)

```
優先順位 1: ParserStringUtilsBox (83行)
           ├─ 削減見込み: 63行 (76%削減!)
           ├─ 工数: 2-3時間
           └─ リスク: 低 (parserに閉じている)

優先順位 2: MirEmitterBox (230行)
           ├─ 削減見込み: 30行 (13%削減)
           ├─ 工数: 3-4時間
           └─ リスク: 低 (局所的な変更)

優先順位 3: pipeline_v2/regex_flow.hako (103行)
           ├─ 削減見込み: 15-20行
           ├─ 工数: 2-3時間
           └─ リスク: 低

優先順位 4: builder/ssa/local.hako (122行)
           ├─ 削減見込み: 10-15行
           ├─ 工数: 2-3時間
           └─ リスク: 低

優先順位 5: builder/ssa/cond_inserter.hako (118行)
           ├─ 削減見込み: 10-15行
           ├─ 工数: 2-3時間
           └─ リスク: 低

合計削減見込み: 128-143行 (5ファイル)
合計工数: 11-16時間
```

---

## 📋 重複パターン一覧表

| パターン名 | 代表実装箇所 | 重複箇所数 | 平均行数 | 総重複行数 |
|-----------|-------------|-----------|---------|-----------|
| **substring操作** | JsonProgramBox | 30 | 8 | 240 |
| **index_of** | JsonProgramBox | 22 | 12 | 264 |
| **to_int/i2s** | ParserStringUtils | 22 | 10 | 220 |
| **trim** | JsonProgramBox | 10 | 10 | 100 |
| **escape_string** | JsonProgramBox | 5 | 15 | 75 |
| **read_XXX (JSON)** | JsonProgramBox | 3 | 40 | 120 |
| **MapBox生アクセス** | 各所 | 23 | 3 | 69 |
| **ConsoleBox生成** | DebugBox | 1 | 3 | 3 |
| **総計** | | | | **1,091行** |

※ 総重複行数: 1,091行 (全体の19.0%)
※ StringUtilsBox統合により: 1,091 → 691行 (▼400行削減可能)

---

## 🔬 最大ボトルネック詳細

### JsonProgramBox (531行) の内訳

```
┌─────────────────────────────────────────────────┐
│ JsonProgramBox (531行)                          │
├─────────────────────────────────────────────────┤
│ 正規化ロジック (200行)       ████████████████░░│ 38%
│   ├─ normalize_program                          │
│   ├─ normalize_stmt_array                       │
│   ├─ normalize_expr_array                       │
│   └─ normalize_stmt/expr                        │
├─────────────────────────────────────────────────┤
│ JSON読み取り (150行)         ███████████░░░░░░░│ 28%
│   ├─ read_string/object/array/literal           │
│   ├─ extract_value/extract_string_value         │
│   └─ split_top_level                            │
│   → JsonUtilsBoxに移動候補                      │
├─────────────────────────────────────────────────┤
│ 文字列操作 (100行)           ███████░░░░░░░░░░░│ 19%
│   ├─ index_of/last_index_of/skip_ws/trim        │
│   ├─ escape_string/unescape_string/quote        │
│   └─ i2s                                         │
│   → StringUtilsBoxに移動候補                    │
├─────────────────────────────────────────────────┤
│ メタデータ注入 (50行)        ████░░░░░░░░░░░░░░│ 9%
│   └─ ensure_meta                                │
├─────────────────────────────────────────────────┤
│ その他 (31行)                ██░░░░░░░░░░░░░░░░│ 6%
│   └─ default_XXX_expr, join等                   │
└─────────────────────────────────────────────────┘

リファクタ後の構成:
JsonProgramBox (330行)  - 正規化ロジック + メタデータ注入
JsonUtilsBox (150行)    - JSON読み取りヘルパー (新規)
StringUtilsBox (統合)   - 文字列操作 (既存拡張)
純削減: 50-80行 (重複排除)
```

---

## 📚 参照先クイックリンク

### 詳細分析レポート
- [セルフホストコンパイラー横断的分析](./selfhost-compiler-cross-cutting-analysis.md)

### 関連ドキュメント
- [開発マスタープラン](../roadmap/phases/00_MASTER_ROADMAP.md)
- [Phase 15 INDEX](../roadmap/phases/phase-15/INDEX.md)
- [Box理論](../../reference/language/LANGUAGE_REFERENCE_2025.md)

### 既存良好実装 (参考)
- StringHelpers: `selfhost/shared/common/string_helpers.hako` (86行)
- MapHelpersBox: `selfhost/compiler/pipeline_v2/map_helpers_box.hako` (48行)
- UsingResolverBox: `selfhost/compiler/pipeline_v2/using_resolver_box.hako` (249行)

---

**生成日時**: 2025-10-12
**分析ツール**: Claude Code横断的分析
**信頼度**: 高 (実ファイル読み取りベース)
