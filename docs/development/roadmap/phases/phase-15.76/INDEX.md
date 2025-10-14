# Phase 15.76 - extern_c & Frozen Toolchain INDEX

## 🎯 Phase概要

**extern_c構文実装 + 凍結EXE（hako-frozen-v1.exe）の背骨確立**

Hakoruneから直接C関数を呼び出し、LLVM BackendをプラグインとしてC ABI経由で利用可能にする。これにより、Rust層を凍結しても日常開発が回る基盤を確立。

---

## 📁 ドキュメント構成

### 🎯 計画・戦略
- **[MILESTONE.md](MILESTONE.md)** ⭐最重要 - ゴールライン（受け入れ基準/DoD）
- **[BUILTIN_VS_PLUGIN_DESIGN.md](BUILTIN_VS_PLUGIN_DESIGN.md)** - 内蔵 vs プラグイン設計方針
- **[GUARD_RAILS.md](GUARD_RAILS.md)** - 安全性・セキュリティ設計（作成予定）

### 📚 技術資料
- **[docs/reference/language/extern_c.md](../../reference/language/extern_c.md)** - extern_c構文リファレンス（作成予定）
- **[docs/guides/frozen-toolchain.md](../../guides/frozen-toolchain.md)** - 凍結ツールチェーン使用ガイド（作成予定）

### 🔧 実装資料
- **[libs/llvm_backend/](../../../../../../../libs/llvm_backend/)** - LLVM Backendプラグイン実装（作成予定）
- **[tools/aot/](../../../../../../../tools/aot/)** - AOT補助スクリプト（作成予定）

---

## 🚀 Phase 15.76 の核心アイデア

### 💡 発見の経緯（2025-10-14）

**User（tomoaki）の思考プロセス**:
1. 「HakoruneからC ABIを呼びたい」
2. 「あ、Rustもやってる！`extern "C"`」
3. 「じゃあHakoruneでも`extern_c`構文を！」

**結果**: 業界標準パターンへの独自収束 ⭐論文価値

### 🎯 3つのルール

1. **単一パーサ**: Hakoruneパーサーのみ開発（Rust凍結）
2. **stage0（凍結EXE）**: hako-frozen-v1.exe 基盤使用
3. **床=Rust / 家=Hakorune**: Rustは基盤、Hakoruneは拡張

### ⚡ なぜこの戦略

- **開発速度**: Rustビルド3-5分 → 凍結EXE即座
- **決定性**: パーサ1つのみ保守 → 2重メンテ回避
- **回帰コスト**: 凍結EXEで戻れる → 安全試行錯誤

---

## 📋 実装の全体像

### 1️⃣ extern_c構文（Hakorune言語側）

```hakorune
// 基本形: C関数を直接呼び出し
static box SystemInfo {
    get_pid() -> IntegerBox {
        local pid = extern_c "getpid"()
        return pid
    }

    string_length(s: StringBox) -> IntegerBox {
        local len = extern_c "strlen"(s.to_cstring())
        return len
    }

    run_command(cmd: StringBox) -> IntegerBox {
        local result = extern_c "system"(cmd.to_cstring())
        return result
    }
}
```

### 2️⃣ バックエンドプラグイン（C ABI出力）

```hakorune
// LLVM Backendをプラグインとして呼び出し
static box Compiler {
    compile_to_object(mir: StringBox, out: StringBox) -> IntegerBox {
        local result = extern_c "llvm_compile_mir_to_object"(
            mir.to_cstring(),
            out.to_cstring()
        )
        return result
    }
}
```

### 3️⃣ AOT導線

```bash
# Step 1: MIR JSON生成
./hako --backend mir --emit-mir-json program.mir.json program.hako

# Step 2: extern_c経由で .o 生成
./hako compiler_wrapper.hako program.mir.json program.o

# Step 3: clangでリンク
clang program.o -o program.exe libhakorune_kernel.a
```

---

## 🔒 セキュリティ設計

### 既定Deny + Allowlist

