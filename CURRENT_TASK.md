# Current Task (2025-09-11)

> Phase 15 LLVM‑only notes (authoritative)
>
> - LLVM AOT is the stable/authoritative path. VM/Cranelift JIT/AOT and the interpreter are not MIR14‑ready in this phase.
> - Fallback logic must be minimal. Prefer fixing MIR type annotations over adding broad implicit conversions.
> - ExternCall (console/debug) selects C‑string vs handle variants by the argument IR type.
> - StringBox: NewBox keeps i8* fast path (no birth); print/log choose automatically based on IR type.

## 🎉 LLVMプラグイン戻り値表示問題修正進行中（2025-09-10）

### ✅ **完了した主要成果**：
1. **プラグイン実装** ✅ - `nyash.plugin.invoke_*`関数はnyrtライブラリに正常実装済み
2. **プラグイン呼び出し** ✅ - 環境変数なしでメソッド呼び出し成功
3. **戻り値型推論修正** ✅ - MIR builder にプラグインメソッド型推論追加
4. **by-id統一完了** ✅ - by-name方式削除、method_id方式に完全統一  
5. **環境変数削減達成** ✅ - `NYASH_LLVM_ALLOW_BY_NAME=1`削除完了
6. **シンプル実行実現** ✅ - `./target/release/nyash --backend llvm program.nyash`
7. **codex TLV修正マージ完了** ✅ - プラグイン戻り値TLVタグ2/6/7対応（2000行修正）
8. **console.log ハンドル対応** ✅ - 不正なi64→i8*変換を修正、ハンドル対応関数に変更
9. **型変換エラー解消** ✅ - bool(i1)→i64拡張処理追加でLLVM検証エラー修正

### 🔬 **現在の動作状況**（2025-09-11 最新）：
- **プラグイン実行** ✅ - CounterBox、FileBox正常実行（副作用OK）
- **型エラー解消** ✅ - LLVM関数検証エラー修正済み
- **コンパイル成功** ✅ - 環境変数なしでLLVMバックエンド動作
- **条件分岐動作** ✅ - `if f.exists("file")` → "TRUE"/"FALSE"表示OK
 - **LLVMコード生成の分割** ✅ - `codegen/` 下にモジュール化（types/instructions）
 - **BoxCall Lower 移譲** ✅ - 実行経路は `instructions::lower_boxcall` に一本化（旧実装は到達不能）
 - **スモーク整理** ✅ - `tools/llvm_smoke.sh` を環境変数整理。プラグイン戻り値スモーク追加

### 🔍 **根本原因判明**：
**Task先生の詳細技術調査により特定**：
- **不正なハンドル→ポインタ変換**: `build_int_to_ptr(iv, i8p, "arg_i2p")` でハンドル値（3）を直接メモリアドレス（0x3）として扱う不正変換
- **修正完了**: console.log を `nyash.console.log_handle(i64) -> i64` に変更、ハンドルレジストリ活用

### 🔍 **残存課題（新）**：

#### 1. **プラグイン戻り値表示問題** 🔶 **←現在調査中**
**症状（修正後も継続）**:
- `CounterBox.get()` → 数値戻り値がprint()で空白
- `FileBox.read(path)` → 文字列戻り値がprint()で空白  
- `FileBox.exists(path)` → if条件では動作、変数格納で失敗
- `local x = 42; print(x)` → 正常（問題はプラグイン戻り値のみ）

**残る問題の推定**:
- print()以外のExternCall処理にも同様の問題がある可能性
- MIR変数代入処理での型情報不整合
- プラグイン戻り値のハンドル→実値変換が不完全

#### 2. **LLVM BinOp 文字列連結まわり（新規スモークで再現）** 🔶
現象:
- 追加スモーク `ny-plugin-ret-llvm-smoke` で LLVM AOT オブジェクト生成中に `binop type mismatch` が発生
- ケース: `print("S=" + t)`（`t` は `StringBox.concat("CD")` の戻り値）

暫定分析:
- `+` 連結の BinOp Lower は (ptr+ptr / ptr+int / int+ptr) を NyRT シム経由で処理する設計
- `t` のLowerが i64 handleのままになっている/もしくは型注釈不足で ptr へ昇格できていない可能性
- `instructions::lower_boxcall` 内の `concat` 特例（f64 fast-path）は撤去済み。以降は tagged invoke → dst 型注釈に応じて i64→ptr へ変換する想定

対応方針:
1) MIR の `value_types` に BoxCall 戻り値（StringBox）の型注釈が入っているか確認（不足時は MIR 側で注入）
2) BinOp 連結経路は fallback として (ptr + 非ptr) / (非ptr + ptr) に対して handle想定の i64→ptr 昇格を再確認
3) 上記fixの後、スモーク `NYASH_LLVM_PLUGIN_RET_SMOKE=1` を通し、VM/LLVM 一致を確認

メモ:
- 現状でも BoxCall は新経路に完全委譲済み。旧実装は到達不能（削除準備OK）。

### 📊 **修正済みファイル**：
- **src/mir/builder.rs** - プラグインメソッド戻り値型推論追加
- **src/backend/llvm/compiler.rs** - by-name方式削除、method_id統一
- **src/backend/llvm/compiler/codegen/mod.rs** - 分割に伴う委譲（Const/Compare/Return/Jump/Branch/Extern/NewBox/Load/Store/BoxCall）
- **src/backend/llvm/compiler/codegen/types.rs** - 型変換/分類/型マップ（新）
- **src/backend/llvm/compiler/codegen/instructions.rs** - Lower群を実装（新）
- **tools/llvm_smoke.sh** - LLVM18 prefix整理・プラグイン戻り値スモーク追加
- **apps/tests/ny-plugin-ret-llvm-smoke/main.nyash** - プラグイン戻り値スモーク（新）
- **CLAUDE.md** - 環境変数セクション更新、コマンド簡素化
- **README.md / README.ja.md** - 環境変数説明削除
- **tools/llvm_smoke.sh** - テストスクリプト環境変数削除

### 🎯 **次期深堀り調査対象**：
1. **プラグインランタイム戻り値パス詳細調査** - `crates/nyrt/src/lib.rs`
2. **LLVM BoxCall戻り値処理詳細分析** - `src/backend/llvm/compiler/real.rs` 戻り値変換
3. **MIR変数代入メカニズム調査** - BoxCall→変数の代入処理  
4. **print()関数とプラグイン値の相性調査** - 型認識処理
5. **BinOp 連結の保険見直し** - 片方が i64 (handle) の場合に ptr に昇格する安全弾性
6. **BoxCall 旧実装の物理削除** - スモークグリーン後に `mod.rs` の死コードを除去

### ✅ **リファクタリング完了（2025-09-11）**：
**PR #136マージ済み** - LLVMコンパイラモジュール化：
- `compiler.rs` → 4モジュールに分割（mod.rs, real.rs, mock.rs, mock_impl.in.rs）
- プラグインシグネチャ読み込みを `plugin_sigs.rs` に移動
- BoxタイプID読み込みを `box_types.rs` に移動
- PR #134の型情報処理を完全保持

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
