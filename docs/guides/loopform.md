# LoopForm ガイド — ループ正規化（ユーザーマクロ）

目的
- while/for/foreach を Nyash のユーザーマクロで“キャリア（タプル）”に正規化し、MIR/LLVMにとって最適化しやすい形へ落とし込む。

使い方（予定）
- マクロ登録（例）:
```
export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS=apps/macros/examples/loop_normalize_macro.nyash
```
- 自己ホスト前展開（auto）を利用して、parse直後にLoopForm展開を有効化（PyVM環境）。

正規化の考え方
- ループで更新される変数群をタプルに束ね、ヘッダに“1個のφ”を置く。
- break/continue は“次キャリア”または“現キャリア”で遷移し、一貫した合流点を保つ。

対応状況（MVP→順次拡張）
- Week1: while（break/continue無し）
- Week2: break/continue/ネスト最小対応、キャリア自動抽出
- Week3: for/foreach（限定）

制約（MVP）
- try/finally/throwとの相互作用は未対応。例外は後続フェーズで設計を明記。

検証
- macro‑golden（展開後ASTのゴールデン）
- LLVM PHI健全性スモーク（空PHI無し、先頭グループ化）

参考
- docs/development/roadmap/phases/phase-17-loopform-selfhost/

