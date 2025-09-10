# Current Task (2025-09-10)

## 🎉 LLVMプラグイン環境変数問題完全解決（2025-09-10）

### ✅ **完了した主要成果**：
1. **プラグイン実装** ✅ - `nyash.plugin.invoke_*`関数はnyrtライブラリに正常実装済み
2. **プラグイン呼び出し** ✅ - 環境変数なしでメソッド呼び出し成功
3. **戻り値型推論修正** ✅ - MIR builder にプラグインメソッド型推論追加
4. **by-id統一完了** ✅ - by-name方式削除、method_id方式に完全統一  
5. **環境変数削減達成** ✅ - `NYASH_LLVM_ALLOW_BY_NAME=1`削除完了
6. **シンプル実行実現** ✅ - `./target/release/nyash --backend llvm program.nyash`

### 🔬 **現在の動作状況**（2025-09-10テスト結果）：
- **プラグイン実行** ✅ - CounterBox、FileBox正常実行
- **副作用処理** ✅ - ファイル書き込み、カウンター増加成功
- **method_id注入** ✅ - `[LLVM] method_id injected: 4-5 places`
- **型推論動作** ✅ - `[BUILDER] Type inference: CounterBox get -> Integer`
- **コンパイル成功** ✅ - 環境変数なしでLLVMバックエンド動作

### 🔍 **残存課題（深堀り調査必要）**：

#### 1. **プラグイン戻り値表示問題** 🔶 **←最重要**
**症状**:
- `CounterBox.get()` → 数値戻り値がprint()で空白
- `FileBox.read(path)` → 文字列戻り値がprint()で空白  
- `FileBox.exists(path)` → if条件では動作、変数格納で失敗
- `local x = 42; print(x)` → 正常（問題はプラグイン戻り値のみ）

**推測される問題層**:
- **プラグインランタイム層**: nyrtライブラリの戻り値処理
- **MIR変数代入処理**: BoxCall結果の変数格納メカニズム  
- **LLVM型変換**: プラグイン戻り値→LLVM値の変換処理
- **print()関数型処理**: プラグイン値のprint()認識問題

#### 2. **IF文処理の不整合** 🔶
**症状**:
- `if f.exists("file")` → 正常動作（条件判定OK）
- `local result = f.exists("file")` → 変数格納失敗
- `print(result)` → 空白表示

**推測**: 戻り値がif条件では評価されるが、変数代入で失われる

#### 3. **MIR生成層の潜在問題** 🟡
**推測される問題**:
- BoxCall戻り値のMIR変数への代入処理
- 型変換処理でのメタデータ不整合
- プラグイン戻り値の寿命管理問題

### 📊 **修正済みファイル**：
- **src/mir/builder.rs** - プラグインメソッド戻り値型推論追加
- **src/backend/llvm/compiler.rs** - by-name方式削除、method_id統一
- **CLAUDE.md** - 環境変数セクション更新、コマンド簡素化
- **README.md / README.ja.md** - 環境変数説明削除
- **tools/llvm_smoke.sh** - テストスクリプト環境変数削除

### 🎯 **次期深堀り調査対象**：
1. **プラグインランタイム戻り値パス詳細調査** - `crates/nyrt/src/lib.rs`
2. **LLVM BoxCall戻り値処理詳細分析** - `src/backend/llvm/compiler.rs` 戻り値変換
3. **MIR変数代入メカニズム調査** - BoxCall→変数の代入処理  
4. **print()関数とプラグイン値の相性調査** - 型認識処理

## 🎯 Restart Notes — Ny Syntax Alignment (2025‑09‑07)

### 目的
- Self‑Hosting 方針（Nyのみで工具を回す前段）に合わせて、Ny 構文とコーディング規約を明示化し、最小版ツールの完走性を優先。

### Ny 構文（実装時の基準）
- 1行1文／セミコロン非使用。
- break / continue を使わない（関数早期 return、番兵条件、if 包みで置換）。
- else は直前の閉じ波括弧と同一行（`} else {`}）。
- 文字列の `"` と `\` は確実にエスケープ。
- 反復は `loop(条件) { …; インクリメント }` を基本とし、必要に応じて「関数早期 return」型で早期脱出。

### 短期タスク（Syntax 合意前提で最小ゴール）
1) include のみ依存木（Nyのみ・配列/マップ未使用）を完走化
   - `apps/selfhost/tools/dep_tree_min_string.nyash`
   - FileBox/PathBox + 文字走査のみで JSON 構築（配列/マップに頼らない）
   - `make dep-tree` で `tmp/deps.json` を出力
2) using/module 対応は次段（構文・優先順位をユーザーと再すり合わせ後）
   - 優先: `module > 相対 > using-path`、曖昧=エラー、STRICT ゲート（要相談）
3) ブリッジ Stage 1 は保留
   - `NYASH_DEPS_JSON=<path>` 読み込み（ログ出力のみ）を最小パッチで用意（MIR/JIT/AOT は不変）