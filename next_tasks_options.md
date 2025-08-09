# 🎯 Nyash開発 - 次のタスク候補

## 1. 🔧 パーサーリファクタリング継続
**declarations.rs作成** - 残り1,249行のmod.rsをさらに分割
- parse_box_declaration
- parse_function_declaration  
- parse_interface_box_declaration
- parse_static_declaration
- parse_global_var
利点: コード整理完了、保守性最大化

## 2. 🎨 新規アプリケーション開発
**実用的なNyashアプリを作る**
- 🐍 Snakeゲーム - ArrayBox/ゲームループ活用
- 📁 ファイル整理ツール - FileBox/パターンマッチング
- 🎮 Tetris - 2次元配列/タイマー/キー入力
- 📊 CSVビューア - ファイル処理/テーブル表示
利点: 言語の実用性実証、バグ発見

## 3. 🌉 extern box実装
**FFI基盤でネイティブ連携**
- C/C++関数呼び出し
- 外部ライブラリ統合
- GUI基盤準備
利点: 実用アプリの可能性拡大

## 4. 📚 標準ライブラリ拡充
**基本機能の充実**
- StringBox拡張 (split/join/regex)
- ArrayBox拡張 (map/filter/reduce)
- FileBox拡張 (ディレクトリ操作)
- NetworkBox実装 (HTTP/Socket)
利点: 開発効率向上

## 5. 🚀 パフォーマンス最適化
**実行速度改善**
- バイトコードコンパイラ
- JITコンパイラ検討
- メモリ管理最適化
利点: 実用レベルの性能確保

## 6. 🧪 テストフレームワーク
**品質保証基盤**
- assert/expect実装
- テストランナー
- カバレッジ測定
利点: 安定性向上

どれが一番楽しそう/必要そうかにゃ？