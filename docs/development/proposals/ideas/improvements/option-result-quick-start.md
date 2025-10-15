# Option/Result クイックスタートガイド

**作成日**: 2025-10-08
**目的**: 今すぐ始められるOption/Result実装の最速ガイド

---

## 🚀 30分で動作確認する方法

### ステップ1: 最小テスト作成（5分）

```bash
# ファイル作成
cat > /home/tomoaki/git/hakorune-selfhost/apps/tests/test_option_minimal.hako << 'EOF'
// test_option_minimal.hako — Option最小テスト
using "apps/lib/boxes/option_std.hako" as Opt

static box Main {
  main() {
    // TC1: Some作成
    local opt1 = Opt.some(42)
    print("TC1_is_some=" + opt1.is_some())
    print("TC1_value=" + opt1.value())

    // TC2: None作成
    local opt2 = Opt.none()
    print("TC2_is_none=" + opt2.is_none())
    print("TC2_unwrap_or=" + opt2.unwrap_or(999))

    print("ALL_TESTS_PASSED")
    return 0
  }
}
EOF
```

### ステップ2: 最小実装作成（10分）

```bash
# ファイル作成
cat > /home/tomoaki/git/hakorune-selfhost/apps/lib/boxes/option_std.hako << 'EOF'
// option_std.hako — Option<T> minimal implementation

box OptionBox {
  _val: Box
  _some: IntegerBox  // 1=some, 0=none

  birth() {
    me._val = null
    me._some = 0
  }

  is_some() {
    return me._some
  }

  is_none() {
    return 1 - me._some
  }

  value() {
    return me._val
  }

  unwrap_or(def) {
    if me._some == 1 {
      return me._val
    }
    return def
  }
}

static box Opt {
  some(v) {
    local opt = new OptionBox()
    opt._val = v
    opt._some = 1
    return opt
  }

  none() {
    local opt = new OptionBox()
    return opt
  }
}
EOF
```

### ステップ3: 動作確認（5分）

```bash
# ビルド（必要なら）
cd /home/tomoaki/git/hakorune-selfhost
cargo build --release

# テスト実行
./target/release/hako apps/tests/test_option_minimal.hako

# 期待出力:
# TC1_is_some=1
# TC1_value=42
# TC2_is_none=1
# TC2_unwrap_or=999
# ALL_TESTS_PASSED
```

### ステップ4: トラブルシューティング（10分）

```bash
# エラーが出た場合の確認手順

# 1. MIR出力確認
./target/release/hako --dump-mir apps/tests/test_option_minimal.hako

# 2. トレースログ確認
export HAKO_VM_TRACE="op=call,boxcall;regs=1"
export NYASH_DISABLE_PLUGINS=1
./target/release/hakorune apps/tests/test_option_minimal.hako 2>&1 | grep -E "\[vm\]|TC"

# 3. 詳細診断
export NYASH_CLI_VERBOSE=1
./target/release/hako apps/tests/test_option_minimal.hako
```

---

## 📋 チェックリスト

### 実装前
- [ ] Phase 15.11成功事例を読んだ
- [ ] 既存のResultBox実装（`selfhost/vm/boxes/result_box.hako`）を確認した
- [ ] Hakorune構文制約を理解している

### 実装中
- [ ] `apps/lib/boxes/option_std.hako` 作成完了
- [ ] `apps/tests/test_option_minimal.hako` 作成完了
- [ ] テスト実行で "ALL_TESTS_PASSED" 表示確認

### 実装後
- [ ] MIR出力が正常（newbox, boxcall, call命令確認）
- [ ] トレースログで内部動作確認済み
- [ ] 次のステップ（完全実装）の計画を立てた

---

## 🎯 次のステップ

### Option完全実装（2-3時間）

**追加メソッド**:
```nyash
// option_std.hako に追加

box OptionBox {
  // 既存メソッド...

  // 高階関数
  map(f) {
    if me._some == 1 {
      local new_val = f(me._val)
      return Opt.some(new_val)
    }
    return Opt.none()
  }

  and_then(f) {
    if me._some == 1 {
      return f(me._val)
    }
    return Opt.none()
  }

  filter(pred) {
    if me._some == 1 {
      if pred(me._val) {
        return Opt.some(me._val)
      }
    }
    return Opt.none()
  }

  // Result相互変換
  ok_or(err_msg) {
    if me._some == 1 {
      return Res.ok(me._val)
    }
    return Res.err(err_msg)
  }
}
```

### Result拡張実装（2-3時間）

**ファイル**: `apps/lib/boxes/result_std.hako`

