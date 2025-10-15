# 🎯 Hakorune環境・スモークテスト 簡略化提案

## 📊 現状の複雑度

- **複雑度スコア**: 72 (Env:34 + Mirrors:13 + Functions:25)
- **主要問題**:
  1. NYASH_*/HAKO_* 二重化 (13ペア = 26変数)
  2. run_nyash_vm 複雑化 (79行、6責務)
  3. NYASH_MODULES 可読性 (510文字1行)
  4. filter_noise 肥大化 (85行、74ルール)

---

## 🔥 提案1: NYASH_*/HAKO_* 統一 (即効性★★★)

### 現状
```bash
export NYASH_USING=1
set_if_unset HAKO_USING "$NYASH_USING"  # 13箇所で重複
```

### 提案A: HAKO_* 完全移行
```bash
# hakoコマンド自体がHAKO_*のみ読む
export HAKO_USING=1
# NYASH_* は削除
```

**効果**: 13変数削減、ドキュメント半減

### 提案B: 後方互換レイヤー
```bash
# Rustコード内でNYASH_*もサポート
if let Ok(v) = std::env::var("HAKO_USING")
    .or_else(|_| std::env::var("NYASH_USING")) {  // fallback
```

**効果**: Bash側のミラーリング削除、Rust側で吸収

---

## 🔥 提案2: run_nyash_vm 分割 (即効性★★★)

### 現状の責務
1. 引数処理 (-c option → tmpfile)
2. ASI修正 (sed magic)
3. プラグイン設定注入
4. タイムアウト管理
5. ENV sanitization
6. filter_noise連携

### 提案: 単一責任化
```bash
# 新構造
run_nyash_vm() {
  local program="$1"; shift
  program=$(_prepare_program "$program")      # 1. 引数→ファイル変換
  program=$(_apply_asi_fixes "$program")      # 2. ASI修正
  local env_args=$(_build_env_args)           # 3. ENV構築
  _execute_with_timeout "$program" "${env_args[@]}" "$@" | filter_noise
}

_prepare_program() { ... }    # -c handling only
_apply_asi_fixes() { ... }    # sed/awk only
_build_env_args() { ... }     # PLUGIN_POLICY etc.
_execute_with_timeout() { ... }  # timeout wrapper
```

**効果**: 79行 → 4×15行、各関数15行以下、テスト容易

---

## 🔥 提案3: NYASH_MODULES 整形 (即効性★★★)

### 現状 (510文字1行)
```bash
export NYASH_MODULES="selfhost.vm.mir_min=selfhost/vm/boxes/mir_vm_min.hako,selfhost.vm.handlers=selfhost/vm/boxes/op_handlers.hako,..."
```

### 提案A: 複数行化
```bash
export NYASH_MODULES=""
NYASH_MODULES+="selfhost.vm.mir_min=selfhost/vm/boxes/mir_vm_min.hako,"
NYASH_MODULES+="selfhost.vm.handlers=selfhost/vm/boxes/op_handlers.hako,"
NYASH_MODULES+="selfhost.vm.scanner=selfhost/vm/boxes/instruction_scanner.hako,"
NYASH_MODULES+="selfhost.vm.json_cur=selfhost/vm/boxes/json_cur.hako,"
NYASH_MODULES+="selfhost.json.core.string_scan=selfhost/shared/json/core/string_scan.hako,"
NYASH_MODULES+="selfhost.json.utils.json_frag=selfhost/shared/json/utils/json_frag.hako,"
NYASH_MODULES+="selfhost.shared.mir.builder=selfhost/shared/mir/block_builder_box.hako,"
NYASH_MODULES+="selfhost.shared.mir.schema=selfhost/shared/mir/mir_schema_box.hako"
```

**効果**: 可読性向上、編集容易、diff明確

### 提案B: 配列 → 結合
```bash
declare -a MODULE_MAPPINGS=(
  "selfhost.vm.mir_min=selfhost/vm/boxes/mir_vm_min.hako"
  "selfhost.vm.handlers=selfhost/vm/boxes/op_handlers.hako"
  "selfhost.vm.scanner=selfhost/vm/boxes/instruction_scanner.hako"
  # ...
)
export NYASH_MODULES=$(IFS=,; echo "${MODULE_MAPPINGS[*]}")
```

**効果**: 配列操作可能、動的追加容易

---

## ⚡ 提案4: filter_noise 設定ファイル化 (中期)

