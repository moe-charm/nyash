# Phase 21: データベース駆動開発（DDD: Database-Driven Development）

## 📋 概要

ソースコードをファイルではなくデータベースで管理する革命的開発パラダイム。
Box、メソッド、名前空間を構造化データとして扱い、リファクタリングを瞬時に完了させる。
**「ファイルは1970年代の遺物。21世紀のコードは構造化データベースに住む」**

## 🎯 背景と動機

### 現状の問題
- **ファイルベース**：物理的な区切り（人間の都合）
- **Box/メソッド**：論理的な単位（プログラムの本質）
- **不一致の結果**：リファクタリングが遅い、検索が非効率、依存関係が不透明

### 解決策
- コードをSQLiteデータベースで管理
- Box、メソッド、依存関係を正規化されたテーブルで表現
- SQLクエリでリファクタリング・検索・分析を高速化

## 🏗️ データベーススキーマ

### 基本テーブル構造

```sql
-- Boxの定義
CREATE TABLE boxes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    namespace TEXT,
    parent_box_id INTEGER,
    box_type TEXT CHECK(box_type IN ('normal', 'static', 'abstract')),
    source_code TEXT,
    metadata JSON,  -- 型情報、アノテーション、ドキュメント
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_box_id) REFERENCES boxes(id)
);

-- メソッド定義
CREATE TABLE methods (
    id INTEGER PRIMARY KEY,
    box_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    visibility TEXT CHECK(visibility IN ('public', 'private', 'protected')),
    params JSON,  -- パラメータ情報
    return_type JSON,
    body TEXT,
    mir_cache BLOB,  -- コンパイル済みMIRをキャッシュ
    optimization_hints JSON,
    FOREIGN KEY (box_id) REFERENCES boxes(id),
    UNIQUE(box_id, name)  -- 同一Box内でメソッド名は一意
);

-- フィールド定義
CREATE TABLE fields (
    id INTEGER PRIMARY KEY,
    box_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    field_type JSON,
    default_value TEXT,
    metadata JSON,
    FOREIGN KEY (box_id) REFERENCES boxes(id)
);

-- 依存関係
CREATE TABLE dependencies (
    from_type TEXT CHECK(from_type IN ('box', 'method')),
    from_id INTEGER,
    to_type TEXT CHECK(to_type IN ('box', 'method')),
    to_id INTEGER,
    dep_type TEXT CHECK(dep_type IN ('uses', 'extends', 'calls', 'implements')),
    metadata JSON,  -- 呼び出し位置、使用頻度など
    PRIMARY KEY (from_type, from_id, to_type, to_id, dep_type)
);

-- 名前空間
CREATE TABLE namespaces (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    parent_id INTEGER,
    metadata JSON,
    FOREIGN KEY (parent_id) REFERENCES namespaces(id)
);

-- コンパイルキャッシュ
CREATE TABLE compile_cache (
    id INTEGER PRIMARY KEY,
    entity_type TEXT,
    entity_id INTEGER,
    mir_version INTEGER,
    mir_data BLOB,
    metadata JSON,  -- 最適化レベル、ターゲットなど
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 全文検索用インデックス
CREATE VIRTUAL TABLE code_search USING fts5(
    entity_type,
    entity_id,
    name,
    content,
    tokenize = 'porter'
);
```

## 🚀 革命的な機能

### 1. 瞬間リファクタリング

```sql
-- 名前変更：トランザクション一発
BEGIN TRANSACTION;
UPDATE boxes SET name = 'NewBoxName' WHERE name = 'OldBoxName';
UPDATE code_search SET name = 'NewBoxName' 
    WHERE entity_type = 'box' AND name = 'OldBoxName';
-- 依存コードも自動更新（トリガーで実装）
COMMIT;

-- メソッド移動：Box間でメソッドを移動
UPDATE methods SET box_id = (SELECT id FROM boxes WHERE name = 'TargetBox')
    WHERE id = ? AND box_id = ?;
```

### 2. 高度な検索・分析

