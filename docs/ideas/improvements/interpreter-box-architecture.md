# InterpreterBox アーキテクチャ - インタープリター層の箱化

## 概要
インタープリター層を丸ごと箱化して疎結合にすることで、将来的な移行・撤退を容易にする設計提案。

## 背景
- ChatGPT5さんの指摘：インタープリター層は将来的に撤退可能
- 現状：AST実行とMIR実行が並存している
- VM層がMIRを直接実行するため、インタープリター層は冗長

## 提案：Everything is Box哲学の適用

### 現在の密結合
```rust
// main.rsで直接呼び出し
match backend {
    Backend::Interpreter => interpreter::execute(ast),  // 密結合
    Backend::VM => vm::execute(mir),
}
```

### 箱化による疎結合
```rust
// 実行エンジンを箱として抽象化
pub trait ExecutorBox: Send + Sync {
    fn execute(&self, input: ExecutionInput) -> Result<Value>;
}

// インタープリター丸ごと箱化
pub struct InterpreterBox {
    ast_executor: AstExecutor,
    symbol_table: SymbolTable,
}

impl ExecutorBox for InterpreterBox {
    fn execute(&self, input: ExecutionInput) -> Result<Value> {
        self.ast_executor.run(input.ast)
    }
}

// VM丸ごと箱化  
pub struct VMBox {
    mir_executor: MirExecutor,
    runtime: Runtime,
}

impl ExecutorBox for VMBox {
    fn execute(&self, input: ExecutionInput) -> Result<Value> {
        let mir = compile_to_mir(input.ast);
        self.mir_executor.run(mir)
    }
}
```

### 使用例
```rust
let executor: Box<dyn ExecutorBox> = match backend {
    Backend::Interpreter => Box::new(InterpreterBox::new()),
    Backend::VM => Box::new(VMBox::new()),
};
executor.execute(program)
```

## メリット

1. **撤退不要**：使わなくなっても箱ごと置いておける
2. **切り替え簡単**：実行時に箱を差し替えるだけ
3. **テスト容易**：両方の箱で実行して結果を比較可能
4. **将来性**：プラグイン化も可能

## Nyash的な書き方
```nyash
// 将来的にはこんな感じ？
box InterpreterBox {
    init { ast_executor, symbol_table }
    
    execute(ast) {
        return me.ast_executor.run(ast)
    }
}

box VMBox {
    init { mir_executor, runtime }
    
    execute(ast) {
        local mir = compile_to_mir(ast)
        return me.mir_executor.run(mir)
    }
}

// 実行エンジンの切り替え
local executor = new VMBox()  // or new InterpreterBox()
executor.execute(program)
```

## まとめ
「捨てる」のではなく「箱に入れる」ことで、Nyashの"Everything is Box"哲学を貫きながら、将来の変更に対して柔軟に対応できる設計。

---
作成日: 2025-09-02
カテゴリ: アーキテクチャ改善
優先度: 中（将来的な改善案）