### 現状 (85行、74ルール)
```bash
filter_noise() {
  grep -v "^\[UnifiedBoxRegistry\]" \
  | grep -v "^\[FileBox\]" \
  # ... 70+ lines
}
```

### 提案: ルールファイル分離
```bash
# configs/filter_noise_rules.txt
^\[UnifiedBoxRegistry\]
^\[FileBox\]
^Net plugin:
# ... (コメント可能)

# lib/filter_noise.sh
filter_noise() {
  local rules="tools/smokes/v2/configs/filter_noise_rules.txt"
  grep -vFf "$rules" || cat  # fallback
}
```

**効果**: 85行 → 5行、ルール管理容易、バージョン管理明確

---

## ⚡ 提案5: 環境変数階層削減 (中期)

### 現状 (4層)
```
Layer 1: configs/env/*.env  (profile defaults)
Layer 2: lib/test_runner.sh (mirror + fallbacks)
Layer 3: run_nyash_vm       (inline overrides)
Layer 4: Individual tests   (test-specific)
```

### 提案: 3層 → 2層
```
Layer 1: configs/env/*.env  (すべてのdefault)
Layer 2: Individual tests   (test-specific override)
# test_runner.shはmirror削除→passthrough only
```

**効果**: 設定探索パス半減、優先順位明確

---

## 🌱 提案6: test_runner.sh 分割 (長期)

### 提案構造
```
lib/
├── test_runner.sh       # 主エントリ (require, run_test)
├── env_helpers.sh       # alias_env, mirror (提案1で削減)
├── program_runner.sh    # run_nyash_vm, run_nyash_llvm
├── filters.sh           # filter_noise (提案4で簡略化)
├── assertions.sh        # compare_outputs, check_exact
└── logging.sh           # log_*, test_pass/fail/skip
```

**効果**: 25関数 → 5ファイル×5関数、責務明確

---

## 🌱 提案7: hako.toml活用 (長期)

### 提案: NYASH_MODULES → hako.toml移行
```toml
# hako.toml
[modules]
"selfhost.vm.mir_min" = "selfhost/vm/boxes/mir_vm_min.hako"
"selfhost.vm.handlers" = "selfhost/vm/boxes/op_handlers.hako"
# ...
```

**効果**: ENV変数削減、TomlのIDE補完、型安全

---

## 📊 優先順位まとめ

| 提案 | 効果 | 工数 | 優先度 |
|-----|------|------|--------|
| 1. NYASH_*/HAKO_* 統一 | 13変数削減 | 小 | 🔥 High |
| 2. run_nyash_vm 分割 | 可読性大幅向上 | 小 | 🔥 High |
| 3. NYASH_MODULES 整形 | 可読性向上 | 極小 | 🔥 High |
| 4. filter_noise 設定化 | 保守性向上 | 中 | ⚡ Medium |
| 5. 環境変数階層削減 | 設計明確化 | 中 | ⚡ Medium |
| 6. test_runner 分割 | スケーラビリティ | 大 | 🌱 Low |
| 7. hako.toml 活用 | 根本的簡略化 | 大 | 🌱 Low |

---

## 🚀 即座実施可能な改善 (30分以内)

### Quick Win 1: NYASH_MODULES 整形
```bash
# tools/smokes/v2/configs/env/quick-selfhost.env
# Before: 1行510文字
# After: 8行×60文字
```

### Quick Win 2: run_nyash_vm コメント追加
```bash
# 各責務にコメント追加で可読性向上（リファクタ前準備）
run_nyash_vm() {
  # === 1. Argument handling ===
  local program="$1"; shift
  ...
  # === 2. ASI fixes ===
  if [ "$program" = "-c" ]; then
  ...
}
```

### Quick Win 3: filter_noise 整理
```bash
# 同系統のルールをグループ化（コメント追加）
filter_noise() {
  # === Plugin initialization ===
  grep -v "^\[UnifiedBoxRegistry\]" \
  | grep -v "^\[FileBox\]" \
  ...
  # === Using resolver ===
  | grep -v "^\[using\]" \
  ...
}
```

---

## 💡 推奨アクション

1. **今すぐ** (30分): Quick Win 1-3実施
2. **今週** (2-3時間): 提案1（HAKO_*統一）+ 提案3（NYASH_MODULES整形）
3. **来週** (1日): 提案2（run_nyash_vm分割）
4. **今月** (2-3日): 提案4-5（filter_noise + 階層削減）
5. **来期** (1週間): 提案6-7（ファイル分割 + toml活用）