```toml
# hako.toml
[ffi.dynamic]
allow = [
    "getpid",     # 既定
    "strlen",     # 既定
    "system",     # 既定
    "getppid",    # 追加
]
```

### ENV変数

```bash
# 開発時のみ許可（CI不可）
HAKO_FFI_ALLOW_ALL=1 ./hako program.hako

# 追加許可（マージ）
HAKO_FFI_ALLOW_LIST=getppid,getuid ./hako program.hako
```

### 優先順位

```
1. 既定ホワイトリスト（最小セット）
2. TOML設定（hako.toml, nyash.toml）
3. ENV変数（HAKO_FFI_ALLOW_LIST）
4. Dev許可（HAKO_FFI_ALLOW_ALL=1）← CI不可
```

---

## 📦 内蔵 vs プラグイン方針

### ✅ 凍結EXEに内蔵（静的同梱）

- **Core Boxes**: String/Integer/Bool/Array/Map
- **Console/Time**: 診断・時刻（最小）
- **JSON**: MIR JSONブリッジ（最小）
- **File**: 読み込み専用（最小）
- **extern_c runtime**: allowlist機構

### 🔌 プラグイン（動的ロード）

- **libllvm_backend.so** ⭐C ABI出力（.o生成）
- **Network/HTTP**: セキュリティ影響大
- **拡張FS**: 書き込み専用
- **重量物**: 圧縮/暗号/画像/正規表現

詳細: [BUILTIN_VS_PLUGIN_DESIGN.md](BUILTIN_VS_PLUGIN_DESIGN.md)

---

## 🧪 テスト戦略

### quick-selfhost プロファイル

```bash
# 基本動作
tools/smokes/v2/profiles/quick-selfhost/extern_c_strlen_vm.sh
tools/smokes/v2/profiles/quick-selfhost/extern_c_getpid_vm.sh
tools/smokes/v2/profiles/quick-selfhost/extern_c_system_vm.sh

# 許可機構
tools/smokes/v2/profiles/quick-selfhost/extern_c_env_allowlist_getppid_vm.sh
tools/smokes/v2/profiles/quick-selfhost/extern_c_deny_unknown_vm.sh
```

### 期待結果

```
extern_c_strlen_vm... [PASS] (0.5s)
extern_c_getpid_vm... [PASS] (0.3s)
extern_c_system_vm... [PASS] (0.4s)
extern_c_env_allowlist_getppid_vm... [PASS] (0.3s)
extern_c_deny_unknown_vm... [PASS] (0.3s)
```

---

## 📊 進捗トラッキング

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

詳細: [MILESTONE.md](MILESTONE.md)

---

## 🚀 次のフェーズ（Phase 15.77/凍結へ）

### 開始条件
1. AOTルート再現可能（MIR JSON→.o→EXE）
2. libs/llvm_backend ビルド緑（Linux）
3. allowlist/TOML運用ガイド完成

### Phase 15.77 目標
- **凍結EXE確定**: hako-frozen-v1.exe タグ付け・配布
- **Rust層最小化**: 100-200行（VM実行エンジンのみ）
- **脱Rust加速**: Hakoruneパーサーのみ開発

---

## 📚 関連リソース

### 設計資料
- [Phase 15.75 - 脱Rust大作戦](../phase-15.75/)
- [Stage 4 - Dual Parser Harness](../phase-15.75/stage-4/)
- [ChatGPT extern_c戦略](../phase-15.75/stage-4-chatgpt/EXTERN_C_SELFHOST_STRATEGY.md)

### 論文資料
- [Rapid Self-Hosting Paper](../../../../private/papers-active/rapid-selfhost-ai-collaboration/)
- [Pattern Rediscovery Evidence](../../../../private/papers-active/rapid-selfhost-ai-collaboration/03_DATA_ANALYSIS.md)

### 業界標準パターン
- **Rust**: stage0（凍結ツールチェーン）
- **Go**: Go 1.4 frozen（bootstrap用）
- **OCaml**: ocamlc frozen（自己ホスト用）

---

**作成日**: 2025-10-14
**Phase開始**: 2025-10-14
**想定期間**: 4週間（Week 1-4）
