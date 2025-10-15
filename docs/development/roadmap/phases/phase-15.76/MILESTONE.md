# Phase 15.76 Milestone & Goal Line

## 🎯 このフェーズのゴール

**extern_c構文 + 凍結EXEの背骨を完成させる**

- Hakoruneから直接C関数を呼べる（Rustの`extern "C"`と同等）
- LLVM BackendをプラグインとしてC ABI経由で呼び出し
- MIR JSON → .o生成 → EXE化の導線確立
- 凍結EXE（hako-frozen-v1.exe）作成の準備完了

---

## ✅ Goal Line（受け入れ基準/DoD）

### 1️⃣ 機能（extern_c）

#### 構文受理
- [x] `extern_c "symbol"(args)` 構文を受理（AST: ExternCCall）
- [x] MIRは統一経路（Call + Callee::Extern("ffi.dynamic.symbol")）
- [ ] VMは動的FFIを実装（0/1/2引数、CString→i64、Fail-Fast）
- [ ] 既定はDeny（最小ホワイトリスト: getpid/strlen/system）

#### 実行モデル
```hakorune
// 基本形
static box SystemInfo {
    get_pid() -> IntegerBox {
        local pid = extern_c "getpid"()
        return pid
    }

    string_length(s: StringBox) -> IntegerBox {
        local len = extern_c "strlen"(s.to_cstring())
        return len
    }
}
```

### 2️⃣ 設定ベース許可

#### ENV変数
- [ ] `HAKO_FFI_ALLOW_LIST=foo,bar` - 既定に追加マージ
- [ ] `HAKO_FFI_ALLOW_ALL=1` - Dev専用（CI不可）

#### TOML設定
- [ ] `hako.toml` または `nyash.toml` の `[ffi.dynamic]`
- [ ] `allow = ["symbol1", "symbol2"]` をマージ

#### 優先順位
```
1. 既定ホワイトリスト（getpid/strlen/system）
2. TOML設定（hako.toml, nyash.toml）
3. ENV変数（HAKO_FFI_ALLOW_LIST）
4. Dev許可（HAKO_FFI_ALLOW_ALL=1）← CI不可
```

### 3️⃣ バックエンドプラグイン（C ABI）⭐

#### ビルド
- [ ] cdylib: `libs/llvm_backend` をビルド可能
- [ ] `cargo build -p llvm_backend` でlibllvm_backend.so生成

#### シンボル
```c
// C ABI
extern "C" {
    int64_t llvm_compile_mir_to_object(
        const char* mir_json,
        const char* output_path
    );
    // 戻り値: 0=成功, -1=失敗
}
```

#### VM側探索対応
- [ ] lib探索パス実装
  - `target/release/`
  - `$NYASH_ROOT/target/release/`
  - カレントディレクトリ
  - `$HAKO_FFI_LIB_PATHS` （コロン区切り）

### 4️⃣ AOT導線（最小）⭐⭐⭐

#### MIR JSON出力
```bash
# Step 1: MIR JSON生成
./hako --backend mir --emit-mir-json program.mir.json program.hako

# Step 2: extern_c経由で .o 生成
./hako tools/aot/emit_object_via_extern_c.hako program.mir.json program.o

# Step 3: clangでリンク
clang program.o -o program.exe libhakorune_kernel.a
```

#### 補助スクリプト
- [ ] `tools/aot/emit_object_via_extern_c.sh <mir.json> <out.o>`
- [ ] `tools/aot/link_exe.sh <program.o> <out.exe>`

### 5️⃣ ドキュメント

- [ ] `docs/reference/language/extern_c.md` - 構文/許可モデル/探索/優先順位
- [ ] `docs/guides/frozen-toolchain.md` - MIR JSON→.o→リンクのクイックレシピ
- [ ] `docs/development/roadmap/phases/phase-15.76/INDEX.md` - Phase概要
- [ ] `docs/development/roadmap/phases/phase-15.76/GUARD_RAILS.md` - 安全性設計

### 6️⃣ スモーク（軽量・観測）

#### quick-selfhost プロファイル
- [ ] `extern_c_strlen_vm.sh` - 基本動作（緑）
- [ ] `extern_c_getpid_vm.sh` - システムコール（緑）
- [ ] `extern_c_system_vm.sh` - コマンド実行（緑）
- [ ] `extern_c_env_allowlist_getppid_vm.sh` - ENV許可（緑）
- [ ] `extern_c_deny_unknown_vm.sh` - Deny確認（緑）

#### ランナー正規化
- [ ] 出力が `OK` / `[PASS]` なら exit 0（集計の安定化）

---

## ❌ Out of Scope（このフェーズではやらない）

### リンクの内蔵
- `extern_c "link_exe"(...)` 等の内蔵リンカーは次フェーズ候補
- 理由: clang/lldを外部依存で十分、スコープ拡大回避

### 凍結EXEの確定配布
- タグ付け（v1.0.0-frozen）
- 配布物生成（tar.gz/zip）
- 理由: AOT導線が安定してから安全に実施

### 複雑なFFI
- 構造体渡し/可変長引数は次フェーズ以降
- 理由: 最小導線確立を優先

---

## 🚀 次フェーズ開始条件（15.77/凍結へ）

### 必須条件
1. **AOTルート再現可能**: MIR JSON→.o→EXEの手順がdocs通りに動作
2. **libs/llvm_backend ビルド緑**: 最低Linux（Ubuntu/WSL）で安定
3. **allowlist/TOML運用ガイド完成**: セキュリティポリシーが明確

### 推奨条件
4. スモークテスト全緑（extern_c関連）
5. 既存quick-selfhostへの影響なし（170 PASS維持）

---

## ⚠️ リスク & 対策

### リスク1: FFI拡大の安全性
**対策**: 既定Deny + ENV/TOMLは追加のみ（Fail-Fast）

### リスク2: 環境差（lib探索/リンク）
**対策**: 検索パスENV + 手順をdocsに固定、疑問はスクリプトで補助

### リスク3: LLVM Backend依存
**対策**: プラグイン化により差し替え容易（セキュリティ境界明確）

---

## 📊 進捗管理

### Week 1（2025-10-14 - 10-18）
- [ ] extern_c構文実装（Parser→AST→MIR）
- [ ] VM動的FFI基盤（0/1/2引数対応）
- [ ] 既定ホワイトリスト実装

### Week 2（2025-10-19 - 10-25）
- [ ] ENV/TOML許可機構
- [ ] libs/llvm_backend プラグイン化
- [ ] lib探索パス実装

### Week 3（2025-10-26 - 11-01）
- [ ] AOT導線整備（MIR JSON→.o→EXE）
- [ ] 補助スクリプト作成
- [ ] ドキュメント完成

### Week 4（2025-11-02 - 11-08）
- [ ] スモークテスト追加
- [ ] 統合テスト（quick-selfhost）
- [ ] レビュー・調整

---

## 🎯 このラインで締めると...

**「凍結EXEを作るための背骨（extern_c + .o生成）」が整う**

→ Rustラインを凍結しても日常開発が回る
→ Hakoruneパーサーのみ開発（2重メンテ回避）
→ 安全な試行錯誤（凍結EXEで戻れる）

---

**作成日**: 2025-10-14
**ChatGPT Goal Line採用**: 2025-10-14
**関連**: Phase 15.76, extern_c戦略, 凍結EXE背骨
