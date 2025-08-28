# Related Work

## 1. 所有権・ライフサイクル管理システム

### 1.1 Rust
**アプローチ**: 型システムによる静的所有権管理
```rust
fn example() {
    let s = String::from("hello");  // sが所有
    let t = s;                      // 所有権移動
    // println!("{}", s);           // コンパイルエラー
}
```

**Nyashとの比較**:
- Rust: コンパイル時に所有権を検証
- Nyash: 実行時に統一契約で管理
- **利点**: Nyashはプラグインも同じ契約で管理可能

### 1.2 Swift (ARC)
**アプローチ**: 自動参照カウント
```swift
class Person {
    let name: String
    init(name: String) { self.name = name }
    deinit { print("Person is being deinitialized") }
}
```

**Nyashとの比較**:
- Swift: ネイティブオブジェクトのみARC
- Nyash: プラグインBoxも含めて統一管理
- **利点**: FFI境界でも一貫したライフサイクル

### 1.3 C++ (RAII)
**アプローチ**: スコープベースのリソース管理
```cpp
{
    std::unique_ptr<File> f = std::make_unique<File>("data.txt");
    // スコープ終了時に自動的にデストラクタ
}
```

**Nyashとの比較**:
- C++: テンプレートとデストラクタ
- Nyash: @must_drop/@gcableアノテーション
- **利点**: GCオン/オフ切り替え可能

## 2. FFIシステム

### 2.1 JNI (Java Native Interface)
**アプローチ**: 複雑なAPI経由でのネイティブ呼び出し
```java
public native void nativeMethod();
```

```c
JNIEXPORT void JNICALL Java_ClassName_nativeMethod(JNIEnv *env, jobject obj) {
    // 複雑なJNI API使用
}
```

**問題点**:
- グローバル参照の管理が複雑
- 例外処理の境界が不明確
- パフォーマンスオーバーヘッド大

**Nyashの解決**:
- ハンドルベース（instance_id）で単純化
- 例外は戻り値で表現
- 静的リンクで高速化

### 2.2 Python ctypes
**アプローチ**: 動的型付けでのFFI
```python
from ctypes import *
lib = CDLL('./library.so')
lib.function.argtypes = [c_int, c_char_p]
result = lib.function(42, b"hello")
```

**Nyashとの比較**:
- Python: 実行時の型チェック
- Nyash: TLVで型情報を含む
- **利点**: より安全で予測可能

### 2.3 WASM Component Model
**アプローチ**: インターフェース定義言語
```wit
interface calculator {
    add: func(a: s32, b: s32) -> s32
}
```

**Nyashとの比較**:
- WASM: モジュール間の境界定義
- Nyash: 言語レベルでの統一契約
- **利点**: より深いライフサイクル統合

## 3. マルチバックエンド実行システム

### 3.1 GraalVM
**アプローチ**: ポリグロット実行環境
- 複数言語を同一VMで実行
- Truffle フレームワークでAST解釈

**Nyashとの比較**:
- GraalVM: 言語間の相互運用重視
- Nyash: 単一言語の実行戦略多様性
- **利点**: より軽量で予測可能

### 3.2 PyPy
**アプローチ**: トレーシングJIT
- Pythonコードを高速化
- RPythonで実装

**Nyashとの比較**:
- PyPy: 既存言語の高速化
- Nyash: 最初から多バックエンド設計
- **利点**: クリーンな設計で保守性高

### 3.3 .NET/CLR
**アプローチ**: 共通中間言語（CIL）
```csharp
// C#コード → CIL → JIT/AOT
public class Example {
    public static void Main() { }
}
```

**Nyashとの比較**:
- .NET: 巨大なランタイム
- Nyash: 最小限のランタイム（~4K LoC）
- **利点**: 理解・検証が容易

## 4. プラグインシステム

### 4.1 COM (Component Object Model)
**アプローチ**: インターフェースベース
```cpp
interface IUnknown {
    virtual HRESULT QueryInterface(REFIID riid, void **ppv) = 0;
    virtual ULONG AddRef() = 0;
    virtual ULONG Release() = 0;
};
```

**問題点**:
- 複雑なレジストリ管理
- Windowsプラットフォーム依存
- バージョニングの悪夢

### 4.2 OSGi (Java)
**アプローチ**: 動的モジュールシステム
- バンドルのライフサイクル管理
- サービスレジストリ

**Nyashとの比較**:
- OSGi: Javaエコシステム専用
- Nyash: 言語中立的なC ABI
- **利点**: より単純で移植性高

## 5. 学術的位置づけ

### 5.1 新規性
1. **統一ライフサイクル**: ユーザーBox＝プラグインBox
2. **GC切り替え可能**: 意味論を保持
3. **実行戦略の透明性**: 5つのバックエンドで同一動作

### 5.2 関連する理論
- **線形型システム**: Rustの所有権との比較
- **エフェクトシステム**: @must_drop/@gcableとの関連
- **抽象機械**: MIR設計の理論的基礎

### 5.3 応用可能性
- **組み込みシステム**: 決定的メモリ管理
- **ゲームエンジン**: 高性能プラグイン
- **科学計算**: 予測可能な性能