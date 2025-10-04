# PythonCompilerBox Property System活用実装

## 🎯 概要

Property System革命（stored/computed/once/birth_once）をPythonCompilerBoxで活用し、Python→Nyash transpilationを実現する技術実装設計。

## 🏗️ アーキテクチャ設計

### コアコンポーネント
```nyash
box PythonCompilerBox {
    // Property System を活用したコンパイラ実装
    classifier: PropertyClassifierBox
    generator: NyashCodeGeneratorBox
    validator: SemanticValidatorBox
    
    // コンパイル結果のキャッシュ（once使用）
    once compilation_cache: MapBox { new MapBox() }
    
    // 動的に計算される統計情報（computed使用）
    success_rate: FloatBox { me.get_success_statistics() }
    total_files_processed: IntegerBox { me.compilation_cache.size() }
    
    birth() {
        me.classifier = new PropertyClassifierBox()
        me.generator = new NyashCodeGeneratorBox()
        me.validator = new SemanticValidatorBox()
    }
}
```

## 🧠 PropertyClassifierBox実装

```nyash
box PropertyClassifierBox {
    // 分類ルールのキャッシュ（once）
    once classification_rules: RuleSetBox { load_classification_rules() }
    
    // 統計情報（computed）
    classified_count: IntegerBox { me.get_classification_stats().count }
    accuracy_rate: FloatBox { me.get_classification_stats().accuracy }
    
    classify_python_property(ast_node) {
        // Python AST → Property type 分類
        if me.has_decorator(ast_node, "@property") {
            if me.is_simple_computation(ast_node) {
                return "computed"  // 単純計算 → computed
            } else {
                return "once"      // 複雑処理 → キャッシュ推奨
            }
        }
        
        if me.has_decorator(ast_node, "@functools.cached_property") {
            return "once"          // 明示的キャッシュ
        }
        
        if me.is_init_assignment(ast_node) {
            return "stored"        // 通常フィールド
        }
        
        if me.is_birth_once_candidate(ast_node) {
            return "birth_once"    // 初期化時のみ評価
        }
        
        return "unsupported"       // Phase 2以降
    }
    
    // ヒューリスティック判定
    is_simple_computation(node) {
        // 副作用なし＋計算量小 → computed適合性判定
        return me.has_no_side_effects(node) and me.is_lightweight(node)
    }
    
    is_birth_once_candidate(node) {
        // 初期化時のみ必要な重い処理を検出
        return me.called_in_init_only(node) and me.is_expensive(node)
    }
}
```

## 🏭 NyashCodeGeneratorBox実装

```nyash
box NyashCodeGeneratorBox {
    // テンプレートエンジン（once）
    once property_templates: TemplateEngineBox { 
        load_property_templates() 
    }
    
    // 生成コード統計（computed）
    generated_lines: IntegerBox { me.count_generated_code_lines() }
    compression_ratio: FloatBox { me.calculate_compression_ratio() }
    
    generate_property_declaration(property_info) {
        local template = me.property_templates.get(property_info.type)
        
        // Property typeごとの生成
        peek property_info.type {
            "stored" => me.generate_stored_property(property_info),
            "computed" => me.generate_computed_property(property_info),
            "once" => me.generate_once_property(property_info),
            "birth_once" => me.generate_birth_once_property(property_info),
            else => throw UnsupportedPropertyError(property_info.type)
        }
    }
    
    generate_computed_property(info) {
        // computed property テンプレート
        return info.name + ": " + info.type + " { " + info.expression + " }"
    }
    
    generate_once_property(info) {
        // once property テンプレート（キャッシュ＋例外安全）
        local code = "once " + info.name + ": " + info.type + " { " + info.expression + " }"
        
        if info.has_exception_risk {
            code = code + " catch(ex) { poison(me." + info.name + ", ex); throw ex }"
        }
        
        return code
    }
    
    generate_birth_once_property(info) {
        // birth_once property テンプレート
        return "birth_once " + info.name + ": " + info.type + " { " + info.expression + " }"
    }
}
```

## 🔍 SemanticValidatorBox実装

