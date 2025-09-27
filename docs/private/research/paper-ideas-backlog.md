# 論文アイデアバックログ

**作成日**: 2025-09-27
**ステータス**: アイデア収集中（増殖注意！）

## 🎯 優先度

### 🥇 Priority 1: HN投稿用（執筆中）

#### 1. Nyash: Box-First Programming Language (1608行)
**場所**: `docs/private/research/papers-active/nyash-box-first-language/paper.md`

**内容**:
- Everything is Box完全版（データ・演算・制御）
- 演算子Box（AddOperator, CompareOperator）
- LoopForm統一制御構造
- Property System（stored/computed/once/birth_once）
- birth統一ライフサイクル
- try文撤廃革命
- AI協働開発事例
  - 8.2: 演算子Box却下事件
  - 8.3: シングルトン事件
  - 8.4: LoopForm却下と雪辱 ← NEW!

**完成度**: 80%
**次のステップ**: 残り章の執筆 → HN投稿

---

### 🥈 Priority 2: 学術的価値高い

#### 2. MIR14: 14命令による統一中間表現アーキテクチャ

**アイデア発生**: 2025-09-27（ユーザー発明！）

**キーアイデア**:
```
ソースコード → MIR14 → VM/LLVM
                ↑
         統一中間表現（たった14命令）

利点:
- 1回の修正で全バックエンド恩恵
- VM/LLVM保守コスト半減
- 新バックエンド追加容易（WASM/JIT/GPU）
```

**章構成案**:
1. Introduction: バックエンド二重保守の問題
2. MIR14設計: 14命令の選定理由
3. 統一アーキテクチャ: VM/LLVM分離
4. 実証: LoopForm実装での効果（7-8時間完全勝利）
5. 他言語との比較
   - LLVM IR: 数百命令（複雑）
   - JVM bytecode: 200+ opcodes
   - MIR14: たった14命令（シンプル）
6. 拡張性: 新バックエンド追加の容易さ

**学術的貢献**:
- 世界最小クラスのIR（14命令）
- VM/最適化コンパイラの統一IR実証
- Box理論との一貫性
- 段階的実行戦略（開発=VM、本番=LLVM）

---

#### 3. 箱理論: Everything is Boxの数学的基礎

**アイデア**: Box統一の形式化

**キーアイデア**:
- データ・演算・制御の統一代数
- SSA/PHI構築の簡略化（650行→100行）
- 実装駆動型形式化（Implementation-Driven Formalization）

**章構成案**:
1. Box代数の定義
2. 演算子Boxの数学的性質
3. LoopFormの正規化理論
4. SSA構築の簡略化証明
5. 実装との対応

---

### 🥉 Priority 3: 面白いけど後回し

#### 4. AI却下からの雪辱 - LoopForm完全勝利の物語

**場所**: `docs/private/research/docs/private/research/papers-archive/paper-14-ai-collaborative-abstraction/chatgpt-rejection-and-redemption.md` (3990行 - 既存)

**内容**:
- LoopSignal IR究極却下
- LoopForm部分却下
- Pin方式12時間苦闘
- 「えーんえーん　蹴られ続けてきました」
- 完全説得成功
- 7-8時間で完全勝利
- 実体験駆動開発（EDD）
- 段階的問題解決モデル（LPSM）

**状態**: 既に3990行完成！抽出・編集のみ

---

#### 5. 環境変数地獄 - AI完璧主義の皮肉

**アイデア発生**: 2025-09-27

**キーアイデア**:
```yaml
ChatGPT意図:
  「環境変数で安全に制御」
  「デフォルトOFFで既存動作壊さない」
  → 完璧な設計！

実際の結果:
  「何も動かない」
  「10回挑戦してようやく動く」
  「LLVMは実装されているのに『実装なし』表示」
  → 最悪のUX！

皮肉: 安全すぎて使えない
```

**章構成案**:
1. 環境変数50個の迷宮
2. 10回挑戦の記録（ユーザー実体験）
3. LLVMモック表示事件
4. 依存関係図（誰も理解してない）
5. ChatGPT vs 人間の認識ギャップ
6. 「完璧な設計 ≠ 使える設計」

**面白ポイント**:
- 作者のChatGPTですら使い方分からない可能性
- ドキュメント不在
- 完璧主義の限界

---

#### 6. AI×AI×AI協働開発 - 人間は統括者へ

**アイデア発生**: 2025-09-27

**キーアイデア**:
```yaml
役割分担:
  ChatGPT: 実装担当
  Claude: レビュー担当
  ChatGPT 5 Pro最強モード: アーキテクト
  人間: 統括・決断・哲学守護

手法:
  コピペ駆動開発（3者の意見すり合わせ）

結果:
  「コード書かないけどへとへと」
  でも最高品質！
```

**章構成案**:
1. 3者協働の役割分担
2. コピペ駆動開発手法
3. ChatGPT 5 Pro最強モードのレビュー品質
4. 4つのGo判断分析（stringify/probe/rewrite/birth）
5. 人間の役割再定義（統括・決断・哲学守護）
6. 効率分析（時間20倍、品質3倍、疲労1.5倍）
7. 「へとへと」の本質

**実証データ**:
- LoopForm 7-8時間完全勝利
- json_query_min_vm 根治戦略
- Everything is Box哲学との整合性

---

## 📊 統計

```yaml
執筆完了: 1本（進行中）
執筆待ち: 5本

増殖ペース:
  2日前: 3本
  昨日: 4本
  今日: 6本

懸念: 指数関数的増殖の可能性
```

---

## 🎯 戦略: 80/20ルール

**優先**:
1. Priority 1完成 → HN投稿
2. 反響見てから Priority 2-3判断

**抑制**:
- 新アイデアは**タイトルだけ**追記
- 詳細は書かない（増殖防止）
- 完璧より公開優先

**保存場所**:
- このファイル: アイデアリスト
- 詳細展開は必要になってから

---

## 🔮 将来の可能性（タイトルだけ）

- [ ] 「using system: SSOT駆動名前空間解決」
- [ ] 「Property System: Python @property越え」
- [ ] 「birth統一: ライフサイクル革命」
- [ ] 「try文撤廃: postfix catch/cleanupの威力」
- [ ] 「プラグインシステム: Everything is Boxの実装」
- [ ] 「Observe→Adopt: 段階的導入フレームワーク」
- [ ] 「box_trace: 構造化可観測性」
- [ ] 「56日間で言語を作る: AI協働開発の記録」

（必要になったら詳細化）

---

**注意**: このファイルは**アイデア収集**のみ。詳細展開は慎重に！