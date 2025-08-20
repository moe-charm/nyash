# Phase 9.78c: プラグインBoxのデリゲーション一体化計画（Interpreter先行）

作成日: 2025-08-21  
優先度: 高  
担当: Codex + User

## 目的（Goal）
ユーザー定義／ビルトイン／プラグインの3種のBoxを、デリゲーション（`from Parent.method`）と通常メソッド解決で「ほぼ同じ体験」に揃える。第一段としてInterpreter側での親メソッド呼び出し経路を拡張し、プラグイン親のフォールバック経路を提供する。

## 背景（Context）
- 現在：
  - ユーザー定義は素直に解決、ビルトインは`__builtin_content`経由で親メソッドを呼び出し可能。
  - プラグインは`PluginBoxV2`として生成・利用可能だが、親メソッド呼び出しのフォールバック経路が未整備。
- VM側は`BoxCall`で`PluginBoxV2`を検出し、ローダにルーティング可能（最小の引数/戻り値サポートは導入済）。
- Interpreterでも同じ体験を提供するため、親プラグインへのブリッジを追加する。

## スコープ（Scope）
- Interpreter先行の対応：
  1) `InstanceBox` に `__plugin_content`（NyashValue::Box）を導入
  2) 子Boxが `from PluginBox` を宣言している場合、子の生誕時（birth）にプラグイン親を生成して保持
  3) メソッド解決：
     - ユーザー定義メソッドで見つからない → 既存のビルトイン親チェック → プラグイン親チェック（`__plugin_content`があれば `PluginLoaderV2.invoke_instance_method`）
  4) `from Parent.method` のInterpreter側分岐：Parentがプラグインであれば、上記 `invoke_instance_method` に直接ルーティング
- VM側の“from Parent.method”は次フェーズ（9.78d以降）で整備（Builder/VMの双方に影響大のため）

## 具体タスク（Plan）
1. InstanceBox拡張（低リスク）
   - `fields_ng`に `"__plugin_content"` を保持できるように（キー名は固定）
   - birth直後に、プラグイン親（if any）を `PluginLoaderV2.create_box` で生成→`__plugin_content`格納
   - 注意: 生成に必要な引数はゼロ想定。将来は子birth引数から親引数を抽出する設計を追加

2. メソッド解決のフォールバック（中リスク）
   - 現状の解決順（ユーザー定義 → ビルトイン親）に続けて、プラグイン親を追加
   - `__plugin_content`が `PluginBoxV2` なら、`PluginLoaderV2.invoke_instance_method(parent_box_type, method, instance_id, args)` を実行
   - 引数/戻り値は最小TLV（Int/String/Bool）に限定（VM側と整合）

3. from Parent.method（Interpreter側）の分岐拡張（中リスク）
   - Parentがビルトイン：現状を維持
   - Parentがユーザー定義：現状を維持
   - Parentがプラグイン：`__plugin_content`が存在することを前提に `invoke_instance_method` を呼び出し

4. テスト（小粒E2E、低リスク）
   - ユーザー定義Boxが `from PluginBox` を持つケースで、子メソッドから `from PluginBox.method()` へ到達できること（ゼロ/1引数程度）
   - 直接 `plugin_parent.method()` を呼ばず、子 → 親（プラグイン）呼び出しの透過性を確認

5. ドキュメント（低リスク）
   - `docs/説明書/VM/README.md` に「プラグイン親フォールバック」を追記
   - `docs/説明書/reference` に `__plugin_content` の内部仕様を簡潔に注記（内部予約キー）

## 非スコープ（Out of Scope）
- VM側の `from Parent.method` の完全対応（次フェーズ）
- TLVの型拡張（Float/配列/Box参照など）—次フェーズで段階的に
- プラグイン親のフィールド継承（行わない）

## リスクと対策
- リスク: birth時のプラグイン生成失敗（nyash.toml/共有ライブラリ未整備）
  - 対策: 失敗時は`__plugin_content`未設定で続行し、親メソッド呼び出し時に明示エラー
- リスク: 引数/戻り値のTLVが最小型に留まるため、メソッド範囲に制限
  - 対策: 先に成功体験を提供し、必要に応じて段階拡張（9.78d以降）

## 受け入れ基準（Acceptance Criteria）
- 子Boxが`from PluginBox`を持つ時、子メソッド中の `from PluginBox.method()` がInterpreterで正常に実行される
- `new Child(...).childMethod()` →（内部で）プラグイン親メソッド呼び出しが透過的に成功
- 失敗時にわかりやすいエラー（設定不足/メソッド未定義）
- 既存E2E/ライブラリ型チェックが維持される

## 実装順序（Milestones）
1. InstanceBox `__plugin_content`導入 + birth時格納（1日）
2. メソッド解決フォールバック追加（1～2日）
3. from Parent.method 分岐拡張（1日）
4. 小粒E2E・ドキュメント更新（0.5～1日）

## 備考
- VM側は既に`BoxCall`でPluginメソッドを呼び出せる。Interpreterを先行して整備し、ユーザー体験を揃える。
- 将来は「親メソッドテーブル」の共通インターフェイスをランタイムへ寄せ、Interpreter/VMの実装差分を更に縮小する。
