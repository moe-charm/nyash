# Phase 15: セルフホスティング技術詳細

## 1. アーキテクチャ設計

### 1.1 全体構成

```
NyashCompiler (Nyashで実装)
├── Frontend
│   ├── Lexer (トークナイザー)
│   ├── Parser (構文解析)
│   └── AST Builder
├── Middle-end
│   ├── Type Checker
│   ├── Name Resolver
│   ├── MIR Lowerer (→13命令)
│   └── Optimizer
└── Backend（複数選択可能）
    ├── CraneliftBox (JITラッパー)
    ├── X86EmitterBox (直接エミッタ)
    ├── TemplateStitcherBox (超小型)
    └── Runtime Linker
```

### 1.2 CompilerBox設計

```nyash
box CompilerBox {
    init { 
        lexer,      // トークン解析器
        parser,     // 構文解析器
        lowerer,    // MIR生成器
        optimizer,  // 最適化器
        backend     // コード生成器
    }
    
    // ソースコードからASTを生成
    parse(source) {
        local tokens = me.lexer.tokenize(source)
        local ast = me.parser.parse(tokens)
        return ast
    }
    
    // ASTからMIRを生成
    lower(ast) {
        local mir = me.lowerer.lower(ast)
        return me.optimizer.optimize(mir)
    }
    
    // MIRから実行可能コードを生成
    codegen(mir) {
        return me.backend.generate(mir)
    }
    
    // 完全なコンパイルパイプライン
    compile(source) {
        local ast = me.parse(source)
        local mir = me.lower(ast)
        return me.codegen(mir)
    }
}
```

## 2. パーサー実装（Nyash版）

### 2.1 Lexer実装例

```nyash
box Lexer {
    init { keywords, operators }
    
    constructor() {
        me.keywords = new MapBox()
        me.keywords.set("box", TokenType.BOX)
        me.keywords.set("if", TokenType.IF)
        me.keywords.set("loop", TokenType.LOOP)
        // ... 他のキーワード
        
        me.operators = new MapBox()
        me.operators.set("+", TokenType.PLUS)
        me.operators.set("-", TokenType.MINUS)
        // ... 他の演算子
    }
    
    tokenize(source) {
        local tokens = new ArrayBox()
        local position = 0
        
        loop(position < source.length()) {
            local char = source.charAt(position)
            
            if me.isWhitespace(char) {
                position = position + 1
                continue
            }
            
            if me.isDigit(char) {
                local token = me.readNumber(source, position)
                tokens.push(token)
                position = token.end
                continue
            }
            
            if me.isLetter(char) {
                local token = me.readIdentifier(source, position)
                tokens.push(token)
                position = token.end
                continue
            }
            
            // ... 他のトークン種別
        }
        
        return tokens
    }
}
```

### 2.2 Parser実装例

```nyash
box Parser {
    init { tokens, current }
    
    parse(tokens) {
        me.tokens = tokens
        me.current = 0
        return me.parseProgram()
    }
    
    parseProgram() {
        local statements = new ArrayBox()
        
        loop(not me.isAtEnd()) {
            local stmt = me.parseStatement()
            statements.push(stmt)
        }
        
        return new ASTNode("Program", statements)
    }
    
    parseStatement() {
        if me.match(TokenType.BOX) {
            return me.parseBoxDeclaration()
        }
        
        if me.match(TokenType.IF) {
            return me.parseIfStatement()
        }
        
        // ... 他の文種別
        
        return me.parseExpression()
    }
}
```

## 3. MIR生成器実装

### 3.1 Lowerer実装例

```nyash
box MIRLowerer {
    init { 
        current_block,
        value_counter,
        block_counter,
        locals
    }
    
    lower(ast) {
        me.value_counter = 0
        me.block_counter = 0
        me.locals = new MapBox()
        
        local mir = new MIRModule()
        me.lowerNode(ast, mir)
        return mir
    }
    
    lowerExpression(node, mir) {
        if node.type == "BinaryOp" {
            local left = me.lowerExpression(node.left, mir)
            local right = me.lowerExpression(node.right, mir)
            local result = me.newValue()
            
            mir.addInstruction(new BinOp(
                node.operator,
                left,
                right,
                result
            ))
            
            return result
        }
        
        if node.type == "Literal" {
            local result = me.newValue()
            mir.addInstruction(new Const(node.value, result))
            return result
        }
        
        // ... 他の式種別
    }
}
```

## 4. バックエンド実装

### 4.1 CraneliftBox実装

```nyash
box CraneliftBox {
    init { jit_module, func_ctx }
    
    constructor() {
        // CraneliftをFFI経由で初期化
        me.jit_module = ExternCall("cranelift_new_module")
        me.func_ctx = ExternCall("cranelift_new_context")
    }
    
    compile(mir) {
        local compiled_funcs = new MapBox()
        
        // 各関数をコンパイル
        for func in mir.functions {
            local code = me.compileFunction(func)
            compiled_funcs.set(func.name, code)
        }
        
        return compiled_funcs
    }
    
    compileFunction(mir_func) {
        // MIR → Cranelift IR変換
        ExternCall("cranelift_begin_function", me.func_ctx)
        
        for inst in mir_func.instructions {
            me.emitInstruction(inst)
        }
        
        // JITコンパイル
        return ExternCall("cranelift_finalize_function", me.func_ctx)
    }
}
```

### 4.2 X86EmitterBox実装（直接x86生成）

