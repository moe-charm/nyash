# LoopSignal IR: Box×Loopによる制御統一と最小MIR設計（短いRFC）

目的: LoopSignal IR（Everything is Box × Everything is Loop の融合）を短いRFCとして1本にまとめてください。構文は従来のまま、内部MIRで LoopBox+Signal に正規化する設計です。日本語でお願いします。

読んでほしいファイル（リポ相対パス）
- docs/private/papers/paper-e-loop-signal-ir/main-paper-jp.md
- docs/private/papers/paper-b-nyash-execution-model/main-paper-jp.md
- src/mir/

出力フォーマット（Markdown）
- タイトル: “LoopSignal IR: Box×Loopによる制御統一と最小MIR設計”
- 1. 背景と問題
- 2. 目標（ユースケース: scope/if/while/for/function/return/generator/async）
- 3. 設計
  - 3.1 型: LoopSignal<T>（Next/Break/Yield[/Return]）とLLVM表現（tag+iN）
  - 3.2 MIR命令: loop.begin/iter/branch/end の仕様（前提/事後条件/未定義）
  - 3.3 Box=Loop1（init/step/fini）とRAIIの対応
  - 3.4 Lowering規則（各構文の擬似MIR; for-in/return/yieldを含む）
- 4. 例
  - while(true){break} の最小例（Before/After）
  - for-inのLoopBox化（ディスパッチ図）
  - scope=Loop1の畳み込み例
- 5. 図（Mermaid）
  - 合流点を1箇所に集約した dispatch CFG
  - Signalタグに基づくswitch分岐の構造
- 6. 最適化と安全性
  - Loop1インライン化、状態省略、DCE/LICM/Inline適用条件、DWARF対策
- 7. 段階導入計画（P1/P2/P3）とフォールバック（旧MIRへの逆Lowering）
- 8. 関連研究と差分（CPS/代数的効果/ジェネレータ/SSA系IRとの比較）
- 9. 成果物とKPI（メトリクス・テスト観点）
- 付録: 用語対比（Box=空間/Loop=時間）、命令一覧、タグ割当、疑似コード集

記法/制約
- コード断片は擬似MIR/擬似LLVMでOK（実コード改変はしない）。
- 図はMermaidで書いてください（```mermaid ...```）。
- 2〜4ページ程度の密度で、可搬な単一Markdownとして出力してください。