```sql
-- 未使用コード検出
SELECT b.namespace || '.' || b.name AS unused_box
FROM boxes b
LEFT JOIN dependencies d ON 
    (d.to_type = 'box' AND d.to_id = b.id)
WHERE d.from_id IS NULL;

-- 循環依存検出（再帰CTE）
WITH RECURSIVE dep_path AS (
    SELECT from_id, to_id, 
           from_id || '->' || to_id as path
    FROM dependencies
    WHERE from_type = 'box' AND to_type = 'box'
    UNION ALL
    SELECT d.from_id, dp.to_id, 
           dp.path || '->' || d.to_id
    FROM dependencies d
    JOIN dep_path dp ON d.to_id = dp.from_id
    WHERE d.from_type = 'box' AND d.to_type = 'box'
      AND dp.path NOT LIKE '%' || d.to_id || '%'
)
SELECT path FROM dep_path WHERE from_id = to_id;

-- 類似コード検出（全文検索）
SELECT b1.name AS box1, m1.name AS method1,
       b2.name AS box2, m2.name AS method2,
       similarity_score(m1.body, m2.body) AS similarity
FROM methods m1
JOIN methods m2 ON m1.id < m2.id
JOIN boxes b1 ON m1.box_id = b1.id
JOIN boxes b2 ON m2.box_id = b2.id
WHERE similarity_score(m1.body, m2.body) > 0.8;
```

### 3. インテリジェントなキャッシング

```sql
-- 変更影響分析
CREATE TRIGGER invalidate_cache_on_method_update
AFTER UPDATE ON methods
BEGIN
    -- 直接依存するエンティティのキャッシュを無効化
    DELETE FROM compile_cache
    WHERE entity_id IN (
        SELECT from_id FROM dependencies
        WHERE to_type = 'method' AND to_id = NEW.id
    );
END;
```

### 4. バージョン管理の統合

```sql
-- 変更履歴
CREATE TABLE history (
    id INTEGER PRIMARY KEY,
    entity_type TEXT,
    entity_id INTEGER,
    version INTEGER,
    change_type TEXT,
    old_value TEXT,
    new_value TEXT,
    changed_by TEXT,
    changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    commit_message TEXT
);

-- Git風のブランチ管理
CREATE TABLE branches (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    base_version INTEGER,
    is_active BOOLEAN DEFAULT TRUE
);
```

## 🎨 実装例

```nyash
box CodeDB {
    db: SQLiteBox
    cache: MapBox
    
    birth(dbPath) {
        me.db = new SQLiteBox(dbPath)
        me.cache = new MapBox()
        me.initSchema()
    }
    
    // Boxを保存
    saveBox(box) {
        local tx = me.db.beginTransaction()
        try {
            local boxId = me.db.insert("boxes", {
                name: box.name,
                namespace: box.namespace,
                source_code: box.toString(),
                metadata: box.getMetadata()
            })
            
            // メソッドも保存
            for method in box.methods {
                me.saveMethod(boxId, method)
            }
            
            tx.commit()
        } catch (e) {
            tx.rollback()
            throw e
        }
    }
    
    // リファクタリング：名前変更
    renameBox(oldName, newName) {
        me.db.execute(
            "UPDATE boxes SET name = ? WHERE name = ?",
            [newName, oldName]
        )
        
        // 全文検索インデックスも更新
        me.updateSearchIndex()
        
        // キャッシュ無効化
        me.invalidateCache(oldName)
    }
    
    // 未使用コード検出
    findUnusedCode() {
        return me.db.query("
            SELECT b.namespace || '.' || b.name AS unused
            FROM boxes b
            LEFT JOIN dependencies d ON d.to_id = b.id
            WHERE d.from_id IS NULL
        ")
    }
    
    // AI連携：類似コード提案
    suggestRefactoring(methodId) {
        local similar = me.findSimilarMethods(methodId)
        if similar.length() > 3 {
            return {
                suggestion: "共通Boxに抽出",
                methods: similar
            }
        }
    }
}
```

## 🔧 開発ツール

### 1. CLI拡張

```bash
# DBクエリ実行
nyash db query "SELECT * FROM boxes WHERE name LIKE '%Handler'"

# リファクタリング
nyash db refactor rename-box OldName NewName

# 依存関係グラフ生成
nyash db deps --format=dot | dot -Tpng -o deps.png

# 未使用コード削除
nyash db clean --remove-unused --dry-run
```

