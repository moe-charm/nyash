# Nyash: Core Minimal + Strong Sugar

> 最小のコア言語に、強力な糖衣構文を重ねて「書きやすさ」と「0コスト正規化」を両立する方針です。

## Core（最小）
- 制御: `if`, `loop(condition) { … }`, `break`, `continue`（単一入口・先頭条件）
- 式: `const/binop/compare/branch/jump/ret/phi`、`call/boxcall`
- 単項: `-x`, `!x` / `not x`（真偽は i64 0/1 へ正規化）
- 例外: `try/catch/cleanup`（postfix 版は正規化で TryCatch に降下）

設計上の非採用
- do‑while: 不採用（先頭条件原則）。代替は糖衣で表現→先頭条件へ正規化。

### 演算子とループの方針（要約）
- 単項 not（`!`）は採用（既存の `not` と同義）。
- do‑while は非採用（明確性と正規化単純性を優先）。
- ループは LoopForm 正規化に合わせて糖衣→正規形に落とす（break/continue を含む）。

## Sugar（強く・美しく・0コスト）
- repeat N { … }
  - 正規化: `i=0; while(i<N){ …; i=i+1 }`（`loop`に降下）
- until cond { … }
  - 正規化: `while(!cond){ … }`（`!` は Compare(Eq,0) へ）
- for i in A..B { … }
  - 正規化: 範囲生成なしでカウンタ while へ
- foreach x in arr { … }
  - 正規化: `i=0; while(i < arr.size()){ x=arr.get(i); …; i=i+1 }`
- 文字列補間: `"hello ${name}"` → `"hello " + name`
- 三項: `cond ? a : b` → `if/else + PHI`
- 論理代入: `a ||= b` / `a &&= b` → `if(!a) a=b` / `if(a) a=b`

いずれも「意味論を変えずに」`loop/if` へ降下します。MIR/LLVM/Cranelift の下層は常にコア形にのみ対応すればよく、認知負荷を小さく保てます。

## 実装ガイド（Rust/PyVM共通）
- Parser は糖衣の表層を受理し、Normalize（前段）で 0コストに正規化→ MIR 降下。
- PyVM は正規化後の MIR を実行（P0 機能のみ実装）。
- LLVM は PHI/SSA 衛生を守る。空PHIは不可、PHIはブロック先頭。

## Profiles（実行プロファイル・方針）
- dev: 糖衣ON/デバッグON（作業向け）
- lite: 糖衣OFF/静音（軽量）
- ci: 糖衣ON/strict/静音（最小スモークのみ）

将来の拡張
- 文字列補間/複数行/安全アクセスなどの糖衣は、常に正規化→コア形（if/loop）へ降下していきます。
- 例外の後処理は `cleanup` に統一し、`defer` 的表現は糖衣→`cleanup` へ。
