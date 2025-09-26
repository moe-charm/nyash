# MacroBox — User Extensible Macro Units

Status: Stable for user macros via Nyash runner route（self‑hosting優先）。

Goals
- Allow users to implement procedural macro-like expansions in pure Nyash (future) and Rust (prototype) with a clean, deterministic interface.
- Preserve “Box independence” (loose coupling). Operate on public interfaces; avoid relying on private internals.

Enabling
- Macro engine is ON by default. Disable with `NYASH_MACRO_DISABLE=1`.
- Register user macro files via `NYASH_MACRO_PATHS=path1,path2`.
- Runner route（Nyashでの実行）が既定。内部子ルートは非推奨（`NYASH_MACRO_BOX_CHILD_RUNNER=0` でのみ強制）。

API（Nyash, 推奨）
- Box: `static box MacroBoxSpec { static function expand(json[, ctx]) -> json }`
- Contract: AST JSON v0（docs/reference/ir/ast-json-v0.md）。ctxはJSON文字列（MVP: {caps:{io,net,env}}）。
- 実行: エンジンは登録されたファイルをNyashランナールートで呼び出し、展開JSONを受け取る。

API（Rust, 内部/開発者向け）
- Trait: `MacroBox { name() -> &'static str; expand(&ASTNode) -> ASTNode }`
- 用途: 最小のビルトイン（例: derive）やプロトタイピング。一般ユーザーはNyash版を使う。

Constraints (philosophy)
- Deterministic; side-effect free by default（io/net/envはcapabilitiesで明示）
- Respect “Box independence”: 公開インターフェースを基本に、疎結合を維持

Built-in example
- `UppercasePrintMacro`（開発者向けの内蔵例）。通常はNyashで実装されたユーザーマクロを使用。

Future plan
- nyash.toml による登録/パッケージ管理
- Attribute-style macros（@derive/@lint）の整理