### 2. VSCode拡張

- **DBエクスプローラー**：Box/メソッドをツリー表示
- **リアルタイム検索**：SQLクエリで即座に検索
- **依存関係ビュー**：グラフィカルに表示
- **リファクタリングパレット**：右クリックで瞬間実行

### 3. Web UI

```nyash
box CodeDBWebUI {
    server: WebServerBox
    db: CodeDB
    
    birth(dbPath, port) {
        me.db = new CodeDB(dbPath)
        me.server = new WebServerBox(port)
        me.setupRoutes()
    }
    
    setupRoutes() {
        // コードグラフ表示
        me.server.get("/graph") { req, res ->
            local deps = me.db.getAllDependencies()
            res.json(me.buildD3Graph(deps))
        }
        
        // リアルタイムSQL実行
        me.server.post("/query") { req, res ->
            local result = me.db.query(req.body.sql)
            res.json(result)
        }
    }
}
```

## 📊 移行戦略

### Phase 1: ハイブリッドモード（3ヶ月）
- 既存ファイル→DB同期ツール開発
- DB→ファイルエクスポート（Git互換性維持）
- 開発者が徐々に慣れる期間

### Phase 2: DB優先モード（3ヶ月）
- 新規開発はDB直接
- ファイルは自動生成
- リファクタリング効率を体感

### Phase 3: 完全DB化（3ヶ月）
- ファイルシステムは配布用のみ
- 開発は100% DB駆動
- 新しい開発パラダイムの確立

## 🌟 期待される効果

### 開発効率
- **リファクタリング**: 100倍高速化（秒単位→ミリ秒単位）
- **検索**: SQLによる高度な検索（正規表現、構造検索）
- **分析**: 依存関係、複雑度、類似性を瞬時に把握

### コード品質
- **重複排除**: 類似コードを自動検出
- **整合性**: DB制約で不整合を防止
- **追跡可能性**: すべての変更を記録

### AI連携
- **構造化データ**: AIが理解しやすい
- **メタデータ**: 型情報、使用頻度など豊富
- **学習効率**: コードパターンを効率的に学習

## 🚀 革新性

### 世界初の要素
1. **完全DB駆動言語**: ファイルシステムからの解放
2. **構造認識エディタ**: Box/メソッド単位の編集
3. **瞬間リファクタリング**: SQLトランザクションで完結
4. **依存関係DB**: コンパイル時情報も含む

### 技術的優位性
- **SQLite**: 軽量、高速、信頼性
- **Everything is Box**: DB表現と相性抜群
- **MIRキャッシュ**: コンパイル高速化

## 📅 実施時期

- **開始条件**: Phase 15（セルフホスティング）完了後
- **推定期間**: 9ヶ月
- **優先度**: 高（開発効率の革命的向上）

## 🔗 関連フェーズ

- [Phase 15: セルフホスティング](../phase-15/) - 基盤技術
- [Phase 12: 統一実行パス](../phase-12/) - MIRキャッシュ活用
- [Phase 16: プラグインエコシステム](../phase-16/) - DB APIの公開

---

> 「コードはファイルに書くもの」という固定観念を打ち破る。
> 21世紀の開発は、構造化データベースで行うべきだにゃ！

## 📚 関連ドキュメント

### Phase 21の進化過程
- [技術的考慮事項](technical-considerations.md) - 詳細な技術検討
- [可逆変換アプローチ](reversible-conversion.md) - Git互換性を保つ方法
- [箱データベース構想v2](README_v2.md) - シンプル化された実装
- [自己解析アプローチ](self-parsing-approach.md) - Nyashの自己パース能力活用

### 学術的評価
- **[AI評価フォルダ](ai-evaluation/)** - Gemini/Codexによる詳細な評価
  - [Gemini評価](ai-evaluation/gemini-evaluation.md) - 完全な学術的分析
  - [Codex評価（部分）](ai-evaluation/codex-evaluation-partial.md) - 深い思考過程
  - [評価サマリー](ai-evaluation/evaluation-summary.md) - 統合的な分析