# Codex Android/Termux セットアップガイド

Date: 2025-09-03
Version: Codex 0.29.0+

## 📱 概要

Codex v0.29.0から、AndroidデバイスでもTermux経由でCodexが使えるようになりました！

## 🤖 Termuxとは？

Termuxは、Android上で動作する**ターミナルエミュレータ + Linux環境**です。
- root化不要
- 本格的なLinux環境
- パッケージマネージャー完備
- 開発ツールが使える

## 📋 セットアップ手順

### 1. Termuxのインストール

1. **F-Droid版を推奨**（Google Play版は更新停止）
   - [F-Droid](https://f-droid.org/)からTermuxをインストール
   - または[Termux公式GitHub](https://github.com/termux/termux-app/releases)からAPKダウンロード

### 2. Termuxの初期設定

```bash
# パッケージリストを更新
pkg update && pkg upgrade

# 必要なパッケージをインストール
pkg install nodejs-lts git openssh
```

### 3. Codexのインストール

```bash
# npmからCodexをインストール
npm install -g @openai/codex

# バージョン確認
codex --version
```

### 4. 認証設定

```bash
# ChatGPTアカウントでログイン（推奨）
codex auth

# または、APIキーを使用
codex auth --api-key
```

### 5. 動作確認

```bash
# チャットモードを開始
codex chat

# 設定確認
codex config
```

## 💡 活用例

### リモートサーバー管理
```bash
# SSHでサーバーに接続
ssh user@server.com

# サーバー上でCodexを使用
codex exec "nginxの設定ファイルを最適化して"
```

### ローカル開発
```bash
# Termux内でプロジェクト作成
mkdir my-project && cd my-project
codex chat --project .
```

### 緊急時のデバッグ
```bash
# エラーログを解析
codex exec "このエラーを解決する方法を教えて: $(tail -n 50 error.log)"
```

## ⚙️ 推奨設定

### Termuxストレージアクセス
```bash
# 外部ストレージへのアクセスを許可
termux-setup-storage
```

### キーボード設定
- 外部キーボード使用を推奨
- またはHacker's Keyboardアプリ

### Termux:API（オプション）
```bash
# Android APIへのアクセス（通知、クリップボード等）
pkg install termux-api
```

## 🚨 注意事項

1. **バッテリー消費**: 長時間使用時は充電器接続推奨
2. **メモリ使用**: 最低2GB RAM推奨
3. **ストレージ**: 1GB以上の空き容量確保
4. **ネットワーク**: 安定したインターネット接続必須

## 🔧 トラブルシューティング

### npmインストールエラー
```bash
# node/npmの再インストール
pkg uninstall nodejs-lts
pkg clean
pkg install nodejs-lts
```

### 認証エラー
```bash
# 設定ファイルをリセット
rm -rf ~/.codex
codex auth
```

### パフォーマンス問題
- Termuxのウェイクロック設定を有効化
- バックグラウンド制限を解除（Android設定）

## 🎉 まとめ

AndroidでCodexが動くことで：
- どこでもAI支援開発が可能
- 緊急時の対応が迅速に
- モバイル開発の新しい可能性

スマホが強力な開発マシンに変身しますにゃ！