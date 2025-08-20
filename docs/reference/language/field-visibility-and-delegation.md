# フィールド可視性とデリゲーション設計（提案/仕様草案）

本書は Nyash 言語の「フィールド可視性」と「デリゲーション（from/override）」の設計をまとめた仕様草案です。実装は段階的に進めます。

## 1. フィールド可視性（Blocks）
- 構文
  ```nyash
  box User {
      private { age, passwordHash, internalCache }
      public  { name, email, displayName }

      init { name, email, displayName, age }
      // ... methods ...
  }
  ```
- ルール
  - `private`: box 内のメソッドからのみアクセス可（外部アクセス不可）
  - `public`: 外部から `obj.field` で参照可
  - いずれにも属さないフィールドはエラー（明示主義）
  - `me.field` は可視性に関わらず常に許可（自身の内部）
  - init は現状どおり public（将来 `public/private init` の導入は別途検討）
- エラー例
  - 外部から `user.age` → Compile/Interpret Error: private field access
  - 親の private フィールド参照 → Error: parent private field not accessible

実装フェーズ（予定）
1. パーサ: `private { ... }` / `public { ... }` の受理、AST/宣言モデルに可視性付与
2. 解決/実行: 外部アクセス時に public のみ許可、`me.field` は常にOK
3. MIR/VM: 既存の `RefGet/RefSet` に可視性チェックフックを追加

## 2. デリゲーション（from/override）
目標: ビルトイン/プラグイン/ユーザー定義の全Boxで、同一の書き味（from/override/super呼び出し）を提供しつつ、安全性を担保する。

- 基本方針
  - 継承モデルは維持（「委譲の糖衣」ではない）
  - 親の内部フィールドは不可視（private は子からも見えない）
  - メソッド解決規則は統一：子の override → なければ親へ解決
  - 親がビルトイン/プラグインの場合、親メソッド呼び出しは `BoxCall` に lower（ローダ経由）

- 書式例
  ```nyash
  box MyFile from FileBox {
      override write(x) {
          // 前処理
          from FileBox.write(x)
          // 後処理
      }
  }
  ```

- 安全性
  - 親の状態は親の実体としてのみ存在（フィールド継承しない）
  - `from Parent.method()` は親タイプ名での明示ディスパッチ（暗黙の内包はしない）
  - MIR では `BoxCall` を生成し、VM/Interpreter はローダへ委譲

実装フェーズ（予定）
1. MIR Lowering: 親がビルトイン/プラグインの `from` を常に `BoxCall` へ（現状踏襲）
2. VM/Interpreter: 既存どおりローダ委譲（Handle/BoxRef を統一処理）

## 3. 参考
- BoxRef/Handle 仕様: `docs/reference/plugin-system/boxref-behavior.md`
- nyash.toml v2.1–v2.2: `docs/reference/plugin-system/nyash-toml-v2_1-spec.md`
- 実装箇所（予定）: `src/parser/declarations/box_definition.rs`, `src/core/model.rs`, `src/interpreter/expressions/access.rs`, `src/mir/*`, `src/backend/vm.rs`

