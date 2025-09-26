# Python Descriptor Protocol → Nyash Property System マッピング

## 🎯 概要

2025-09-18のProperty System革命により、Python transpilationが飛躍的に向上。Pythonのdescriptor protocolを完全にNyashのProperty System（stored/computed/once/birth_once）にマッピングする設計。

## 🧬 Pythonプロパティ分類とNyashマッピング

### 1. 通常フィールド → stored
```python
class PyClass:
    def __init__(self):
        self.name = "example"  # 通常の属性
```
```nyash
box PyClass {
    name: StringBox  // stored: 通常のフィールドストレージ
    
    birth() {
        me.name = "example"
    }
}
```

### 2. @property → computed  
```python
class PyClass:
    @property
    def full_name(self):
        return f"{self.first} {self.last}"
```
```nyash
box PyClass {
    first: StringBox
    last: StringBox
    full_name: StringBox { me.first + " " + me.last }  // computed: 毎回計算
}
```

### 3. @functools.cached_property → once
```python
import functools

class PyClass:
    @functools.cached_property  
    def expensive_data(self):
        return expensive_computation()
```
```nyash
box PyClass {
    once expensive_data: DataBox { expensive_computation() }  // once: 遅延評価＋キャッシュ
}
```

### 4. カスタムdescriptor → 判定ロジック
```python
class CustomDescriptor:
    def __get__(self, obj, objtype=None):
        # カスタムロジック
        pass
    
    def __set__(self, obj, value):
        # セットロジック  
        pass

class PyClass:
    custom_prop = CustomDescriptor()
```

**判定ロジック（PythonCompilerBox内）**:
```nyash
box DescriptorAnalyzer {
    analyze_descriptor(descriptor_ast) {
        if descriptor_ast.has_get_only() {
            if descriptor_ast.is_pure_computation() {
                return "computed"  // 副作用なし計算
            } else {
                return "once"      // 副作用あり＝キャッシュ推奨
            }
        }
        
        if descriptor_ast.has_get_and_set() {
            return "stored"        // getterとsetterあり＝通常フィールド
        }
        
        return "unsupported"       // 複雑すぎ→Phase 2以降
    }
}
```

## 🎯 自動判定アルゴリズム

### Phase 1: 基本パターン認識
```python
def classify_python_property(ast_node):
    # 1. デコレータ解析
    if has_decorator(ast_node, "@property"):
        if is_simple_computation(ast_node.body):
            return "computed"
        else:
            return "once"  # 複雑→キャッシュ推奨
    
    # 2. cached_property検出
    if has_decorator(ast_node, "@functools.cached_property"):
        return "once"
    
    # 3. 通常の__init__内代入
    if is_init_assignment(ast_node):
        return "stored"
    
    # 4. descriptor検出（Phase 2以降）
    if has_custom_descriptor(ast_node):
        return analyze_descriptor_complexity(ast_node)
```

### Phase 2: 高度な判定（将来）
- **副作用解析**: I/O、外部状態変更の検出
- **コスト解析**: 計算量推定による once vs computed 判定
- **依存解析**: 他のプロパティとの依存関係

## 🌟 poison-on-throw統合

### Python例外 → Nyash例外処理
```python
class PyClass:
    @functools.cached_property
    def risky_operation(self):
        if random.random() < 0.1:
            raise ValueError("Failed!")
        return expensive_result()
```

```nyash
box PyClass {
    once risky_operation: ResultBox { 
        if random_float() < 0.1 {
            throw ValueError("Failed!")
        }
        return expensive_result()
    } catch(ex) {
        poison(me.risky_operation, ex)  // poison-on-throw適用
        throw ex
    }
}
```

## 📊 実装フェーズ

### Phase 10.7a: 基本認識（1週間）
- @property, @cached_property, 通常フィールドの自動分類
- 単純なcomputedプロパティ生成

### Phase 10.7b: 拡張判定（2週間）  
- カスタムdescriptor解析
- 副作用検出ロジック
- poison-on-throw統合

### Phase 10.7c: 最適化（1週間）
- 依存解析による once vs computed 最適選択
- LLVM最適化との連携

## 🧪 テストケース

### 成功パターン
```python
# シンプルcomputed
@property
def area(self): return self.width * self.height  # → computed

# キャッシュ必要
@functools.cached_property  
def heavy_calc(self): return sum(range(1000000))  # → once

# 通常フィールド
def __init__(self): self.name = "test"  # → stored
```

### 限界ケース（Phase 2以降）
```python
# 複雑なdescriptor（未対応）
class ComplexDescriptor:
    def __get__(self, obj, objtype=None):
        # 複雑な条件分岐、外部API呼び出し等
        pass
```

## 🚀 期待される効果

1. **開発体験**: PythonプロパティがNyashで自然に表現
2. **性能向上**: LLVMによるproperty最適化（10-50x高速化）
3. **型安全性**: NyashのBox型システムによる実行時安全性
4. **保守性**: 生成されたNyashコードの可読性・編集可能性

このマッピング設計により、PythonからNyashへの自然で高性能なtranspilationが実現できます！