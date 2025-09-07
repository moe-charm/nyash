# 📦 Nyash Boxシステム設計ドキュメント

## 🎯 概要

Nyashの核心哲学「**Everything is Box**」に関する完全な設計ドキュメント集。
言語設計の根幹から実装詳細まで、Boxシステムのすべてを網羅しています。

## 📚 ドキュメント構成

### 🌟 コア哲学

#### [everything-is-box.md](everything-is-box.md)
Nyashの核心哲学「Everything is Box」の解説。なぜすべてをBoxにするのか、その設計思想と利点。

### 📖 完全リファレンス

#### [box-reference.md](box-reference.md)  
**統合版Box型完全リファレンス**。全ビルトインBox型のAPI仕様、基本型からプラグインBoxまで。

### 🔄 システム設計

#### [delegation-system.md](delegation-system.md)
完全明示デリゲーションシステムの設計。`from`構文、`override`必須、`pack`構文の詳細仕様。

#### [memory-finalization.md](memory-finalization.md)
**統合版メモリ管理&finiシステム**。Arc<Mutex>一元管理、fini()論理的解放、weak参照、プラグインメモリ安全性。

## 🔗 関連ドキュメント

- **[プラグインシステム](../plugin-system/)**: BID-FFIプラグインシステム完全仕様
- **[言語仕様](../core-language/)**: デリゲーション構文、言語リファレンス
- **[実行バックエンド](../execution-backend/)**: MIR、P2P通信仕様

## 🎨 設計原則

### Everything is Box
- すべての値がBoxオブジェクト
- 統一的なメソッド呼び出し
- プリミティブ型と参照型の区別なし

### メモリ安全性
- Arc<Mutex>による統一管理
- fini()による決定論的リソース解放
- weak参照による循環参照回避

### プラグイン拡張性
- BID-FFIによる外部ライブラリ統合
- 型情報管理による安全な変換
- HostVtableによるメモリ管理

---

**最終更新**: 2025年8月19日 - boxes-system統合整理完了  
**Phase 9.75g-0成果**: プラグインシステムとの完全統合