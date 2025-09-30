# AOTスタンドアロン実行ファイル生成

## 📋 概要

**発見日**: 2025-09-30
**優先度**: 🟡 中（Phase 16-17で実装予定）
**影響範囲**: AOTコンパイラ・配布システム

## 🎯 問題

現在のAOTコンパイラは部分的実装：

### 該当箇所

#### `src/backend/aot/mod.rs:85`
```rust
// TODO: Implement full standalone executable generation
```

### 現状の機能
- オブジェクトファイル（`.o`）生成は可能
- LLVM IRからネイティブコードへの変換は実装済み

### 未実装機能
- ✗ ランタイムとのリンク
- ✗ 実行可能バイナリ（`.exe`/ELF）生成
- ✗ クロスコンパイル対応
- ✗ 最適化レベル制御

## 💡 解決策案

### Option A: LLVMリンカー統合（推奨）
```rust
pub fn generate_executable(
    mir: &MirProgram,
    output_path: &str,
    options: &AOTOptions,
) -> Result<(), String> {
    // 1. LLVM IRからオブジェクトファイル生成
    let obj_path = format!("{}.o", output_path);
    generate_object_file(mir, &obj_path)?;

    // 2. ランタイムライブラリパス取得
    let nyrt_path = find_nyrt_library()?;

    // 3. システムリンカー呼び出し
    let linker_cmd = match options.target {
        Target::Linux => format!("clang {} {} -o {} -lpthread -ldl", obj_path, nyrt_path, output_path),
        Target::Windows => format!("link.exe {} {} /OUT:{} /SUBSYSTEM:CONSOLE", obj_path, nyrt_path, output_path),
        Target::MacOS => format!("clang {} {} -o {} -framework CoreFoundation", obj_path, nyrt_path, output_path),
    };

    run_command(&linker_cmd)?;
    Ok(())
}
```

**利点**:
- システムリンカー活用（実績あり）
- クロスコンパイル対応容易
- 最適化フラグ制御可能

**欠点**:
- 外部ツール依存（clang/link.exe必要）

### Option B: LLD組み込み
LLVMのリンカー（LLD）を直接使用：

```rust
use llvm_sys::linker::*;

pub fn link_with_lld(
    object_files: &[&str],
    output_path: &str,
) -> Result<(), String> {
    // LLDを直接呼び出し
    unsafe {
        LLDLinkELF(/* ... */);
    }
}
```

**利点**:
- 外部ツール不要
- 完全制御可能

**欠点**:
- llvm-sysのunsafe API多用
- 実装複雑

### Option C: Cargo統合
Rustのビルドシステムを活用：

```rust
// Cargo.tomlを自動生成
pub fn generate_cargo_project(
    mir: &MirProgram,
    output_dir: &str,
) -> Result<(), String> {
    // 1. Rustコード生成（MIR → Rust）
    let rust_code = transpile_mir_to_rust(mir)?;

    // 2. Cargo.toml生成
    let cargo_toml = format!(r#"
[package]
name = "nyash-aot-output"
version = "0.1.0"

[dependencies]
nyash-runtime = {{ path = "{}" }}
"#, nyrt_path);

    // 3. cargo build --release実行
    run_command("cargo build --release")?;
    Ok(())
}
```

**利点**:
- Rustエコシステム活用
- クロスコンパイル容易（cargo-cross）
- 最適化自動

**欠点**:
- Cargo必須（配布時も）
- ビルド時間長い

## 🚀 実装ステップ

### Phase 1: 最小実装（Option A） - 2-3時間
1. システムリンカー検出ロジック
2. nyrtライブラリパス解決
3. リンクコマンド生成・実行
4. Linux/Windows/macOS対応

### Phase 2: クロスコンパイル対応 - 3-4時間
1. ターゲット指定（`--target x86_64-pc-windows-gnu`）
2. クロスリンカー対応
3. ランタイムライブラリ複数バージョン管理

### Phase 3: 最適化制御 - 2-3時間
1. 最適化レベル指定（`-O0`～`-O3`）
2. LTO（Link Time Optimization）
3. デバッグ情報生成制御

## 📊 影響範囲

### 修正必要ファイル
- `src/backend/aot/mod.rs` - 実装本体
- `src/backend/aot/linker.rs` - リンカー抽象化（新規）
- `src/runner/modes/aot.rs` - CLI統合
- `Cargo.toml` - nyash-runtime依存追加

### 新規追加
- `crates/nyash_runtime/` - ランタイムライブラリ（既存）
- `tools/aot_smoke.sh` - AOTスモークテスト

### テスト追加
- `tests/aot_hello_world.rs` - 基本実行ファイル生成
- `tests/aot_cross_compile.rs` - クロスコンパイル
- スモークテスト: 生成した実行ファイル動作確認

## 🎯 成功基準

- ✅ Linux/Windows/macOSで実行可能バイナリ生成
- ✅ 生成バイナリが外部依存なしで動作
- ✅ ファイルサイズ <10MB（最適化後）
- ✅ クロスコンパイル対応（Linux→Windows等）
- ✅ 既存のMIRプログラムすべてがAOT可能

## 🔗 関連資料

- [Phase 16-17計画](../../../../development/roadmap/phases/phase-16/)
- [LLVM Backend設計](../../../../reference/backends/llvm-design.md)
- `src/backend/aot/mod.rs:85` - この実装により解決

## 📝 補足

**優先度判断**:
- Phase 15（セルフホスティング）では不要
- Phase 16-17（配布・最適化）で必須
- 現時点では低優先度

**実装タイミング**: Phase 15完了後、Phase 16で実装推奨

**代替手段**: 現時点では以下で代用可能：
```bash
# オブジェクトファイル生成
./target/release/nyash --emit-obj program.nyash -o program.o

# 手動リンク
clang program.o libnyrt.a -o program
```