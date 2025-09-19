# ScopeBox（コンパイル時メタ）設計ガイド

目的
- スコープ境界・defer・capabilities を“箱（Box）”のメタとして表現しつつ、最終的な実行物ではゼロコストにする。
- LoopForm（ループのみキャリア整形）と責務分離し、If/Match は合流点正規化（join変数＋単一PHI群）に限定する。

基本方針
- ScopeBox は“消える箱”。AST/マクロ段階で Block に属性を付与し、MIR ではヒントとして解釈、IR では完全に剥がす。
- ループを伴わないスコープを LoopForm に持ち込まない（0回ループ化はしない）。

属性とヒント（MVP）
- AST（マクロ内部表現）
  - Block.attrs.scope: { id, name, caps?: {io,net,env}, defer?: [Call…], diag?: {...} }
  - 備考: 現段階では属性はマクロ内で保持するだけ。MIR 降下時にヒントへ写すのが目標。
- MIR（ヒントのみ／構造不変）
  - hint.scope_enter(id) / hint.scope_leave(id)
  - hint.defer(list) ・・・ 静的展開（cleanup合流）に用いる
  - hint.join_result(var) ・・・ If/Match の合流結果を明示
  - hint.loop_carrier(vars...) ・・・ ループヘッダで同一PHI群に導く
  - hint.no_empty_phi（検証）
- IR（Release）
  - すべての hint は生成前に剥離。追加命令は一切残らない。

パイプライン
1) Parse → Macro（If/Match 正規化、Scope 属性付与）
2) LoopForm（while/for/foreach のみ、キャリア整形）
3) Resolve → MIR Lower（ヒント埋め、defer静的展開）
4) 検証（PHI先頭／空PHI無し）→ ヒント剥離 → Backend

受け入れ基準（Release）
- IR に scope/hint 名が一切出現しない。
- LoopForm ケースはヘッダ先頭に PHI がまとまり、空PHI無し。
- If/Match 式は join 変数 1 個で収束（空PHI無し）。

今後の予定（短期）
- マクロ: @scope/@defer 記法のスキャフォールド → AST 属性付与（挙動は現状どおり、構造変更無し）。
- MIR: hint.* の型と注入ポイント定義（降下部のスケルトン、既定は no-op）。
- スモーク: IR に scope/hint 名が残らないこと、PHI 健全性が保たれることを確認。

