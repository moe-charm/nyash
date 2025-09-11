# Phase 21: Nyash自己解析アプローチ - AST直接保存

## 📋 概要

Nyashの最大の強み「自分自身をパースできる」を活かした究極にシンプルなアプローチ。
外部パーサー不要、複雑な変換層不要。NyashのASTをそのままデータベースに保存する。

## 🎯 核心的なアイデア

```nyash
// Nyashは自分自身を解析できる！
NyashParser.parse(sourceCode) → AST → Database → NyashPrinter.print(AST) → sourceCode
```

**重要な気づき：**
- Nyashにはすでにパーサーがある
- ASTから元のソースを生成する機能もある
- だから、ASTをDBに保存すれば完全可逆！

## 🏗️ シンプルな実装

### データベース構造
```sql
-- ASTノードをそのまま保存
CREATE TABLE ast_nodes (
    id INTEGER PRIMARY KEY,
    node_type TEXT,        -- "Box", "Method", "Field", "Statement"等
    node_data JSON,        -- ASTノードの完全な情報
    parent_id INTEGER,
    position INTEGER,      -- 親ノード内での位置
    source_file TEXT,      -- 元のファイルパス
    metadata JSON,         -- 後から追加する解析情報
    FOREIGN KEY (parent_id) REFERENCES ast_nodes(id)
);

-- インデックス
CREATE INDEX idx_node_type ON ast_nodes(node_type);
CREATE INDEX idx_parent ON ast_nodes(parent_id);
CREATE INDEX idx_source ON ast_nodes(source_file);
```

### 基本実装
```nyash
box NyashCodeDB {
    parser: NyashParser
    printer: NyashPrinter
    db: SQLiteBox
    
    birth() {
        me.parser = new NyashParser()
        me.printer = new NyashPrinter()
        me.db = new SQLiteBox("code.db")
    }
    
    // ファイルをDBに保存
    importFile(filePath) {
        local source = FileBox.read(filePath)
        local ast = me.parser.parse(source)
        
        // ASTを再帰的にDBに保存
        me.saveAST(ast, null, filePath)
    }
    
    // ASTノードを保存
    saveAST(node, parentId, sourceFile) {
        local nodeId = me.db.insert("ast_nodes", {
            node_type: node.type,
            node_data: node.toJSON(),
            parent_id: parentId,
            position: node.position,
            source_file: sourceFile
        })
        
        // 子ノードも保存
        for (i, child) in node.children.enumerate() {
            child.position = i
            me.saveAST(child, nodeId, sourceFile)
        }
        
        return nodeId
    }
    
    // DBからソースコードを復元
    exportFile(filePath) {
        local rootNodes = me.db.query(
            "SELECT * FROM ast_nodes 
             WHERE source_file = ? AND parent_id IS NULL 
             ORDER BY position",
            filePath
        )
        
        local source = ""
        for node in rootNodes {
            local ast = me.loadAST(node.id)
            source += me.printer.print(ast) + "\n"
        }
        
        FileBox.write(filePath, source)
    }
    
    // ASTを再構築
    loadAST(nodeId) {
        local node = me.db.get("ast_nodes", nodeId)
        local astNode = ASTNode.fromJSON(node.node_data)
        
        // 子ノードも読み込む
        local children = me.db.query(
            "SELECT * FROM ast_nodes 
             WHERE parent_id = ? 
             ORDER BY position",
            nodeId
        )
        
        for child in children {
            astNode.addChild(me.loadAST(child.id))
        }
        
        return astNode
    }
}
```

## 🚀 高度な機能

### リファクタリング
```nyash
box ASTRefactorer {
    db: SQLiteBox
    
    // 名前変更
    renameBox(oldName, newName) {
        // Box定義を見つける
        local boxNodes = me.db.query(
            "SELECT * FROM ast_nodes 
             WHERE node_type = 'Box' 
               AND json_extract(node_data, '$.name') = ?",
            oldName
        )
        
        for node in boxNodes {
            // AST上で名前を変更
            local data = JSON.parse(node.node_data)
            data.name = newName
            me.db.update("ast_nodes", node.id, {
                node_data: JSON.stringify(data)
            })
        }
        
        // 使用箇所も更新
        me.updateReferences(oldName, newName)
    }
    
    // メソッド移動
    moveMethod(methodName, fromBox, toBox) {
        // SQLで親ノードを変更するだけ！
        local fromBoxId = me.findBoxNode(fromBox)
        local toBoxId = me.findBoxNode(toBox)
        
        me.db.execute(
            "UPDATE ast_nodes 
             SET parent_id = ? 
             WHERE parent_id = ? 
               AND node_type = 'Method'
               AND json_extract(node_data, '$.name') = ?",
            [toBoxId, fromBoxId, methodName]
        )
    }
}
```

### メタデータ解析（オンデマンド）
```nyash
box MetadataEngine {
    // 必要な時だけ解析
    analyzeOnDemand(nodeId) {
        local node = db.get("ast_nodes", nodeId)
        
        if not node.metadata or me.isOutdated(node.metadata) {
            local metadata = {
                dependencies: me.findDependencies(node),
                complexity: me.calculateComplexity(node),
                lastAnalyzed: now()
            }
            
            db.update("ast_nodes", nodeId, {
                metadata: JSON.stringify(metadata)
            })
        }
        
        return JSON.parse(node.metadata)
    }
    
    // 依存関係を動的に検出
    findDependencies(node) {
        local deps = []
        
        // "new XXXBox" パターンを検索
        local matches = me.searchPattern(node, "NewBox")
        for match in matches {
            deps.push(match.boxType)
        }
        
        // "from XXX" パターンを検索
        local inherits = me.searchPattern(node, "From")
        for inherit in inherits {
            deps.push(inherit.parentBox)
        }
        
        return deps
    }
}
```

## 📊 利点

### 1. 実装の簡単さ
- パーサーはすでにある（Nyash自身）
- プリンターもすでにある
- 複雑な変換層不要

### 2. 100%の正確性
- Nyash公式パーサーを使うから完璧
- ASTは言語の完全な表現
- 情報の欠落なし

### 3. 柔軟性
- メタデータは後から追加
- 部分的な解析が可能
- 増分更新が簡単

### 4. 高速性
- ASTの一部だけ読み込み可能
- SQLの力でクエリが高速
- キャッシュも自然に実装

## 🎯 実装ステップ

### Phase 1: 基本機能（1週間）
- AST保存・読み込み
- ファイル単位のインポート・エクスポート
- 基本的なクエリ

### Phase 2: リファクタリング（1週間）
- 名前変更
- メソッド移動
- 依存関係追跡

### Phase 3: 高度な機能（2週間）
- メタデータ解析
- インクリメンタル更新
- VSCode統合

## 🌟 まとめ

**「Nyashの能力をフル活用する」**

- 外部ツール不要
- 複雑な実装不要
- Nyashらしいシンプルさ

このアプローチなら、Phase 21は「NyashのASTをDBに入れるだけ」という
極めてシンプルな実装で、強力な機能を実現できる！

---

> 「なぜ複雑にする？Nyashにはすでに必要なものが全部ある」 - にゃ