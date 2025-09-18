# LLVM Native EXE Generation Strategy

## 📋 概要

LLVM バックエンドを使用した完全なネイティブ実行ファイル生成パイプラインの実装戦略。
Rustのビルド時間を削減するため、LLVM コンパイラ部分を独立した crate として分離する。

## 🎯 目標

### 短期目標（Phase 15.5）
1. **LLVM オブジェクト生成の安定化**（ChatGPT5実装中）
2. **リンカー統合**によるEXE生成
3. **基本的なビルド・実行パイプライン**の確立

### 中期目標（Phase 15.6）
1. **`nyash-llvm-compiler` crate の分離**
2. **ビルド時間の大幅短縮**（5分→2分）
3. **CI/CD での並列ビルド**対応

## 🏗️ アーキテクチャ

### 現在の構成（モノリシック）
```
nyash-rust/
├── Cargo.toml (features = ["llvm"])  # 重い！
├── src/
│   ├── backend/
│   │   └── llvm/          # LLVM実装
│   └── main.rs
```

### 目標構成（分離型）
```
nyash-rust/                    # メインクレート（軽量）
├── Cargo.toml                 # LLVM機能なし
├── src/

nyash-llvm-compiler/          # 独立コンパイラ
├── Cargo.toml                # LLVM依存のみ
├── src/
│   ├── main.rs              # CLI エントリポイント
│   ├── mir_reader.rs        # MIR入力処理
│   ├── codegen/             # LLVM コード生成
│   └── linker.rs            # リンカー統合
```

## 🔧 実装計画

### Phase 1: 現在のLLVMバックエンド完成
```rust
// src/backend/llvm/compiler.rs
impl LLVMCompiler {
    pub fn compile_to_executable(
        &self,
        mir: &MirModule,
        output: &Path,
        link_options: &LinkOptions,
    ) -> Result<(), Error> {
        // 1. MIR → LLVM IR
        let llvm_module = self.compile_module(mir)?;
        
        // 2. LLVM IR → Object file
        let obj_path = self.emit_object(llvm_module)?;
        
        // 3. Link with runtime
        self.link_executable(obj_path, output, link_options)?;
        
        Ok(())
    }
}
```

### Phase 2: インターフェース定義
```rust
// MIR交換フォーマット（JSON or MessagePack）
#[derive(Serialize, Deserialize)]
pub struct MirPackage {
    pub version: u32,
    pub module: MirModule,
    pub metadata: CompileMetadata,
}

// コンパイラAPI
pub trait MirCompiler {
    fn compile(&self, package: MirPackage, options: CompileOptions) -> Result<Output>;
}
```

### Phase 3: 独立crateの作成
```toml
# nyash-llvm-compiler/Cargo.toml
[package]
name = "nyash-llvm-compiler"
version = "0.1.0"

[dependencies]
inkwell = { version = "0.5", features = ["llvm18-0"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.0", features = ["derive"] }

# Nyash共通定義（MIR構造体など）
nyash-mir = { path = "../nyash-mir" }
```

### Phase 4: CLI実装
```rust
// nyash-llvm-compiler/src/main.rs
#[derive(Parser)]
struct Args {
    /// Input MIR file (JSON format)
    input: PathBuf,
    
    /// Output executable path
    #[arg(short, long)]
    output: PathBuf,
    
    /// Target triple
    #[arg(long, default_value = "native")]
    target: String,
    
    /// Link with nyrt runtime
    #[arg(long)]
    nyrt_path: Option<PathBuf>,
    
    /// Static linking
    #[arg(long)]
    static_link: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // 1. Read MIR
    let mir_package = read_mir_package(&args.input)?;
    
    // 2. Compile to object
    let compiler = LLVMCompiler::new(&args.target)?;
    let obj_file = compiler.compile_to_object(&mir_package)?;
    
    // 3. Link
    let linker = Linker::new()?;
    linker.link_executable(
        &obj_file,
        &args.output,
        LinkOptions {
            runtime: args.nyrt_path,
            static_link: args.static_link,
        },
    )?;
    
    Ok(())
}
```

## 🚀 使用方法

### 統合実行（将来）
```bash
# ワンステップビルド
nyash build --backend llvm --emit exe program.nyash -o program.exe

# デバッグ用分離実行
nyash --dump-mir program.nyash > program.mir.json
nyash-llvm-compiler program.mir.json -o program.exe
```

### パイプライン実行
```bash
# Unix pipe
nyash --dump-mir program.nyash | nyash-llvm-compiler - -o program.exe

# Windows
nyash --dump-mir program.nyash > temp.mir
nyash-llvm-compiler temp.mir -o program.exe
```

## 📊 期待される効果

### ビルド時間の改善
```
現在:
cargo build --release --features llvm    # 5-7分

分離後:
cargo build --release                     # 1-2分（メイン）
cd nyash-llvm-compiler && cargo build     # 2-3分（LLVM部分）

並列ビルド可能 → トータル3分程度
```

### CI/CD の改善
- メインビルドとLLVMビルドを並列実行
- LLVM部分の変更がない場合はキャッシュ利用
- プラットフォーム別ビルドの高速化

## 🔗 関連ファイル
- [lld-strategy.md](lld-strategy.md) - Cranelift版のリンカー戦略
- [../README.md](../README.md) - Phase 15全体計画
- [CURRENT_TASK.md](/mnt/c/git/nyash-project/nyash_self_main/CURRENT_TASK.md) - 現在の実装状況

## 📅 マイルストーン

- [ ] ChatGPT5によるLLVMバックエンド完成
- [ ] オブジェクトファイル生成の安定化
- [ ] build_llvm.shからの移行
- [ ] MIR交換フォーマットの確定
- [ ] nyash-llvm-compiler crate作成
- [ ] CI統合とドキュメント整備

---

> 「重いビルドは開発の敵。必要な部分だけを切り出して、高速な開発サイクルを実現するにゃ！」