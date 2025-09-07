Loaded cached credentials.
承知いたしました。LoopSignal IRの技術レビューと実装計画について、ご指定のファイルを基に分析し、提案をまとめます。

---

### LoopSignal IR: 技術レビューと実装計画 v1

#### 1. 仕様の確定案

##### 1.1. `LoopSignal` 型の正格仕様

`LoopSignal` は、ループ内の制御フローを統一的に表現するための型であり、タグとオプションのペイロードで構成される。

*   **型定義 (Rust風):**
    ```rust
    enum LoopSignal<T> {
        Continue(T),      // ループ継続 (ループ変数の更新値を運ぶ)
        Break(T),         // ループ脱出 (ループの戻り値を運ぶ)
        Return(T),        // 関数からのリターン (関数の戻り値を運ぶ)
        Yield(T),         // ジェネレータのyield (yieldする値を運ぶ)
        // Await(Future<T>), // 将来的な拡張: 非同期処理の中断
    }
    ```

*   **タグ値:**
    *   `0`: Continue
    *   `1`: Break
    *   `2`: Return
    *   `3`: Yield

*   **ABI/LLVM表現例:**
    ゼロコスト抽象を目指し、`Continue` が最も頻繁に発生することを前提に最適化する。LLVM IRでは、戻り値を `struct` で表現するのが一般的。

    ```llvm
    ; LoopSignal<Value> の表現例
    ; %Value はペイロードの型 (e.g., i64, %MyObject*)
    %LoopSignal = type { i8, %Value } ; { tag, payload }

    ; ゼロコスト表現の検討
    ; 関数の戻り値として LoopSignal を直接返すのではなく、
    ; ループ本体の関数はペイロードの値だけを返し、
    ; シグナル自体は別の方法（例：ステータス引数への書き込み、複数のリターンブロック）で伝える。
    ; しかし、LLVMの最適化能力を信じ、まずは上記のシンプルなstruct表現で実装し、
    ; パフォーマンスが問題になる箇所で特殊化するのが現実的。
    ; 多くの `Continue` はループ内で処理され、関数境界を越えないため、
    ; レジスタ割り当てによりオーバーヘッドはほぼゼロになることが期待される。
    ```
    **ゼロコスト表現の可否:** **可能**である。ループが関数境界をまたがない限り、`LoopSignal` は物理的なメモリ確保を伴わず、仮想レジスタと条件分岐にコンパイルされる。`loop.branch` が `switch` 命令に変換され、`Continue` のケースが他のケースより優先的に配置（`default` 分岐など）されることで、最も高速なパスとなる。

##### 1.2. MIR命令の厳密な意味論

*   `loop.begin <label>`:
    *   新しいループスコープを開始する。`<label>` はこのループを一意に識別する。
    *   SSAのPHIノードの配置ポイントとなる。ループに複数回突入する全ての値（ループ変数など）は、この地点でPHIノードによってマージされる。
    *   **未定義動作:** `loop.end` と対応が取れない場合。

*   `loop.iter <label>, (%vars...) -> (%next_vars...)`:
    *   ループの1イテレーションの開始を示す。ループ条件の評価やループ変数の更新を行うコードがここに配置される。
    *   `%vars` を受け取り、次の状態である `%next_vars` を生成する。ジェネレータの再開時には、外部から渡された値が `%vars` の一部となる。
    *   **意味論:** この命令自体は副作用を持たないが、後続のコードがループ継続/脱出の判断を行う。

*   `loop.branch <label>, %signal`:
    *   `%signal` (`LoopSignal` 型) の値に基づき、制御フローを分岐させる。
    *   `Continue(val)`: `<label>` に対応する `loop.iter` のバックエッジにジャンプする。`val` は次のイテレーションのPHIノードへの入力となる。
    *   `Break(val)` / `Return(val)` / `Yield(val)`: `<label>` に対応する `loop.end` にジャンプする。`val` は `loop.end` の結果となる。
    *   **未定義動作:** `%signal` が初期化されていない場合。対応する `<label>` がスコープ内にない場合。

*   `loop.end <label> -> (%signal, %value)`:
    *   ループスコープを終了する。`loop.branch` からの `Break`, `Return`, `Yield` シグナルを受け取る。
    *   この命令の結果として、ループを終了させたシグナルの種類と、そのペイロード値が出力される。
    *   親ループが存在する場合、この結果が親ループの `LoopSignal` となり、`loop.branch` に渡される。
    *   **例外扱い:** ループ内で発生した例外は、`LoopSignal` とは別の経路（例: `invoke` と `landingpad`）で処理されるべき。`loop.branch` は例外を伝播させない。

