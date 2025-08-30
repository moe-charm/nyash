[Archived] 旧10.1系ドキュメントです。最新は ../INDEX.md を参照してください。

Note: 本来は「PythonをNyashで動かすフェーズ（Core実装）」の位置づけでしたが、現在は順番を変更し、先に 10.5b（MIR→VM→ネイティブビルド基盤）を進めています。

# Phase 10.1d - Core実装（Phase 1機能）

## 🎯 このフェーズの目的
Python AST → Nyash AST変換のPhase 1機能（基本構文）を実装する。

## 📁 実装ドキュメント
- **`python_implementation_roadmap.txt`** - Phase別実装ロードマップ

## 🔧 Phase 1必須要素（Codex先生強調）

### 意味論の必須実装
1. **LEGB + locals/freevars** - スコーピング規則
2. **デフォルト引数の評価タイミング** - 定義時に一度だけ
3. **イテレータベースのfor文** - `__iter__`/`__next__`プロトコル
4. **for/else + while/else** - Python独特のelse節
5. **Python真偽値判定** - `__bool__` → `__len__`
6. **短絡評価** - and/orの正確な挙動

### サポートする文（Statement）
- [x] def - 関数定義
- [x] if/elif/else - 条件分岐
- [x] for - ループ（else節対応必須）
- [x] while - ループ（else節対応必須）
- [x] break/continue - ループ制御
- [x] return - 戻り値

### サポートする式（Expression）
- [x] 算術演算子（+,-,*,/,//,%）
- [x] 比較演算子（==,!=,<,>,<=,>=,is,is not）
- [x] 論理演算子（and,or,not）- 短絡評価
- [x] 関数呼び出し
- [x] 変数参照/代入
- [x] リテラル（数値/文字列/bool）

## 🧪 テストケース
```python
# Phase 1で動作すべきコード
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

# for/else のテスト
for i in range(10):
    if i == 5:
        break
else:
    print("No break")  # 実行されない

# デフォルト引数の罠
def append_to_list(item, lst=[]):  # 定義時に評価！
    lst.append(item)
    return lst
```

## ✅ 完了条件
- [ ] 基本的な関数定義が変換できる
- [ ] 制御フローが正しく変換される
- [ ] 演算子が正しくマッピングされる
- [ ] Python意味論が保たれている
- [ ] 70%以上の関数がコンパイル可能

## 📊 テレメトリー確認
```bash
[PythonParser] Module: test.py (Python 3.11)
  Functions: 10 total
  Compiled: 7 (70%) ← 目標達成！
  Fallback: 3 (30%)
```

## ⏭️ 次のフェーズ
→ Phase 10.1e (トランスパイラー)
