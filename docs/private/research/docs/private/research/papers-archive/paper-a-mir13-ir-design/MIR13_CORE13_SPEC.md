# MIR14 Core‑14 Specification (Draft)

本メモは、MIR14（Core‑14）命令体系の確定仕様と、レガシー命令の廃止方針をまとめる。実装は"Core‑14既定ON・forbid‑legacy"を前提とする。

## 1. Core‑14 命令一覧（最小限＋実践的）

| 区分 | 命令 | 役割（要点） |
|------|------|---------------|
| 値   | Const      | 即値・アドレス等の定数生成（副作用なし） |
| 演算 | BinOp      | 加減乗除/ビット演算（純粋） |
| 演算 | UnaryOp    | 単項演算（否定、NOT等）【実用性から復活】 |
| 演算 | Compare    | 比較演算（純粋） |
| 制御 | Jump       | 無条件遷移（終端） |
| 制御 | Branch     | 条件分岐遷移（終端） |
| 制御 | Return     | 関数復帰（終端） |
| 形状 | Phi        | SSA合流（純粋） |
| 呼出 | Call       | 直接/間接呼出（ユーザー関数） |
| 呼出 | BoxCall    | Boxへのメッセージ呼出（配列/フィールド/メソッドの統一） |
| 呼出 | ExternCall | ランタイム/プラグインへの呼出（FFI境界） |
| 型   | TypeOp     | 型判定・型変換（型関連演算の統合） |
| 実行 | Safepoint  | 安全点（GC/割込み協調） |
| 実行 | Barrier    | 書込/読込バリア等の最小表現 |

注:
- Branch/Jump/Return は終端命令。Phi は構文木上で合流点にのみ出現。
- BoxCall は自由可変長引数（receiver＋メソッド名/操作名＋args...）を標準とし、BoxCallWithは廃止。

## 2. IR規約（Invariants）
- SSA: すべての値は一度だけ定義。Phiは支配関係に従い配置。
- 終端整合: Blockの末尾は {Return|Jump|Branch} のいずれか1つ。
- 副作用モデル:
  - 純粋: Const/BinOp/Compare/Phi
  - 効果あり: Call/BoxCall/ExternCall/Safepoint/Barrier（効果種別はEffect Maskで注釈可）
- Safepoint配置: ループヘッダ/長期待機前/FFI直後などに挿入（最小限）。
- Barrier: write/read バリアはCore‑13で抽象化し、下位で最適化。

## 3. 高位→Core‑13 への標準Lowering
- 配列/フィールド/メソッド: すべて BoxCall で統一。
  - 例) `a[i]` → `BoxCall(a, "get", i)`
  - 例) `o.name` → `BoxCall(o, "getField", "name")`
  - 例) `o.add(x)` → `BoxCall(o, "add", x)`
- ランタイム/プラグイン: `ExternCall("iface", "method", args...)` による一貫表現。
- 型操作: `TypeOp(kind, value[, type])`（型判定/変換を単一路に集約）。
- 制御構造: if/loop は Branch/Jump/Phi で表現。

### 3.1 記法方針（表記と内部の二層）
- 表記: 従来の if / while / for / return などの構文を維持（ユーザフレンドリ）。
- 内部: LoopForm IR（loop.begin/iter/branch/end）に正規化。
- 最終: LoopForm → Core‑13 へ逆Lowering（Branch/Jump/Phi/Return へ落とす）。

これにより、言語表記の自由度とIRの直交性（正規形）を両立する。

## 4. LoopForm（LoopSignal IR）との整合
- LoopForm は“中間正規形”として `loop.begin/iter/branch/end` を導入（Core‑13の上位層）。
- 逆Lowering: LoopForm → Core‑13 は以下の基本変換で常時可能：
  - `loop.begin` → ヘッダBlock生成＋Phi配置
  - `loop.iter`   → 条件/stepコードをヘッダ/ボディに分配
  - `loop.branch` → `switch/Branch` + `Jump`
  - `loop.end`    → 合流先にReturn/Jump（Signal種別に応じる）
- Safepoint/Barrier は Core‑13 層で維持。LoopFormは制御の正規化に専念。

## 5. レガシー命令の廃止マップ
- Load/Store / ArrayGet/ArraySet / RefGet/RefSet / WeakNew/WeakLoad → BoxCall（必要時Barrier/Safepoint併用）
- TypeCheck/Cast → TypeOp
- PluginInvoke → ExternCall / BoxCall（ABIに応じて）
- Nop/Copy/UnOp 等の補助命令 → 最適化/ビルダ内部に吸収（表面APIから排除）

## 6. ExternCall の階層化（境界の明示）
- iface例:
  - `env.runtime`: ランタイム内部API（checkpoint等）
  - `env.gc`: GC操作（将来）
  - `plugin.*`: プラグイン提供のFFI群
- 指針: BoxCallで表現可能な操作は BoxCall を優先（抽象度維持）。どうしてもhost境界を越える必要がある場合のみ ExternCall。

## 7. 妥当性検査（Lint/Verify）
- Phi配置の正当性（支配木チェック）
- 終端命令の整合
- EffectとSafepointの整合（長期ループでの安全点確保）
- レガシー命令検出（forbid‑legacy がONであること）

## 8. 移行計画（段階導入）
1) フラグ: Core‑13 既定ON/forbid‑legacy を実装側で保証（nyash.tomlも同値）。
2) レガシー→Core‑13 置換を段階実施（ビルダ/最適化/バックエンドを横断）
3) LoopForm（任意）を導入し、while/for/scope から正規化→逆LoweringでCore‑13へ落とす
4) 検証: 既存スイート + 再現ベンチ（AOT/VM/JIT） + Lint で差分監視

---

この文書は“仕様の真実の源泉（single source of truth）”として、Core‑13 と上位LoopFormの整合と廃止路線を明示する。実装の進捗に合わせて更新する。