**追加メソッド**:
```nyash
box ResultBox {
  // 既存: _val, _err, _ok (selfhost/vm/boxes/result_box.hako から移植)

  // 高階関数
  map(f) {
    if me._ok == 1 {
      local new_val = f(me._val)
      return Res.ok(new_val)
    }
    return Res.err(me._err)
  }

  map_err(f) {
    if me._ok == 0 {
      local new_err = f(me._err)
      return Res.err(new_err)
    }
    return Res.ok(me._val)
  }

  and_then(f) {
    if me._ok == 1 {
      return f(me._val)
    }
    return Res.err(me._err)
  }

  or_else(f) {
    if me._ok == 0 {
      return f(me._err)
    }
    return Res.ok(me._val)
  }

  // Option相互変換
  ok() {
    if me._ok == 1 {
      return Opt.some(me._val)
    }
    return Opt.none()
  }
}
```

### スモークテスト統合（1時間）

**ファイル1**: `tools/smokes/v2/profiles/quick/core/option_basic_vm.sh`
```bash
#!/bin/bash
# option_basic_vm.sh — Option<T> basic operations test

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/option_basic_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.hako"

cat > "$SRC" << 'EOF'
using "apps/lib/boxes/option_std.hako" as Opt

static box Main {
  main() {
    local opt1 = Opt.some(42)
    print("is_some=" + opt1.is_some())
    print("value=" + opt1.value())

    local opt2 = Opt.none()
    print("is_none=" + opt2.is_none())
    print("unwrap_or=" + opt2.unwrap_or(999))

    return 0
  }
}
EOF

out=$(run_nyash_vm "$SRC")
want=$(cat << 'E'
is_some=1
value=42
is_none=1
unwrap_or=999
E
)

compare_outputs "$want" "$out" "option_basic_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
```

**ファイル2**: `tools/smokes/v2/profiles/quick/core/result_basic_vm.sh`
```bash
#!/bin/bash
# result_basic_vm.sh — Result<T,E> basic operations test

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/result_basic_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.hako"

cat > "$SRC" << 'EOF'
using "apps/lib/boxes/result_std.hako" as Res

static box Main {
  main() {
    local res1 = Res.ok(42)
    print("is_ok=" + res1.is_ok())
    print("value=" + res1.value())

    local res2 = Res.err("error")
    print("is_err=" + (1 - res2.is_ok()))
    print("unwrap_or=" + res2.unwrap_or(999))

    return 0
  }
}
EOF

out=$(run_nyash_vm "$SRC")
want=$(cat << 'E'
is_ok=1
value=42
is_err=1
unwrap_or=999
E
)

compare_outputs "$want" "$out" "result_basic_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
```

**実行**:
```bash
# 個別実行
bash tools/smokes/v2/profiles/quick/core/option_basic_vm.sh
bash tools/smokes/v2/profiles/quick/core/result_basic_vm.sh

# 全体実行
tools/smokes/v2/run.sh --profile quick
```

---

## 🚨 よくある失敗と対策

### 失敗1: using文でパースエラー

**症状**:
```
Error: Expected identifier, found "."
```

**原因**:
- `using "apps/lib/boxes/option_std.hako"` の形式が間違っている

**対策**:
```bash
# hako.toml に追加
[modules]
"std.option" = "apps/lib/boxes/option_std.hako"

# コード内
using "std.option" as Opt
```

### 失敗2: セミコロン区切りエラー

**症状**:
```
Error: Unexpected token ';'
```

**原因**:
- Hakoruneは1文に複数文をセミコロン区切りで書けない

**対策**:
```nyash
// ❌ NG: セミコロン区切り
me._val = null  me._some = 0

// ✅ OK: 複数行
me._val = null
me._some = 0
```

### 失敗3: lambda構文エラー

**症状**:
```
Error: Unexpected token 'fn'
```

**原因**:
- lambda構文（`fn(x) { x * 2 }`）がまだ未実装の可能性

**対策**:
```nyash
// ❌ NG: lambda構文（未実装なら）
opt.map(fn(x) { x * 2 })

// ✅ OK: 通常の関数
static box Helpers {
  double(x) { return x * 2 }
}
opt.map(Helpers.double)
```

---

## 📊 見積もり精度の改善

### Phase 2.1 の失敗（見積もり18%）
- **見積もり**: 108-150行削減
- **実際**: 20行削減のみ
- **原因**: 構文制約を考慮せず

### Option/Result の現実的見積もり

| 作業 | 楽観的 | 現実的 | 悲観的 |
|------|--------|--------|--------|
| 最小実装 | 30分 | 1時間 | 2時間 |
| 完全実装 | 2時間 | 4時間 | 6時間 |
| スモークテスト | 1時間 | 2時間 | 3時間 |
| 合計 | 3.5時間 | 7時間 | 11時間 |

**推奨**: 現実的見積もり（7時間）を基準にする

---

## 📖 参考資料

- **完全戦略書**: `docs/development/proposals/ideas/improvements/option-result-test-strategy.md`
- **Phase 15.11成功事例**: `apps/selfhost/test_string_helpers.hako`
- **既存ResultBox**: `selfhost/vm/boxes/result_box.hako`
- **スモークテスト例**: `tools/smokes/v2/profiles/quick/core/wasm_std_array_resize_vm.sh`

---

**次のアクション**: 最小テスト作成（ステップ1）から開始してください。30分で動作確認できます！
