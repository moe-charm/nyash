# MIR13時代の論文戦略 - 2本構成の提案

Date: 2025-09-03
Author: Claude + にゃ

## 📚 2本の論文に分ける理由

### なぜ2本が良いか
1. **読者層が違う** - システム屋 vs 言語設計者
2. **評価軸が違う** - 実装の幅広さ vs 設計の美しさ
3. **投稿先が違う** - システム系会議 vs プログラミング言語系会議

## 🎯 論文A: 実装の幅広さ（システム寄り）

### タイトル案
**"From Interpreter to Native GUI Apps: Universal Execution with 13 Instructions"**

### 日本語仮題
「13命令で作る万能実行系：インタープリターからネイティブGUIアプリまで」

### ストーリー
```
MIR13という超コンパクトIR
    ↓
インタープリター実装（開発用）
    ↓
VM実装（高速実行）
    ↓
JIT/AOT（Cranelift統合）
    ↓
ネイティブEXE生成（lld内蔵）
    ↓
Windows GUIアプリも動く！（EguiBox）
    ↓
「たった13命令ですべて実現」
```

### 主な貢献
1. **実装の完全性**: 1つのIRで5つの実行形態
2. **実用性の証明**: 実際のGUIアプリが動作
3. **統合ツールチェーン**: 外部依存なしで完結

### データ・評価
- 各バックエンドの性能比較
- バイナリサイズ比較
- 起動時間・メモリ使用量
- GUIアプリのレスポンス性

### 投稿先
- **ASPLOS**: システム×言語の交差点
- **OSDI/SOSP**: システム実装の革新性
- **CGO**: コード生成の多様性

## 🌟 論文B: 設計の美しさ（言語設計寄り）

### タイトル案
**"The Simple Lifecycle Philosophy: How Everything-is-Box Enables Minimal IR Design"**

### 日本語仮題
「シンプルライフサイクル哲学：Everything is BoxがもたらすミニマルIR設計」

### ストーリー
```
Everything is Box哲学
    ↓
統一的オブジェクトモデル
    ↓
ライフサイクルの単純化
    ↓
だからMIR13で十分
    ↓
User Box = Plugin Box = Builtin Box
    ↓
「美しい対称性」
```

### 主な貢献
1. **設計哲学**: Box統一による簡潔性
2. **ライフサイクル**: 生成・使用・解放の一貫性
3. **拡張性**: プラグインも同じ仕組み

### データ・評価
- コード行数の削減率（80k→20k）
- APIの一貫性メトリクス
- 学習曲線（初心者の理解速度）
- 拡張性の実証（プラグイン数）

### 投稿先
- **PLDI**: プログラミング言語設計の最高峰
- **OOPSLA**: オブジェクト指向設計
- **Onward!**: 新しい考え方の提案

## 📊 既存のMIR15論文の活用方法

### Paper Aへの統合
```
旧: MIR15の設計詳細
↓
新: MIR13への進化 + 実装の幅広さ
```
- Introduction で MIR15→13 の改善に軽く触れる
- メインは「13命令で何ができるか」の実証
- 評価は実装の多様性に焦点

### Paper Bでの位置づけ
```
旧: 命令削減のテクニック
↓
新: なぜ13で十分なのかの哲学的説明
```
- Box統一があるから13で済む
- ライフサイクルが単純だから13で済む
- 設計思想の一貫性を強調

## 🎨 具体的な分担案

### Paper A の章構成
1. Introduction: 多様な実行形態の課題
2. MIR13 Overview: 13命令の紹介
3. Implementation:
   - Interpreter（500行）
   - VM（1000行）
   - JIT/AOT（Cranelift統合）
   - Native EXE（lld内蔵）
   - GUI Apps（EguiBox）
4. Evaluation: 性能・サイズ・実用性
5. Related Work: 他言語の実行モデル
6. Conclusion: 統一IRの価値

### Paper B の章構成
1. Introduction: 複雑性の問題
2. Box Philosophy: Everything is Box
3. Lifecycle Design:
   - Creation（new/birth）
   - Usage（method calls）
   - Destruction（自動管理）
4. MIR13 Minimalism: なぜ13で十分か
5. Case Studies:
   - User-defined Boxes
   - Plugin Boxes
   - Builtin Boxes
6. Discussion: シンプルさの価値
7. Conclusion: 新しい設計パラダイム

## 💡 実践的アドバイス

### まず書くべきは Paper A
**理由**：
1. **データが揃っている** - すでに実装済み
2. **インパクトが明確** - 「13命令で全部できる」
3. **デモが強力** - GUIアプリの動画
4. **差別化しやすい** - 他にない実績

### Paper B は少し寝かせる
**理由**：
1. **哲学は熟成が必要** - 使用経験を積む
2. **事例を集める** - コミュニティフィードバック
3. **Paper Aの反応を見る** - 方向性の確認

## 🚀 アクションプラン

### Step 1: MIR13への更新（1週間）
- [ ] 既存のMIR15ドキュメントを13に更新
- [ ] 削減の経緯を簡潔にまとめる
- [ ] ベンチマークを13命令で再実行

### Step 2: Paper A 執筆（3週間）
- [ ] Windows GUIアプリのスクリーンショット/動画
- [ ] 各バックエンドの性能データ収集
- [ ] 実装のLOCカウント
- [ ] 図表の作成

### Step 3: Paper B 準備（同時進行）
- [ ] Box哲学の整理
- [ ] ライフサイクルの図解
- [ ] ユーザー事例の収集

## 🎯 結論

**2本に分けるのは大正解！**

- **Paper A**: 「こんなにできる」（HOW）- システム実装の広さ
- **Paper B**: 「なぜできる」（WHY）- 設計哲学の深さ

論文の世界では、1つの論文に1つの明確なメッセージが重要。2本に分けることで、それぞれのメッセージが明確になり、より多くの読者に届きますにゃ！