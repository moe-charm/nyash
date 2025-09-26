# Phase 21: ソースコード⇔データベース完全可逆変換システム

## 📋 概要

データベース駆動開発の最大の課題であるGit互換性を、**完全可逆変換**によって根本的に解決する革新的アプローチ。
ソースコードとデータベースを自在に行き来できることで、両方の利点を最大限に活用する。

## 🎯 核心的なアイデア

```
ソースコード（.nyash） ⇔ データベース（SQLite）
       ↓                    ↓
    Git管理             高速リファクタリング
    エディタ編集         構造化分析
    既存ツール互換       AI最適化
```

**重要な原則：**
- ソースコード → DB → ソースコードで100%元に戻る（情報の欠落なし）
- 開発者は好きな方式（ファイルまたはDB）を自由に選択可能
- Git運用は完全に従来通り（テキストファイルとしてコミット）

## 🏗️ 技術設計

### 1. 完全可逆変換の要件

```nyash
box ReversibleConverter {
    // 変換の基本原則
    verify(sourceCode) {
        local db = me.sourceToDb(sourceCode)
        local restored = me.dbToSource(db)
        return sourceCode == restored  // 必ずtrue
    }
}
```

### 2. メタデータの完全保存

```sql
-- コード構造
CREATE TABLE code_structure (
    id INTEGER PRIMARY KEY,
    entity_type TEXT, -- 'box', 'method', 'field'
    entity_id INTEGER,
    source_order INTEGER,
    indentation_level INTEGER,
    line_start INTEGER,
    line_end INTEGER,
    column_start INTEGER,
    column_end INTEGER
);

-- スタイル情報
CREATE TABLE style_metadata (
    id INTEGER PRIMARY KEY,
    entity_id INTEGER,
    whitespace_before TEXT,
    whitespace_after TEXT,
    line_endings TEXT, -- '\n' or '\r\n'
    indentation_style TEXT, -- 'space' or 'tab'
    indentation_width INTEGER
);

-- コメント保存
CREATE TABLE comments (
    id INTEGER PRIMARY KEY,
    entity_id INTEGER,
    comment_type TEXT, -- 'line', 'block', 'doc'
    content TEXT,
    position TEXT, -- 'before', 'after', 'inline'
    line_number INTEGER,
    column_number INTEGER
);

-- 元のソース（差分検証用）
CREATE TABLE original_sources (
    file_path TEXT PRIMARY KEY,
    content_hash TEXT,
    full_content TEXT,
    last_synced TIMESTAMP
);
```

### 3. 変換アルゴリズム

#### ソース → DB

```nyash
box SourceToDbConverter {
    convert(filePath, sourceCode) {
        // 1. AST解析
        local ast = Parser.parseWithFullInfo(sourceCode)
        
        // 2. 構造抽出
        local boxes = me.extractBoxes(ast)
        local methods = me.extractMethods(ast)
        local dependencies = me.analyzeDependencies(ast)
        
        // 3. メタデータ抽出
        local metadata = {
            comments: me.extractComments(sourceCode),
            whitespace: me.extractWhitespace(sourceCode),
            style: me.detectCodingStyle(sourceCode),
            positions: me.mapSourcePositions(ast)
        }
        
        // 4. DB保存（トランザクション）
        me.db.transaction {
            me.saveStructure(boxes, methods)
            me.saveMetadata(metadata)
            me.saveDependencies(dependencies)
            me.saveOriginal(filePath, sourceCode)
        }
    }
}
```

#### DB → ソース

```nyash
box DbToSourceConverter {
    convert(filePath) {
        // 1. 構造読み込み
        local structure = me.db.loadStructure(filePath)
        local metadata = me.db.loadMetadata(filePath)
        
        // 2. ソース再構築
        local builder = new SourceBuilder(metadata.style)
        
        for entity in structure.entities {
            // 元の位置情報を使って再配置
            builder.addEntity(entity, metadata.positions[entity.id])
            
            // コメントの復元
            for comment in metadata.comments[entity.id] {
                builder.addComment(comment)
            }
            
            // 空白の復元
            builder.applyWhitespace(metadata.whitespace[entity.id])
        }
        
        return builder.toString()
    }
}
```

### 4. スタイルの扱い