##### 1.3. SSA/PHI配置の規則

*   **合流点の標準形:**
    *   `loop.begin`: ループの入り口（初回実行とバックエッジからの合流）。ループ変数はここでPHIノードを持つ。
    *   `loop.end` の後: ループが正常に終了した（`Break`した）後の実行パス。ループの戻り値はここでPHIノードを持つ（ループに入らなかったケースとの合流）。
*   **インバリアント:**
    *   全てのループ変数は `loop.begin` でPHIノードによって定義される。
    *   `loop.begin` のPHIノードへの入力は、ループへの初回突入時の値と、各 `loop.branch` の `Continue` ペイロードから供給される。
    *   `loop.end` は複数の `loop.branch` からの `Break` 値を受け取るため、PHIノードと同様の機能を持つ。

#### 2. Lowering規則（構文→MIR）

##### 2.1. 各構文から擬似MIRへの写像

*   **`if cond { true_branch } else { false_branch }`**
    ```mir
    %cond_val = ...
    br.cond %cond_val, then: <bb_true>, else: <bb_false>

    <bb_true>:
      ... // true_branch のコード
      %signal_true = ...
      br <bb_merge>

    <bb_false>:
      ... // false_branch のコード
      %signal_false = ...
      br <bb_merge>

    <bb_merge>:
      %signal = phi [%signal_true, <bb_true>], [%signal_false, <bb_false>]
      // この %signal が後続の loop.branch に渡される
    ```

*   **`while cond { body }`**
    ```mir
    loop.begin <L0>
    br <L0_iter>

    <L0_iter>:
      %loop_var_next = phi [%loop_var_init, <entry>], [%loop_var_updated, <L0_body>]
      %cond_val = ... // cond の評価
      br.cond %cond_val, then: <L0_body>, else: <L0_break>

    <L0_body>:
      ... // body のコード
      %loop_var_updated = ...
      %signal = (Signal::Continue, %loop_var_updated)
      loop.branch <L0>, %signal // Continue

    <L0_break>:
      %break_val = ... // ループの戻り値 (e.g., unit)
      %signal = (Signal::Break, %break_val)
      loop.branch <L0>, %signal // Break

    (%final_signal, %final_value) = loop.end <L0>
    // final_signal に基づいて後続の処理
    ```

*   **`function` と `return`**
    ```mir
    fn my_func(%arg1, ...) -> %ret_val {
      loop.begin <F_main> // 関数全体を暗黙のループと見なす

      ... // 関数の本体
      // return expr; は以下に変換
      %return_val = ... // expr の評価
      %signal = (Signal::Return, %return_val)
      loop.branch <F_main>, %signal

      ...

      // 関数の終端 (暗黙の return)
      %implicit_ret_val = ...
      %signal_implicit = (Signal::Return, %implicit_ret_val)
      loop.branch <F_main>, %signal_implicit

      (%final_signal, %final_value) = loop.end <F_main>
      // final_signal は必ず Return のはず
      ret %final_value
    }
    ```

*   **`generator` と `yield`**
    `yield` は `Return` と同様に `loop.branch` に変換されるが、`Signal::Yield` を使用する。関数の再開は、`loop.iter` に値（`resume_arg`）を渡すことで実現される。

##### 2.2. `Return` を `Signal` に含めるか分離するかの設計比較

*   **統一モデル (ReturnをSignalに含める)**
    *   **長所:**
        *   **IRの直交性:** 制御フローの変更はすべて `LoopSignal` と `loop.branch` に統一され、IRがシンプルで美しくなる。
        *   **最適化の容易さ:** `return`, `break`, `yield` を同じ枠組みで扱えるため、最適化パス（特にインライン化）の設計が単純になる。`if cond { return } else { break }` のような複雑な制御フローも自然に表現できる。
        *   **リファクタリング耐性:** ループ内のコードを別関数に抽出したり、その逆を行ったりするリファクタリングが容易になる。
    *   **短所:**
        *   **僅かな冗長性:** 関数の末尾にある単純な `return` も `Signal` 生成と `loop.branch` を経由するため、見た目上は冗長に感じる可能性がある。

*   **分離モデル (Returnを別の命令 `mir.ret` にする)**
    *   **長所:**
        *   **従来との親和性:** 既存のCFGベースの考え方に近く、`ret` 命令は関数の終端として直感的。
    *   **短所:**
        *   **IRの複雑化:** 制御フローを終端させる命令が `loop.branch` と `ret` の2種類になり、解析や変換が複雑になる。
        *   **最適化の阻害:** `if cond { return } else { break }` の合流点での処理が困難になる。インライン化の際に、呼び出し先の `ret` を呼び出し元の `break` や `continue` に変換する必要があり、アドホックな処理が必要になる。

