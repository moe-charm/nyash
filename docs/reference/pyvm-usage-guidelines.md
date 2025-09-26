# PyVM使用ガイドライン

**PyVM重要インフラ特化保持戦略**（2025-09-24確定）

## ⚠️ **重要: PyVMの役割限定**

PyVMは**一般的なプログラム実行には使用しないでください**。Phase 15戦略により、PyVMは以下の重要インフラ機能に特化して保持されています。

### ✅ **PyVM適切な用途**

#### 1. JSON v0ブリッジ機能
```bash
# セルフホスティング実行（PyVM自動使用）
NYASH_SELFHOST_EXEC=1 ./target/release/nyash program.nyash
```
- **用途**: Rust→Python連携でMIR JSON生成
- **重要性**: Phase 15.3コンパイラMVP開発に必須
- **自動処理**: ユーザーが直接PyVMを意識する必要なし

#### 2. using処理共通パイプライン
```bash
# using前処理（PyVM内部使用）
./target/release/nyash --enable-using program_with_using.nyash
```
- **用途**: `strip_using_and_register`統一処理
- **重要性**: Rust VM・LLVMとの共通前処理基盤
- **内部処理**: PyVMは前処理段階でのみ使用

#### 3. サンドボックス実行環境
```bash
# 開発者の明示的使用（上級者のみ）
NYASH_VM_USE_PY=1 ./target/release/nyash program.nyash
```
- **用途**: 安全なコード実行制御、実験的検証
- **対象**: 開発者・研究者の明示的使用のみ

### ❌ **PyVM不適切な用途**

#### 1. 一般的なプログラム実行
```bash
# ❌ 使わないでください
NYASH_VM_USE_PY=1 ./target/release/nyash my_application.nyash

# ✅ 代わりにこれを使用
./target/release/nyash my_application.nyash                 # Rust VM
./target/release/nyash --backend llvm my_application.nyash  # LLVM
```

#### 2. 性能比較・ベンチマーク
```bash
# ❌ 意味のない比較
time NYASH_VM_USE_PY=1 ./target/release/nyash program.nyash

# ✅ 意味のある比較
time ./target/release/nyash program.nyash                   # Rust VM
time ./target/release/nyash --backend llvm program.nyash    # LLVM
```

#### 3. 新機能開発・テスト
```bash
# ❌ PyVMでの新機能テスト
NYASH_VM_USE_PY=1 ./target/release/nyash new_feature.nyash

# ✅ 2本柱での新機能テスト
./target/release/nyash new_feature.nyash                    # Rust VM開発
./target/release/nyash --backend llvm new_feature.nyash     # LLVM検証
```

## 🎯 **Phase 15推奨実行方法**

### **開発・デバッグ・一般用途**
```bash
# 基本実行（最も推奨）
./target/release/nyash program.nyash

# 詳細診断
NYASH_CLI_VERBOSE=1 ./target/release/nyash program.nyash

# プラグインエラー対策
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash program.nyash
```

### **本番・最適化・配布用途**
```bash
# LLVM最適化実行
./target/release/nyash --backend llvm program.nyash

# LLVM詳細診断
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend llvm program.nyash
```

### **セルフホスティング開発用途**
```bash
# JSON v0ブリッジ（PyVM自動使用）
NYASH_SELFHOST_EXEC=1 ./target/release/nyash program.nyash

# using処理テスト
./target/release/nyash --enable-using program_with_using.nyash
```

## 📊 **技術的根拠**

### **PyVM保持理由**
1. **JSON v0ブリッジ**: Rust→Python連携で不可欠
2. **using前処理**: 共通パイプライン基盤として重要
3. **セルフホスト開発**: Phase 15.3コンパイラMVP実装に必須

### **PyVM除外理由**
1. **性能劣化**: Rust VM (712行) > PyVM (1074行)
2. **保守負荷**: Python実装の複雑性
3. **開発分散**: 2本柱集中による効率化

### **切り離しリスク評価**
- ❌ **即座削除**: Phase 15.3開発停止
- ❌ **段階削除**: JSON v0ブリッジ断絶
- ✅ **特化保持**: 重要インフラとして最小維持

## 🚨 **よくある誤用パターン**

### **誤用例1: 性能測定でのPyVM使用**
```bash
# ❌ 間違い
echo "Performance test:"
time NYASH_VM_USE_PY=1 ./target/release/nyash benchmark.nyash

# ✅ 正しい
echo "Performance test:"
time ./target/release/nyash benchmark.nyash                # Rust VM
time ./target/release/nyash --backend llvm benchmark.nyash # LLVM
```

### **誤用例2: デフォルトとしてのPyVM使用**
```bash
# ❌ 間違い
export NYASH_VM_USE_PY=1  # グローバル設定として使用

# ✅ 正しい
# デフォルトはRust VM、特殊用途のみ個別指定
NYASH_SELFHOST_EXEC=1 ./target/release/nyash selfhost_script.nyash
```

### **誤用例3: 学習・練習でのPyVM使用**
```bash
# ❌ 間違い（学習者向け）
NYASH_VM_USE_PY=1 ./target/release/nyash hello_world.nyash

# ✅ 正しい（学習者向け）
./target/release/nyash hello_world.nyash  # シンプルで高品質なRust VM
```

## 💡 **まとめ**

**PyVMは「重要インフラ」として保持し、一般使用は避ける**

- **役割**: JSON v0ブリッジ・using処理・セルフホスト開発
- **非推奨**: 一般実行・性能測定・新機能開発
- **推奨**: Rust VM（開発）+ LLVM（本番）の2本柱

この方針により、Phase 15の開発効率を最大化し、重要機能を安全に保持できます。