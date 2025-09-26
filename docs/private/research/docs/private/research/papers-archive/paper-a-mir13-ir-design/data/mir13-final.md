# MIR13 (Core-13) Final Instruction Set

## The 13 Instructions

### 1. 値・計算 (3命令)
- **Const**: 定数値のロード
- **BinOp**: 二項演算（算術、論理、ビット演算すべて）
- **Compare**: 比較演算（==, !=, <, >, <=, >=）

### 2. 制御フロー (4命令)
- **Jump**: 無条件ジャンプ
- **Branch**: 条件分岐
- **Return**: 関数からの戻り
- **Phi**: SSA形式での値の合流

### 3. 呼び出し (3命令)
- **Call**: 通常の関数呼び出し
- **BoxCall**: Boxメソッド呼び出し（配列、オブジェクト、すべてのデータ操作）
- **ExternCall**: 外部関数呼び出し（システムコール、プラグイン等）

### 4. メタ操作 (3命令)
- **TypeOp**: 型関連操作（型チェック、キャスト）
- **Safepoint**: GCセーフポイント
- **Barrier**: メモリバリア

## 削除された命令とその統合先

| 削除された命令 | 統合方法 |
|--------------|---------|
| Load/Store | BoxCallまたはCall（変数もBoxとして扱う） |
| UnaryOp | BinOp（例：-x → 0-x, !x → x XOR true） |
| ArrayGet/ArraySet | BoxCall |
| NewBox | BoxCall（コンストラクタ呼び出し） |
| FunctionNew | Const（関数も値） |
| RefNew/RefGet/RefSet | BoxCall |
| TypeCheck/Cast | TypeOp |
| Debug/Print | ExternCall |
| Copy/Nop | 不要（最適化で除去） |

## 設計の革新性

### 1. 変数アクセスの統一
すべての変数アクセスが関数呼び出しとして表現される：
```mir
// 従来: %1 = Load %x
%1 = Call @get_local "x"

// 従来: Store %y, %1  
Call @set_local "y" %1
```

### 2. Everything is Box の究極形
- 変数もBox
- 関数もBox（Constで表現）
- すべての操作がBoxCall

### 3. 実用性とのバランス
- Safepointでガベージコレクションをサポート
- Barrierで並行性を考慮
- ExternCallで拡張性を確保