*   **採用推奨:** **統一モデルを強く推奨する。** 長期的な保守性、拡張性、最適化のポテンシャルを考慮すると、IRのシンプルさと直交性がもたらすメリットは、僅かな冗長性を補って余りある。

#### 3. 最適化パス

*   **Loop1完全インライン化:**
    *   ループ本体に `loop.branch` が1つしかなく、そのシグナルが静的に `Continue` である場合、ループ構造 (`loop.begin/iter/branch/end`) を完全に除去し、ループ本体のコードを親ブロックに展開する。これは単純な `scope { ... }` のゼロコスト抽象化に繋がる。
*   **Yieldなし状態省略:**
    *   ジェネレータ関数のMIRを解析し、`Signal::Yield` を生成する `loop.branch` が存在しない場合、その関数をジェネレータではなく通常の関数として扱う。これにより、ステートマシンの生成コストを回避できる。
*   **分岐合流点正規化:**
    *   ネストした `if` 文などを解析し、複数の `loop.branch` を持つフラットな構造に変換する。これにより、後続のパスが扱いやすい標準形にできる。
*   **既存パスへの影響:**
    *   **DCE (Dead Code Elimination):** `loop.branch` の静的に到達不能な分岐（例: `if false { return }`）を検出し、関連コードを削除できる。
    *   **LICM (Loop Invariant Code Motion):** `loop.begin` から `loop.end` の範囲が明確になるため、ループ不変条件の検出がより正確かつ容易になる。
    *   **Inlining:** 関数インライン化が非常に強力になる。呼び出し先の `loop.end` が返す `Signal` を、呼び出し元の `loop.branch` に直接接続できる。例えば、`return` は呼び出し元での値の代入に、`break` は呼び出し元のループの `break` に変換される。

#### 4. フォールバック/互換性

*   **LoopSignal IR → 従来MIRへの逆Loweringパス:**
    *   新IRの健全性を検証し、段階的に導入するために、コンパイラフラグ (`--no-loop-signal-ir`) でON/OFF可能な逆変換パスを設計する。
    *   **変換ロジック:**
        1.  `loop.begin <L>`: 新しい基本ブロック `<L_header>` を作成。
        2.  `loop.iter`: ヘッダブロック内にコードを配置。
        3.  `loop.branch <L>, %signal`: `%signal` のタグで `switch` 命令を生成する。
            *   `Continue`: `<L_header>` へのバックエッジを作成。
            *   `Break`: ループ外の `<L_exit>` ブロックへジャンプ。
            *   `Return`: 関数のグローバルなリターンブロックへジャンプ。
        4.  `loop.end <L>`: `<L_exit>` ブロックに対応。複数の `Break` からの値はPHIノードでマージする。
    *   このパスにより、LoopSignal IRをサポートしないバックエンド（旧VMやデバッガなど）でも動作を継続できる。

#### 5. リスクと回避策

*   **デバッグ情報 (DWARF/位置情報):**
    *   **リスク:** `Signal` による非線形な制御フローで、ステップ実行がソースコードの見た目と乖離する可能性がある。
    *   **回避策:** `loop.begin` から `loop.end` までをソースコード上のループ構文（`while`, `for`など）のスコープとして正確に対応付けるデバッグ情報を生成する。`loop.branch` 命令には、ソースコード上の `break`, `return` 文の位置情報を付与する。
*   **小反復ループの関数境界コスト:**
    *   **リスク:** ループ本体が小さく、頻繁に呼び出される関数内にある場合、`LoopSignal` 構造体の生成・返却コストが無視できなくなる可能性がある。
    *   **回避策:**
        1.  強力なインライナを実装し、関数境界を越える `LoopSignal` の受け渡しを極力なくす。
        2.  LLVMバックエンドで、戻り値の `struct` がレジスタ経由で渡されるよう最適化されていることを確認する（多くのABIではそうなっている）。
*   **例外/効果の扱い:**
    *   **リスク:** `panic!` や FFI 呼び出しなどの副作用が `LoopSignal` のセマンティクスを破壊する可能性がある。
    *   **回避策:** 仕様として、`LoopSignal` は純粋な制御フローのみを扱い、例外やパニックは別の機構（LLVMの `invoke`/`landingpad` や、Rustの `catch_unwind` に相当する機構）で処理することを明確にする。`loop.branch` を含む可能性のある処理は `invoke` で呼び出す必要がある。

#### 6. 段階導入ロードマップ