```nyash
box X86EmitterBox {
    init { code_buffer, label_map }
    
    constructor() {
        me.code_buffer = new ArrayBox()
        me.label_map = new MapBox()
    }
    
    compile(mir) {
        // MIR 13命令を直接x86-64に変換！
        for func in mir.functions {
            me.emitFunction(func)
        }
        
        return me.code_buffer
    }
    
    emitInstruction(inst) {
        // MIR命令をx86テンプレートに変換
        if inst.type == "Const" {
            // mov rax, imm64
            me.emit_mov_imm(inst.dst, inst.value)
        }
        
        if inst.type == "BinOp" {
            if inst.op == "Add" {
                // add rax, rbx
                me.emit_add(inst.dst, inst.left, inst.right)
            }
        }
        
        if inst.type == "BoxCall" {
            // mov rdi, receiver
            // mov rax, [rdi]     ; vtable
            // call [rax+slot*8]  ; method call
            me.emit_boxcall(inst.recv, inst.slot)
        }
        
        // ... 残り10命令のテンプレート
    }
    
    emit_mov_imm(reg, value) {
        // REX.W + mov r64, imm64
        me.code_buffer.push(0x48)  // REX.W
        me.code_buffer.push(0xB8 + reg)  // mov opcode
        
        // 64ビット即値をリトルエンディアンで
        for i in range(0, 8) {
            me.code_buffer.push((value >> (i * 8)) & 0xFF)
        }
    }
}
```

### 4.3 テンプレート・スティッチャ実装（超小型バイナリ）

```nyash
box TemplateStitcherBox {
    init { stub_addresses, jump_table }
    
    constructor() {
        // 各MIR命令の共通スタブアドレス
        me.stub_addresses = new MapBox()
        me.stub_addresses.set("Const", 0x1000)
        me.stub_addresses.set("UnaryOp", 0x1100)
        me.stub_addresses.set("BinOp", 0x1200)
        me.stub_addresses.set("Compare", 0x1300)
        me.stub_addresses.set("TypeOp", 0x1400)
        me.stub_addresses.set("Load", 0x1500)
        me.stub_addresses.set("Store", 0x1600)
        me.stub_addresses.set("Branch", 0x1700)
        me.stub_addresses.set("Jump", 0x1800)
        me.stub_addresses.set("Return", 0x1900)
        me.stub_addresses.set("Phi", 0x1A00)
        me.stub_addresses.set("BoxCall", 0x1B00)
        me.stub_addresses.set("ExternCall", 0x1C00)
    }
    
    compile(mir) {
        me.jump_table = new ArrayBox()
        
        // プログラムはスタブへのジャンプ列として表現
        for inst in mir.instructions {
            local stub_addr = me.stub_addresses.get(inst.type)
            
            // jmp rel32
            me.jump_table.push(0xE9)  // jmp opcode
            me.jump_table.push_rel32(stub_addr)
            
            // 命令固有のパラメータをデータセクションに配置
            me.encodeParameters(inst)
        }
        
        return me.jump_table
    }
}
```

## 5. ブートストラップ手順

### 5.1 段階的移行

1. **Stage 0**: Rustコンパイラで初期Nyashコンパイラをビルド
2. **Stage 1**: Stage 0コンパイラでNyashコンパイラ（Nyash版）をコンパイル
3. **Stage 2**: Stage 1コンパイラで自分自身をコンパイル
4. **検証**: Stage 1とStage 2の出力が同一であることを確認

### 5.2 検証スクリプト

```nyash
box BootstrapVerifier {
    verify() {
        // Stage 0でStage 1をビルド
        local stage0 = new CompilerBox()  // Rust版
        local stage1_code = stage0.compile(readFile("compiler.nyash"))
        
        // Stage 1でStage 2をビルド
        local stage1 = stage1_code.instantiate()
        local stage2_code = stage1.compile(readFile("compiler.nyash"))
        
        // バイナリ比較
        if stage1_code.equals(stage2_code) {
            print("🎉 Bootstrap successful!")
            return true
        } else {
            print("❌ Bootstrap failed - outputs differ")
            return false
        }
    }
}
```

## 6. 性能最適化

### 6.1 ホットパス最適化

```nyash
box OptimizingCompiler from CompilerBox {
    init { profiler }
    
    constructor() {
        from CompilerBox.constructor()
        me.profiler = new ProfilerBox()
    }
    
    compile(source) {
        // プロファイル収集モード
        if me.profiler.isEnabled() {
            me.profiler.start()
        }
        
        local result = from CompilerBox.compile(source)
        
        // ホット関数をJIT再コンパイル
        if me.profiler.hasHotFunctions() {
            for func in me.profiler.getHotFunctions() {
                me.recompileWithOptimization(func)
            }
        }
        
        return result
    }
}
```

## 7. エラー処理とデバッグ

### 7.1 エラーレポート

```nyash
box CompilerError {
    init { message, location, suggestions }
    
    format() {
        local output = "Error at " + me.location + ": " + me.message
        
        if me.suggestions.length() > 0 {
            output = output + "\nSuggestions:"
            for suggestion in me.suggestions {
                output = output + "\n  - " + suggestion
            }
        }
        
        return output
    }
}
```

## 8. テストフレームワーク

```nyash
box CompilerTest {
    testParser() {
        local parser = new Parser()
        local ast = parser.parse("box Test { }")
        
        assert(ast.type == "Program")
        assert(ast.children.length() == 1)
        assert(ast.children[0].type == "BoxDeclaration")
    }
    
    testMIRGeneration() {
        local compiler = new CompilerBox()
        local mir = compiler.lower(compiler.parse("1 + 2"))
        
        assert(mir.instructions.length() == 3)  // 2 Const + 1 BinOp
    }
    
    testEndToEnd() {
        local compiler = new CompilerBox()
        local code = compiler.compile("print('Hello')")
        local output = code.run()
        
        assert(output == "Hello")
    }
}
```

このようにして、NyashでNyashコンパイラを実装することで、真のセルフホスティングを実現します。