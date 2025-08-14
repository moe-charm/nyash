# 🔧 Box設計の現在の課題

## 📅 最終更新: 2025-08-14

このドキュメントは、Nyash Box設計における現在進行中の技術的課題と対応状況をまとめています。

## 🚨 Critical: Arc<Mutex>責務の二重化問題

### 問題の概要
現在、15個のBox型において、内部と外部で二重にロック管理が行われています。

```rust
// 🚨 現在の問題構造
pub struct SocketBox {
    listener: Arc<Mutex<Option<TcpListener>>>,  // 内部ロック
    is_server: Arc<Mutex<bool>>,                // 内部ロック
}

// インタープリター側
Arc<Mutex<SocketBox>>  // 外部ロック

// 結果: 二重ロック → デッドロック・状態不整合
```

### 影響を受けるBox型（15個）
1. **ネットワーク系**: SocketBox, HTTPServerBox
2. **コレクション系**: ArrayBox, MapBox, BufferBox
3. **I/O系**: FileBox, StreamBox
4. **P2P系**: P2PBox, IntentBox, SimpleIntentBox
5. **GUI系**: EguiBox
6. **特殊系**: RandomBox, DebugBox, FutureBox, JSONBox

### 根本原因
- **責務の混在**: Box自身がスレッドセーフティを管理
- **設計の不統一**: 各Box実装者が独自にArc<Mutex>を使用
- **ガイドライン不足**: 正しいBox実装パターンが未確立

### 対応計画
**Phase 9.75**として緊急対応中。詳細は[phase-9-75-redesign.md](phase-9-75-redesign.md)参照。

## ⚠️ High: SocketBox状態保持問題

### 問題の詳細
SocketBoxで`bind()`後に`isServer()`を呼ぶと、状態が保持されていない。

```nyash
// 期待される動作
server = new SocketBox()
server.bind("127.0.0.1", 8080)  // true
server.isServer()                // true であるべき

// 実際の動作
server.isServer()                // false （状態が失われる）
```

### 技術的分析
詳細は[socket-box-problem.md](socket-box-problem.md)参照。

### 暫定対策
- PR #75でArc参照共有を試みたが、根本解決には至らず
- デッドロック問題は解決したが、状態保持問題は継続

## 🟡 Medium: Box型の増殖管理

### 現状
- 基本実装済みBox: 20種類以上
- 各Boxが独自の実装パターン
- 統一的な品質管理が困難

### 課題
1. **実装の一貫性**: 各Boxで異なる実装スタイル
2. **テストカバレッジ**: Box間でテスト密度にばらつき
3. **ドキュメント**: API仕様の記述レベルが不統一

### 対応方針
- Box実装テンプレートの作成
- 自動テスト生成ツールの検討
- APIドキュメント自動生成

## 🟢 Low: パフォーマンス最適化

### 観測された問題
- 二重ロックによるパフォーマンス低下
- Box生成時のオーバーヘッド
- メソッド呼び出しの動的ディスパッチコスト

### 最適化の機会
1. **ロック削減**: Arc<Mutex>一元化で改善見込み
2. **Box生成**: オブジェクトプールの検討
3. **メソッド呼び出し**: インライン化・特殊化

## 📊 課題の優先順位

1. **🔴 最優先**: Arc<Mutex>責務一元化（Phase 9.75）
2. **🟠 高優先**: SocketBox状態保持問題の根本解決
3. **🟡 中優先**: Box実装ガイドライン策定
4. **🟢 低優先**: パフォーマンス最適化

## 🔄 進捗追跡

### 2025-08-14
- Phase 9.75として「Box設計根本革命」を`copilot_issues.txt`に追加
- Box設計ドキュメントフォルダを新規作成
- 現在の課題を体系的に整理

### 今後の予定
- Phase 9.75 Phase A: 設計ガイドライン策定（3日）
- Phase 9.75 Phase B: 最優先Box修正（1週間）
- Phase 9.75 Phase C: ステートフルBox修正（1週間）
- Phase 9.75 Phase D: 残りのBox統一（3日）

---

関連ドキュメント：
- [Phase 9.75実装計画](phase-9-75-redesign.md)
- [SocketBox問題詳細](socket-box-problem.md)
- [Box設計原則](../memory-management.md)