*   **P1: `while(true)` と `break` のLoop1化 (最小ループ)**
    *   **目標:** 最も単純な無限ループと `break` を `loop.begin/branch/end` で表現する。
    *   **実装:** `while` 文のLoweringを修正。`break` を `Signal::Break` に変換。
    *   **検証:** 生成されるMIRが正しいか、性能計測を行い、従来のCFGベースの実装と比較してオーバーヘッドがないことを確認する。
*   **P2: `for-in` ステートマシンの実装**
    *   **目標:** イテレータを使った `for` ループを `LoopSignal` で表現する。
    *   **実装:** `for` 文のLoweringで、`iterator.next()` の呼び出しと `Some/None` のマッチを `loop.iter` と `loop.branch` で構成する。
    *   **検証:** VM（インタプリタ）とAOT（LLVM）で、ループの挙動（特にループ変数の値）が完全に一致することをアサーションテストで検証する。
*   **P3: 最小ジェネレータ (`yield`) の実装**
    *   **目標:** `yield` を持つジェネレータ関数をサポートする。
    *   **実装:** `yield` を `Signal::Yield` に変換。ジェネレータの再開ロジック（ステートマシンの復元と `loop.iter` への値渡し）を実装する。
    *   **検証:** ジェネレータを複数回再開させ、状態が正しく保存・復元されるかを確認するテストケースを多数作成する。

#### 7. テスト計画とメトリクス

*   **収集するメトリクス:**
    *   **MIRレベル:** `MirOp` の総数、`loop.*` 命令の数、PHIノードの数。
    *   **コンパイル時:** フロントエンド処理時間、最適化パス処理時間、バックエンド（LLVM）処理時間。
    *   **実行時:** 主要なベンチマークスイートにおける実行時間、メモリ使用量。
*   **収集方法:**
    *   コンパイラに `--emit-metrics=json` フラグを追加し、ビルドごとに上記のメトリクスをJSONファイルに出力させる。
    *   CI上でPRごとにメトリクスを計測・比較し、閾値（例: 実行時間 5%以上の悪化）を超えた場合に警告を出す。
*   **テストケース:**
    *   ネストしたループ、複雑な `if/else` を含むループ、`return`/`break`/`continue` が混在するループなど、エッジケースを網羅した単体テストを追加する。
    *   既存のE2Eテスト (`apps/tests/`) がすべてパスすることを確認する。

#### 8. 小タスクのTODO一覧

1.  **[型定義] `LoopSignal` enum と `loop.*` 命令の追加**
    *   **ファイル:** `src/mir/ops.rs`, `src/mir/nodes.rs`
    *   **内容:** `LoopSignal` enumを定義。`MirOp` に `LoopBegin`, `LoopIter`, `LoopBranch`, `LoopEnd` を追加。`Terminator` (または `ControlFlow`) から `Br`, `CondBr` などを非推奨化または削除していく。

2.  **[Lowering] `while` 文のLoweringを `LoopSignal` ベースに移行 (P1)**
    *   **ファイル:** `src/hir/lowering.rs` (または相当するファイル)
    *   **内容:** `while` 文のASTノードから `loop.begin/iter/branch/end` を生成するロジックを実装する。

3.  **[SSA] `loop.begin` でのPHIノード生成ロジックの実装**
    *   **ファイル:** `src/mir/builder_modularized/`
    *   **内容:** ループのバックエッジを検出し、`loop.begin` の位置に正しくPHIノードを挿入するよう、SSA構築アルゴリズムを修正する。

4.  **[バックエンド] LLVMバックエンドでの `loop.*` 命令の処理 (P1)**
    *   **ファイル:** `src/runner/modes/llvm.rs`
    *   **内容:** `LoopSignal` のLLVM `Type` を定義。`loop.begin` でLLVMのループヘッダブロックを、`loop.branch` で `switch` 命令を、`loop.end` でループ出口ブロックを生成するコードを追加する。

5.  **[最適化] Loop1完全インライン化パスの実装**
    *   **ファイル:** `src/mir/optimization/simplify.rs` (または新規ファイル)
    *   **内容:** 条件を満たす `loop.*` 命令シーケンスを検出し、除去する最適化パスを実装する。

6.  **[互換性] 逆Loweringパスの実装**
    *   **ファイル:** `src/mir/passes/compat_lower_loop_signal.rs` (新規ファイル)
    *   **内容:** LoopSignal IRを従来のCFGベースMIRに変換するパスを実装し、コンパイラフラグで制御できるようにする。

7.  **[テスト] メトリクス収集機能の追加**
    *   **ファイル:** `src/main.rs`, `src/driver.rs`
    *   **内容:** コンパイルの各ステージでメトリクスを収集し、指定されたフォーマットで出力する機能を追加する。
