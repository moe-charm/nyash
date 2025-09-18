# Phase 21: 技術的考慮事項

## 🏗️ アーキテクチャ設計

### レイヤー構造

```
┌─────────────────────────────┐
│     開発ツール層            │ (VSCode, CLI, Web UI)
├─────────────────────────────┤
│     API層                   │ (GraphQL/REST)
├─────────────────────────────┤
│     CodeDB抽象層            │ (統一インターフェース)
├─────────────────────────────┤
│     SQLite実装層            │ (具体的なDB操作)
├─────────────────────────────┤
│     ストレージ層            │ (ローカル/リモート)
└─────────────────────────────┘
```

## 🔐 セキュリティ考慮事項

### アクセス制御
```sql
-- ユーザー権限管理
CREATE TABLE permissions (
    user_id INTEGER,
    resource_type TEXT,
    resource_id INTEGER,
    permission TEXT CHECK(permission IN ('read', 'write', 'admin')),
    granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    granted_by INTEGER,
    PRIMARY KEY (user_id, resource_type, resource_id, permission)
);

-- 監査ログ
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    action TEXT,
    resource_type TEXT,
    resource_id INTEGER,
    old_value TEXT,
    new_value TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT,
    session_id TEXT
);
```

### SQLインジェクション対策
```nyash
box SecureCodeDB from CodeDB {
    // パラメータ化クエリを強制
    query(sql, params) {
        // SQLをパースして危険な構文をチェック
        local ast = me.parseSql(sql)
        if me.hasDangerousPattern(ast) {
            throw new SecurityError("Dangerous SQL pattern detected")
        }
        
        return from CodeDB.query(sql, params)
    }
    
    // ホワイトリスト方式のテーブル名検証
    validateTableName(name) {
        if not name.matches("^[a-z_]+$") {
            throw new SecurityError("Invalid table name")
        }
    }
}
```

## 🚀 パフォーマンス最適化

### インデックス戦略
```sql
-- 頻繁なクエリ用インデックス
CREATE INDEX idx_boxes_namespace ON boxes(namespace);
CREATE INDEX idx_methods_box_id ON methods(box_id);
CREATE INDEX idx_deps_from ON dependencies(from_type, from_id);
CREATE INDEX idx_deps_to ON dependencies(to_type, to_id);

-- 複合インデックス
CREATE INDEX idx_box_namespace_name ON boxes(namespace, name);
CREATE INDEX idx_method_box_name ON methods(box_id, name);

-- 部分インデックス（アクティブなものだけ）
CREATE INDEX idx_active_boxes ON boxes(name) 
    WHERE deleted_at IS NULL;
```

### クエリ最適化
```nyash
box QueryOptimizer {
    cache: MapBox
    
    // クエリ結果のキャッシング
    cachedQuery(sql, params, ttl) {
        local key = me.hash(sql + params.toString())
        
        if me.cache.has(key) {
            local cached = me.cache.get(key)
            if cached.timestamp + ttl > now() {
                return cached.result
            }
        }
        
        local result = me.db.query(sql, params)
        me.cache.set(key, {
            result: result,
            timestamp: now()
        })
        
        return result
    }
}
```

## 🔄 同期・レプリケーション

### マルチデバイス同期
```nyash
box CodeDBSync {
    local: CodeDB
    remote: RemoteCodeDB
    
    // 変更を追跡
    trackChanges() {
        CREATE TRIGGER track_box_changes
        AFTER INSERT OR UPDATE OR DELETE ON boxes
        BEGIN
            INSERT INTO sync_queue (
                table_name, operation, entity_id, data
            ) VALUES (
                'boxes', 
                CASE 
                    WHEN OLD.id IS NULL THEN 'INSERT'
                    WHEN NEW.id IS NULL THEN 'DELETE'
                    ELSE 'UPDATE'
                END,
                COALESCE(NEW.id, OLD.id),
                json_object('old', OLD, 'new', NEW)
            );
        END;
    }
    
    // 差分同期
    sync() {
        local changes = me.local.query("
            SELECT * FROM sync_queue 
            WHERE synced_at IS NULL 
            ORDER BY created_at
        ")
        
        for change in changes {
            me.remote.applyChange(change)
            me.local.markSynced(change.id)
        }
    }
}
```

## 🎯 互換性戦略

### ファイルシステムとの相互変換
```nyash
box FileDBBridge {
    // DB→ファイル エクスポート
    exportToFiles(outputDir) {
        local boxes = me.db.query("SELECT * FROM boxes")
        
        for box in boxes {
            local path = outputDir + "/" + 
                        box.namespace.replace(".", "/") + "/" +
                        box.name + ".nyash"
            
            local file = new FileBox(path)
            file.write(me.generateFileContent(box))
        }
    }
    
    // ファイル→DB インポート
    importFromFiles(sourceDir) {
        local files = FileBox.glob(sourceDir + "/**/*.nyash")
        
        me.db.beginTransaction()
        try {
            for file in files {
                local ast = Parser.parse(file.read())
                me.importAST(ast, file.path)
            }
            me.db.commit()
        } catch (e) {
            me.db.rollback()
            throw e
        }
    }
}
```

## 🔍 高度な分析機能

