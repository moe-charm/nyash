# BID Converter from Copilot

このフォルダには、Copilotさんが実装してくれたBID (Box Interface Definition) の変換部分を保存しています。

## 📦 含まれるファイル

- **tlv.rs**: TLV (Type-Length-Value) エンコード/デコード実装
- **types.rs**: BID型定義（NyashValue変換等）
- **error.rs**: BIDエラー型定義

## 🎯 用途

将来的にnyash2.tomlを実装する際に、以下の用途で活用予定：

1. **型変換**: Nyash型 ↔ BID型の相互変換
2. **シリアライズ**: プラグイン通信用のデータ変換
3. **エラーハンドリング**: 統一的なエラー処理

## 💡 なぜ保存？

- CopilotさんのTLV実装は汎用的で再利用価値が高い
- 現在のnyash.tomlベースの実装をシンプルに保ちつつ、将来の拡張に備える
- プラグイン間通信やネットワーク通信でも活用可能

## 📝 メモ

- 現在は使用していない（既存のnyash.tomlベースが動作中）
- Phase 9.8以降で活用予定
- 他言語プラグイン対応時には必須になる可能性