# 🚨 WASM/AOT WASM 現在の問題・制限事項

## 📅 最終更新: 2025-08-15

## 🔴 **緊急度: 高**

### 1. **BoxCall命令未実装**
**問題**: `toString()`, `print()` 等のBox メソッド呼び出しがWASMで未対応

**エラー例**:
```bash
❌ WASM compilation error: Unsupported instruction: BoxCall { 
    dst: Some(ValueId(6)), 
    box_val: ValueId(4), 
    method: "toString", 
    args: [], 
    effects: EffectMask(16) 
}
```

**影響範囲**:
- 基本的なBox操作（toString, print等）が全て使用不可
- 実用的なNyashプログラムがWASMでコンパイル不可

**修正必要ファイル**:
- `src/backend/wasm/codegen.rs`: BoxCall命令の実装

---

### 2. **wasmtimeバージョン互換性問題**
**問題**: コンパイル時wasmtimeとシステムwasmtimeのバージョン不一致

**エラー例**:
```bash
Error: Module was compiled with incompatible Wasmtime version '18.0.4'
System wasmtime: 35.0.0
```

**原因**:
```toml
# Cargo.toml
wasmtime = "18.0"      # コンパイル時

# システム
wasmtime 35.0.0        # 実行時
```

**影響**:
- AOT (.cwasm) ファイルが実行不可
- 配布用実行ファイル生成失敗

**修正策**:
1. **短期**: Cargo.tomlのwasmtimeバージョン更新
2. **長期**: 互換性保証メカニズム実装

---

### 3. **WASM出力バイナリエラー**
**問題**: WAT → WASM変換でUTF-8エラー発生

**エラー例**:
```bash
❌ Generated WASM is not valid UTF-8
```

**推測原因**:
- WAT形式の生成に問題
- wabt crateとの連携ミス

**修正必要箇所**:
- `src/backend/wasm/codegen.rs`: WAT生成ロジック
- WASM バイナリ出力パイプライン

---

## 🟠 **緊急度: 中**

### 4. **RuntimeImports未実装機能**
**問題**: WASMで必要なランタイム関数が部分実装

**未実装機能**:
- Box メモリ管理 (malloc, free)
- 型キャスト・変換
- 配列・Map操作
- 例外ハンドリング

**ファイル**: `src/backend/wasm/runtime.rs`

---

### 5. **メモリ管理最適化不足**
**問題**: WASMメモリレイアウトが非効率

**課題**:
- Box ヘッダーサイズ固定 (12 bytes)
- ガベージコレクション未実装
- メモリ断片化対策なし

**ファイル**: `src/backend/wasm/memory.rs`

---

## 🟡 **緊急度: 低**

### 6. **デバッグ情報不足**
**問題**: WASM実行時のエラー情報が不十分

**改善点**:
- ソースマップ生成
- スタックトレース詳細化
- ブレークポイント対応

---

### 7. **最適化機能未実装**
**問題**: WASM出力が最適化されていない

**未実装最適化**:
- デッドコード除去
- インライン展開
- 定数畳み込み

---

## 📊 **問題優先度マトリクス**

| 問題 | 緊急度 | 重要度 | 修正工数 | 優先順位 |
|------|--------|--------|----------|----------|
| BoxCall未実装 | 高 | 高 | 中 | **1** |
| wasmtimeバージョン | 高 | 高 | 低 | **2** |
| WASM出力エラー | 高 | 中 | 中 | **3** |
| RuntimeImports | 中 | 高 | 高 | **4** |
| メモリ管理 | 中 | 中 | 高 | **5** |
| デバッグ情報 | 低 | 中 | 中 | **6** |
| 最適化 | 低 | 低 | 高 | **7** |

## 🎯 **修正ロードマップ**

### Phase 1: 基本機能復旧 (1週間)
1. **BoxCall命令実装**
2. **wasmtimeバージョン統一**
3. **WASM出力エラー修正**

### Phase 2: 機能拡充 (2週間)
4. **RuntimeImports完全実装**
5. **メモリ管理改善**

### Phase 3: 品質向上 (1週間)
6. **デバッグ情報強化**
7. **基本最適化実装**

## 📝 **テスト必要項目**

### 基本動作テスト
```bash
# BoxCall テスト
./target/release/nyash --compile-wasm test_boxcall.nyash

# AOT テスト  
./target/release/nyash --aot test_simple.nyash
wasmtime --allow-precompiled test_simple.cwasm
```

### 互換性テスト
```bash
# バージョン確認
cargo tree | grep wasmtime
wasmtime --version

# 実行テスト
wasmtime test_output.wasm
```

---

**🎯 目標**: Phase 1完了でWASM基本機能復旧、Nyash WASMが実用レベルに到達