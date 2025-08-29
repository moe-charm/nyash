# 1ヶ月間の開発タイムライン

## Day 1-3: 言語設計とパーサー
- Everything is Box哲学の確立
- 基本構文設計（box, init, from）
- パーサー実装（無限ループ対策付き）

## Day 4-6: インタープリター実装
- 基本的なBox型（StringBox, IntegerBox, BoolBox）
- メソッド呼び出し機構
- デリゲーション（from構文）実装

## Day 7-9: MIRとVM実装
- MIR設計（26命令）
- AST→MIR変換
- VMインタープリター実装
- **マイルストーン**: Hello Worldが3形態で実行可能

## Day 10-12: 最適化とプラグイン
- VM最適化（FastPath）
- プラグインシステム（BID-FFI）設計
- FileBoxプラグイン実装

## Day 13-15: JIT基盤構築
- Cranelift統合
- MIR→CLIF変換
- 基本的なJITコンパイル
- **マイルストーン**: 13.5倍高速化達成

## Day 16-18: JIT完成とHostCall
- HostCall機構（Array/Map操作）
- JIT最適化（型特殊化）
- フォールバック機構

## Day 19-20: AOTとネイティブEXE
- AOT object生成
- libnyrt.a 実装
- ネイティブリンク成功
- **最終マイルストーン**: スタンドアロンEXE生成

## 驚異的な点
- 各フェーズが2-3日で完了
- 後戻りなしの一直線開発
- 毎フェーズで動作確認可能
- 最終的に5つの実行形態が意味論等価で動作