```nyash
box StylePreserver {
    modes: {
        EXACT: "完全保持",      // 空白・改行すべて元通り
        NORMALIZE: "正規化",    // フォーマッタ適用
        HYBRID: "ハイブリッド"  // コメント保持＋コード正規化
    }
    
    preserveStyle(source, mode) {
        switch mode {
            case EXACT:
                return me.captureEverything(source)
            case NORMALIZE:
                return me.formatCode(source)
            case HYBRID:
                return me.preserveComments(me.formatCode(source))
        }
    }
}
```

## 🔄 同期メカニズム

### 1. リアルタイム同期

```nyash
box FileSyncDaemon {
    watchers: MapBox
    
    birth() {
        me.watchers = new MapBox()
    }
    
    watch(directory) {
        local watcher = new FileWatcher(directory)
        
        watcher.on("change") { event ->
            if event.file.endsWith(".nyash") {
                me.syncFileToDb(event.file)
            }
        }
        
        watcher.on("db_change") { event ->
            if not event.fromFile {
                me.syncDbToFile(event.entity)
            }
        }
        
        me.watchers.set(directory, watcher)
    }
}
```

### 2. Git統合

```bash
# .git/hooks/pre-commit
#!/bin/bash
nyash sync --db-to-files --verify

# .git/hooks/post-checkout  
#!/bin/bash
nyash sync --files-to-db --incremental

# .git/hooks/post-merge
#!/bin/bash
nyash sync --files-to-db --full
```

### 3. 差分最適化

```sql
-- 変更追跡
CREATE TABLE sync_status (
    entity_id INTEGER PRIMARY KEY,
    file_modified TIMESTAMP,
    db_modified TIMESTAMP,
    sync_status TEXT, -- 'synced', 'file_newer', 'db_newer', 'conflict'
    last_sync_hash TEXT
);

-- 差分計算の高速化
CREATE INDEX idx_sync_status ON sync_status(sync_status, file_modified);
```

## 🚀 実装段階

### Phase 1: 基本的な可逆変換（1ヶ月）
- Box/メソッドレベルの変換
- コメントなし、インデント固定
- 単体テストで100%可逆性検証

### Phase 2: メタデータ保持（1ヶ月）
- コメントの位置と内容を保存
- インデントスタイルの保持
- 改行コードの維持

### Phase 3: 完全なスタイル保存（1ヶ月）
- 任意の空白パターン対応
- コーディングスタイルの自動検出
- チーム規約との調整機能

### Phase 4: 高度な同期（2ヶ月）
- 増分同期アルゴリズム
- コンフリクト解決UI
- パフォーマンス最適化

## 📊 利点の整理

### 開発者にとって
- **選択の自由**: ファイル編集もDB操作も可能
- **既存ツール互換**: VSCode、Vim、Git等すべて使える
- **高速リファクタリング**: 必要な時だけDB機能を活用

### システムにとって
- **Git完全互換**: 通常のテキストファイルとして管理
- **増分コンパイル**: DB側で依存関係を高速解析
- **AI連携強化**: 構造化データで学習効率UP

### チームにとって
- **移行リスクなし**: 段階的導入が可能
- **レビュー互換**: PRは従来通りのテキスト差分
- **柔軟な運用**: プロジェクト毎に最適な方式を選択

## 🎯 成功の指標

1. **完全可逆性**: 1000ファイルで往復変換してもバイト単位で一致
2. **パフォーマンス**: 1000行のファイルを100ms以内で変換
3. **互換性**: 既存のNyashプロジェクトがそのまま動作
4. **開発者満足度**: 90%以上が「便利」と評価

## 🔮 将来の拡張

### 意味的な可逆変換
- コードの意味を保ちながらスタイルを自動最適化
- チーム規約への自動適応
- リファクタリング履歴の保存

### マルチビュー編集
```nyash
// 同じコードを異なる視点で編集
- ファイルビュー: 従来のテキストエディタ
- 構造ビュー: Box/メソッドのツリー表示
- 依存ビュー: グラフィカルな関係表示
- クエリビュー: SQLで直接操作
```

### バージョン管理の革新
- 意味的な差分表示（「名前を変更」vs「全行変更」）
- 構造認識マージ（メソッド単位での自動解決）
- リファクタリング履歴の永続化

## 📝 実装優先順位

1. **コア変換エンジン**: 可逆性の証明
2. **メタデータ設計**: 完全な情報保存
3. **同期デーモン**: リアルタイム連携
4. **開発ツール**: CLI/IDE統合
5. **最適化**: パフォーマンスチューニング

---

この可逆変換システムにより、データベース駆動開発の利点を最大化しながら、既存の開発フローとの完全な互換性を実現できるにゃ！