```nyash
box SemanticValidatorBox {
    // 検証ルール（once）
    once validation_rules: ValidationRuleSetBox { 
        load_semantic_validation_rules() 
    }
    
    // 検証結果統計（computed）
    validation_success_rate: FloatBox { me.get_validation_stats().success_rate }
    error_categories: ArrayBox { me.get_validation_stats().error_types }
    
    validate_property_semantics(python_ast, nyash_code) {
        local errors = new ArrayBox()
        
        // 1. Property type一致性検証
        me.validate_property_type_consistency(python_ast, nyash_code, errors)
        
        // 2. 例外安全性検証  
        me.validate_exception_safety(python_ast, nyash_code, errors)
        
        // 3. 性能特性検証
        me.validate_performance_characteristics(python_ast, nyash_code, errors)
        
        return ValidationResult.new(errors)
    }
    
    validate_property_type_consistency(python_ast, nyash_code, errors) {
        // Pythonの@propertyとNyashのcomputedが対応しているかチェック
        local python_properties = me.extract_python_properties(python_ast)
        local nyash_properties = me.extract_nyash_properties(nyash_code)
        
        loop(python_properties.iter()) property {
            local expected_type = me.infer_nyash_property_type(property)
            local actual_type = nyash_properties.get(property.name).type
            
            if expected_type != actual_type {
                errors.add(PropertyTypeMismatchError.new(property.name, expected_type, actual_type))
            }
        }
    }
}
```

## 🎯 統合ワークフロー

```nyash
box PythonTranspilationWorkflow {
    compiler: PythonCompilerBox
    
    birth() {
        me.compiler = new PythonCompilerBox()
    }
    
    transpile_python_file(file_path) {
        // 1. Pythonファイル解析
        local python_ast = me.parse_python_file(file_path)
        
        // 2. Property分類
        local classified_properties = me.compiler.classifier.classify_all_properties(python_ast)
        
        // 3. Nyashコード生成
        local nyash_code = me.compiler.generator.generate_nyash_code(classified_properties)
        
        // 4. セマンティック検証
        local validation_result = me.compiler.validator.validate_property_semantics(python_ast, nyash_code)
        
        if validation_result.has_errors() {
            throw TranspilationError.new(validation_result.errors)
        }
        
        // 5. コンパイル結果キャッシュ（once活用）
        me.compiler.compilation_cache.set(file_path, nyash_code)
        
        return nyash_code
    }
}
```

## 🧪 テスト実装例

```nyash
box PropertySystemTranspilationTest {
    test_computed_property_generation() {
        local python_code = '''
        class TestClass:
            @property
            def doubled_value(self):
                return self.value * 2
        '''
        
        local compiler = new PythonCompilerBox()
        local result = compiler.transpile(python_code)
        
        assert result.contains("doubled_value: IntegerBox { me.value * 2 }")
    }
    
    test_once_property_generation() {
        local python_code = '''
        class TestClass:
            @functools.cached_property
            def expensive_calc(self):
                return heavy_computation()
        '''
        
        local compiler = new PythonCompilerBox()
        local result = compiler.transpile(python_code)
        
        assert result.contains("once expensive_calc: ResultBox { heavy_computation() }")
    }
    
    test_poison_on_throw_integration() {
        local python_code = '''
        class TestClass:
            @functools.cached_property
            def risky_operation(self):
                if random.random() < 0.1:
                    raise ValueError("Failed")
                return success_result()
        '''
        
        local compiler = new PythonCompilerBox()
        local result = compiler.transpile(python_code)
        
        assert result.contains("catch(ex) { poison(me.risky_operation, ex); throw ex }")
    }
}
```

## 📊 期待される効果

### 1. 実装効率の向上
- Property System活用により、コンパイラ自体の実装がクリーン
- once活用でコンパイルキャッシュ、computed活用で統計計算

### 2. 生成コードの高品質化  
- Python property → Nyash property の自然な1:1マッピング
- poison-on-throw統合による例外安全性

### 3. 保守性の向上
- Box化されたコンポーネント設計
- 明確な責任分離（分類・生成・検証）

この設計により、Property System革命を最大限活用したPython transpilation実装が実現できます！