### コードメトリクス
```sql
-- 循環的複雑度の計算
CREATE VIEW method_complexity AS
SELECT 
    m.id,
    b.name || '.' || m.name as full_name,
    (
        SELECT COUNT(*) 
        FROM json_each(m.body) 
        WHERE value LIKE '%if%' 
           OR value LIKE '%loop%'
           OR value LIKE '%catch%'
    ) + 1 as cyclomatic_complexity
FROM methods m
JOIN boxes b ON m.box_id = b.id;

-- コード行数統計
CREATE VIEW code_stats AS
SELECT
    COUNT(DISTINCT b.id) as total_boxes,
    COUNT(DISTINCT m.id) as total_methods,
    SUM(LENGTH(m.body) - LENGTH(REPLACE(m.body, char(10), ''))) as total_lines,
    AVG(LENGTH(m.body) - LENGTH(REPLACE(m.body, char(10), ''))) as avg_method_lines
FROM boxes b
LEFT JOIN methods m ON b.id = m.box_id;
```

### 依存関係の可視化
```nyash
box DependencyAnalyzer {
    // 影響範囲分析
    getImpactedEntities(changedEntity) {
        return me.db.query("
            WITH RECURSIVE impacted AS (
                -- 直接依存
                SELECT to_type, to_id, 1 as level
                FROM dependencies
                WHERE from_type = ? AND from_id = ?
                
                UNION
                
                -- 推移的依存
                SELECT d.to_type, d.to_id, i.level + 1
                FROM dependencies d
                JOIN impacted i ON 
                    d.from_type = i.to_type AND 
                    d.from_id = i.to_id
                WHERE i.level < 5  -- 最大5階層まで
            )
            SELECT DISTINCT * FROM impacted
            ORDER BY level
        ", [changedEntity.type, changedEntity.id])
    }
}
```

## 🌐 分散開発対応

### ブランチ・マージ戦略
```sql
-- ブランチ管理
CREATE TABLE branches (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    base_commit_id INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by INTEGER,
    is_active BOOLEAN DEFAULT TRUE
);

-- コミット（変更セット）
CREATE TABLE commits (
    id INTEGER PRIMARY KEY,
    branch_id INTEGER,
    parent_commit_id INTEGER,
    message TEXT,
    author INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    changes JSON,  -- 変更の詳細
    FOREIGN KEY (branch_id) REFERENCES branches(id)
);
```

### コンフリクト解決
```nyash
box ConflictResolver {
    // 3-way マージ
    merge(base, mine, theirs) {
        if mine == theirs {
            return mine  // 変更なし or 同じ変更
        }
        
        if base == mine {
            return theirs  // 相手のみ変更
        }
        
        if base == theirs {
            return mine  // 自分のみ変更
        }
        
        // 両方変更 - コンフリクト
        return me.resolveConflict(base, mine, theirs)
    }
    
    resolveConflict(base, mine, theirs) {
        // AST レベルでのマージを試みる
        local baseAST = Parser.parse(base)
        local mineAST = Parser.parse(mine)
        local theirsAST = Parser.parse(theirs)
        
        // メソッド単位でマージ可能か確認
        if me.canMergeAtMethodLevel(baseAST, mineAST, theirsAST) {
            return me.mergeASTs(baseAST, mineAST, theirsAST)
        }
        
        // マージ不可 - ユーザーに選択させる
        throw new MergeConflict(base, mine, theirs)
    }
}
```

## 📊 メトリクス・モニタリング

### パフォーマンス追跡
```sql
-- クエリパフォーマンスログ
CREATE TABLE query_performance (
    id INTEGER PRIMARY KEY,
    query_hash TEXT,
    query_text TEXT,
    execution_time_ms INTEGER,
    rows_affected INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- DB統計情報
CREATE VIEW db_stats AS
SELECT
    (SELECT COUNT(*) FROM boxes) as total_boxes,
    (SELECT COUNT(*) FROM methods) as total_methods,
    (SELECT COUNT(*) FROM dependencies) as total_dependencies,
    (SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()) as db_size_bytes,
    (SELECT COUNT(*) FROM compile_cache) as cached_compilations;
```

## 🔮 将来の拡張性

### プラグインアーキテクチャ
```nyash
box CodeDBPlugin {
    // フック機能
    hooks: MapBox
    
    register(event, handler) {
        if not me.hooks.has(event) {
            me.hooks.set(event, new ArrayBox())
        }
        me.hooks.get(event).push(handler)
    }
    
    trigger(event, data) {
        if me.hooks.has(event) {
            for handler in me.hooks.get(event) {
                handler(data)
            }
        }
    }
}

// 使用例：自動フォーマッター
box AutoFormatter from CodeDBPlugin {
    birth() {
        me.register("before_save", me.formatCode)
    }
    
    formatCode(data) {
        if data.entity_type == "method" {
            data.body = Formatter.format(data.body)
        }
    }
}
```

### AI統合の準備
```sql
-- ベクトル埋め込み保存
CREATE TABLE embeddings (
    id INTEGER PRIMARY KEY,
    entity_type TEXT,
    entity_id INTEGER,
    embedding BLOB,  -- float配列をBLOBで保存
    model_version TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 類似性検索用の仮想テーブル（将来的にベクトル検索エンジンと統合）
CREATE VIRTUAL TABLE vector_search USING vector_index(
    embedding FLOAT[768]
);
```

---

これらの技術的考慮事項を踏まえて、段階的に実装を進めることで、
安全で高性能なデータベース駆動開発環境を実現できるにゃ！