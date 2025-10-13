# Golden Tests アーカイブ (2025-09)

**日付**: 2025-10-12
**ステータス**: アーカイブ（未使用テストファイル）

## 概要

2025年9月28日に作成されたMIR命令の期待値ファイル（ゴールデンテスト）です。

## ファイル一覧

1. `await_simple.mir.txt` (167 bytes)
2. `boxcall_array_getset.mir.txt` (1,905 bytes)
3. `extern_console_log.mir.txt` (740 bytes)
4. `loop_nested_if.mir.txt` (1,445 bytes)
5. `loop_simple.mir.txt` (1,125 bytes)
6. `typeop_in_if_loop_poc.mir.txt` (1,809 bytes)
7. `typeop_is_as_func_poc.mir.txt` (808 bytes)
8. `typeop_is_as_poc.mir.txt` (902 bytes)
9. `typeop_mixed.mir.txt` (1,320 bytes)

**合計**: 9ファイル、10,221 bytes

## 用途（推測）

これらのファイルは以下のMIR命令の正しい出力を記録したものと思われます：
- `await` - 非同期待機
- `boxcall` - Boxメソッド呼び出し
- `externcall` - 外部関数呼び出し
- `loop` - ループ構造
- `typeop` - 型操作（is/as）

## なぜアーカイブしたか

**調査結果**（2025-10-12）:
- ✅ 最終更新: 2025-09-28（2週間前）
- ❌ コードベース内で参照なし（Rustコード、シェルスクリプト共に0件）
- ❌ 対応するテストコードが見つからない
- ❌ 他の場所に `.mir.txt` ファイルなし

**判定**: 実験的に作成されたが、実際のテストスイートに統合されなかった

## 将来の活用可能性

これらのファイルは以下の用途で再利用できる可能性があります：
- MIR出力の回帰テスト実装時
- MIR命令セットの正当性検証
- ドキュメント・教材としての活用

## 関連情報

- MIR命令セット: `docs/reference/mir/INSTRUCTION_SET.md`
- スモークテスト: `tools/smokes/`
- Phase 15.8: WASM実装（MIR出力検証）

## アーカイブ日

2025-10-12: docs/development/testing/golden/ から移動
