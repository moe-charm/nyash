# EntryResolveBox — エントリ解決の一元化（設計）

目的
- 既定を Strict（`Main.main` のみ）に保ちつつ、明示指定（`--entry <dotted>`）や候補列挙を統一的に扱う小さな“箱”（薄い境界）を用意する。
- VM/LLVM/PyVM/子プロセス（ハーネス）すべてで同じ規則・同じエラーメッセージにする。

方針（環境変数は増やさない）
- 既定は Strict: 自動採用は `Main.main` だけ。
- 例外は CLI `--entry <dotted>` のみ（明示）。
- 自動推測（唯一の `<Box>.main` や top-level `main`）は採用しない。候補として列挙はするが、実行はしない。

インターフェース（Rust 参考案）
```rust
// 入力: MIR モジュール + CLI 指定（任意）
pub struct EntryResolveInput<'m> {
    pub module: &'m MirModule,
    pub cli_entry: Option<String>, // e.g., "Fibonacci.main" or "Main.main"
}

// 出力: 解決結果のメタ + 実エントリ名
#[derive(Debug, Clone)]
pub struct ResolvedEntry {
    pub name: String,          // 実行する関数キー（"Main.main" など）
    pub kind: EntryKind,       // 解決の理由
    pub candidates: Vec<String>, // 観測された候補（診断用）
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    MainMain,     // 既定: Main.main
    Specified,    // CLI --entry 指定で解決
}

pub enum EntryError {
    MissingMainStrict { candidates: Vec<String> },
    CliSpecifiedNotFound { name: String, candidates: Vec<String> },
}

pub fn resolve_entry(input: EntryResolveInput) -> Result<ResolvedEntry, EntryError>;
```

解決アルゴリズム
1) CLI が `--entry <dotted>` を指定した場合
   - その名前（`name` or `name/arity` を許容）を正規化して一致探索
   - 見つからなければ `CliSpecifiedNotFound`（候補列挙）
2) 未指定（Strict）
   - `Main.main`（`/0` を含む派生も受理）を探索し、合致すれば採用
   - それ以外は `MissingMainStrict`（候補列挙 + ガイダンス）

候補列挙の規則
- `*.main` / `*.main/0` を候補として収集（辞書順）
- トップレベル `main` が存在する場合も候補として表示（Strict のため採用しない旨を明記）

エラーメッセージ（例）
- Strict で見つからない場合:
  - "entry not found (Strict). Expected 'Main.main'. Candidates: A.main, B.main/0. Use 'flow Main' or pass '--entry A.main'"
- CLI 指定が不一致の場合:
  - "entry '--entry Foo.main' not found. Candidates: Main.main, Foo.main/0"

優先順位表（簡易）
- CLI `--entry` > Strict `Main.main` > （採用しない: UniqueStatic / TopLevel）

実装分割（箱の責務）
- EntryResolveBox（runner 層）
  - 責務: 解析・列挙・解決・メッセージ整形
  - 非責務: 実行（VM/LLVM/PyVM）は呼び出し側の責務
- 利用箇所
  - MIR Interpreter（VM）
  - JSON→PyVM ブリッジ
  - LLVM ハーネス/モック実行
  - 子プロセス（selfhost）起動前のエントリ選別

スモーク（設計レベルの仕様）
- strict_missing_main.sh
  - 入力: `flow App { main() {} }`（Main ではない）
  - 期待: 非0終了 + エラーメッセージに `Candidates:` と `Use 'flow Main' or '--entry'` が含まれる
- strict_flow_main_ok.sh
  - 入力: `flow Main { main() { return 0 } }`
  - 期待: 0 終了
- cli_entry_ok.sh
  - 入力: `flow App { main() { return 0 } }` + `--entry App.main`
  - 期待: 0 終了
- cli_entry_not_found.sh
  - 入力: `flow App { main() { return 0 } }` + `--entry Foo.main`
  - 期待: 非0終了 + 候補列挙

段階導入（変更リスクの抑制）
- Phase A（今）: ドキュメント合意（本ファイル）、既存の緩和フラグはドキュメント上“非推奨”にする
- Phase B: CLI `--entry` を追加（挙動は Strict 既定、実装は小粒）
- Phase C: ランナー各所のエントリ選択を EntryResolveBox に置換（振る舞い同一）
- Phase D: 便宜的な自動推測・環境変数の撤廃（または既定OFF+削除予定アノテーション）

備考
- Flow Main を第一推奨。静的状態が必要な場合のみ static Main。
- 仕様の単純さ（Strict）を保つことで、入門体験とバグ原因の切り分けが大幅に改善される。

