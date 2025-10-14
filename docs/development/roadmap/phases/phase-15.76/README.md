# Phase 15.76 - extern_c & Frozen Toolchain

**期間**: 2025-10-14 - 2025-11-08（4週間）
**状態**: 🚧 計画中（ChatGPT実装開始準備）

---

## 🎯 このフェーズで実現すること

**extern_c構文 + 凍結EXE（hako-frozen-v1.exe）の背骨確立**

1. **extern_c構文**: HakoruneからC関数を直接呼び出し（Rust `extern "C"` 相当）
2. **LLVM Backendプラグイン化**: C ABI経由で.o生成
3. **AOT導線確立**: MIR JSON → .o → EXE
4. **凍結EXE準備**: Rust層を凍結しても日常開発が回る

---

## 💡 発見の経緯（2025-10-14）

**User（tomoaki）の直感**:
> 「HakoruneからC ABIを呼びたい」
> → 「あ、Rustもやってる！`extern "C"`」
> → 「じゃあHakoruneでも`extern_c`構文を！」

**結果**: 業界標準パターン（Rust stage0, Go 1.4 frozen）への独自収束 ⭐論文価値

---

## 📁 ドキュメント

### 🎯 必読
- **[INDEX.md](INDEX.md)** - Phase全体像・実装の流れ
- **[MILESTONE.md](MILESTONE.md)** ⭐最重要 - ゴールライン（DoD）
- **[GUARD_RAILS.md](GUARD_RAILS.md)** - セキュリティ設計

### 📚 設計資料
- **[BUILTIN_VS_PLUGIN_DESIGN.md](BUILTIN_VS_PLUGIN_DESIGN.md)** - 内蔵 vs プラグイン方針

---

## 🚀 クイックスタート（実装後）

### 基本的なextern_c呼び出し

```hakorune
// プロセスID取得
static box SystemInfo {
    get_pid() -> IntegerBox {
        local pid = extern_c "getpid"()
        return pid
    }
}
```

### LLVM Backendプラグイン呼び出し

```hakorune
// MIR JSON → .o 生成
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

### AOT導線

```bash
# Step 1: MIR JSON生成
./hako --backend mir --emit-mir-json program.mir.json program.hako

# Step 2: .o生成
./hako compiler_wrapper.hako program.mir.json program.o

# Step 3: リンク
clang program.o -o program.exe libhakorune_kernel.a
```

---

## 🔒 セキュリティ

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

### 開発時のみ全許可（CI不可）

```bash
HAKO_FFI_ALLOW_ALL=1 ./hako program.hako  # ⚠️ Dev only
```

詳細: [GUARD_RAILS.md](GUARD_RAILS.md)

---

## 📊 進捗（Week 1-4）

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

## 🎯 成功基準

### 必須（DoD）
1. ✅ extern_c構文が動作（getpid/strlen/system）
2. ✅ libs/llvm_backend プラグイン化完了
3. ✅ AOT導線再現可能（MIR JSON→.o→EXE）
4. ✅ allowlist/TOML運用ガイド完成

### 推奨
5. ✅ スモークテスト全緑（extern_c関連）
6. ✅ 既存テスト影響なし（170 PASS維持）

---

## 🚀 次のフェーズ（Phase 15.77/凍結へ）

### 開始条件
- AOTルートが安定稼働
- libs/llvm_backend ビルド緑（Linux）
- セキュリティ運用ガイド完成

### Phase 15.77 目標
- **凍結EXE確定**: hako-frozen-v1.exe タグ付け・配布
- **Rust層最小化**: 100-200行（VM実行エンジンのみ）
- **脱Rust加速**: Hakoruneパーサーのみ開発

---

## 📚 関連リソース

### 前フェーズ
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

## 💬 質問・議論

### ChatGPTと相談
このフェーズはChatGPTが主導で実装予定。設計判断はChatGPT↔tomoaki↔Claude三者で合意形成。

### Claudeのレビュー担当
- コード品質チェック
- セキュリティレビュー
- ドキュメント整備

---

**作成日**: 2025-10-14
**実装開始**: ChatGPT（2025-10-14 Stage 4完了後）
**レビュー**: Claude（実装後）
