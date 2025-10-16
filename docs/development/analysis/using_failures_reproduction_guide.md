# using系失敗 - 再現手順ガイド

**目的**: 各エラーパターンを個別に再現し、修正効果を検証する

---

## パターンA: Parser Error (invalid key)

### 最小再現例

```bash
# テスト実行
SMOKES_DEV_LOG=1 tools/smokes/v2/profiles/quick/core/using_missing_strict_vm.sh

# 期待されるエラー
# "TOML parse error at line 1, column 1"
# "invalid key"
```

### 再現コード

```nyash
# /tmp/test_parser_error.nyash
using "Foo.Bar" as Baz

static box Main {
  main() {
    return 0
  }
}
```

```bash
# 実行
export NYASH_USING_STRICT=1
export NYASH_DISABLE_PLUGINS=1
./target/release/hako /tmp/test_parser_error.nyash 2>&1 | grep "invalid key"

# エラー箇所
# → apps/examples/module_hako_demo/module.hako がTOMLとしてパースされる
```

### デバッグ手順

1. **module候補の列挙確認**:
   ```bash
   NYASH_USING_TRACE=1 ./target/release/hako /tmp/test_parser_error.nyash 2>&1 | grep "ws cand"
   ```
   - 出力: `[using] ws cand: "apps/examples/module_hako_demo/module.hako"`
   - 確認: `.hako` ファイルが候補に含まれている

2. **TOML parse 試行確認**:
   ```bash
   NYASH_USING_TRACE=1 ./target/release/hako /tmp/test_parser_error.nyash 2>&1 | grep "parse TOML error"
   ```
   - 出力: `[using] parse TOML error at "module.hako": TOML parse error`

3. **エラー発生箇所**:
   ```rust
   // src/frontend/using_resolver.rs (推測)
   let candidates = vec![
       "hako_module.toml",
       "module.toml",
       "module.hako",  // ← ここが問題
   ];
   for c in candidates {
       if let Ok(content) = fs::read_to_string(c) {
           toml::from_str(&content)?;  // ← ここでエラー
       }
   }
   ```

### 修正案

```rust
// src/frontend/using_resolver.rs
let candidates = vec![
    "hako_module.toml",
    "module.toml",
    // "module.hako" を除外
];
```

または

```rust
for c in candidates {
    if !c.ends_with(".hako") {  // .hako をスキップ
        if let Ok(content) = fs::read_to_string(c) {
            toml::from_str(&content)?;
        }
    }
}
```

---

## パターンB: Type Error (Void/UnknownBox)

### 最小再現例 (flow_using_alias_vm)

```bash
# テスト実行
SMOKES_DEV_LOG=1 tools/smokes/v2/profiles/quick/core/flow_using_alias_vm.sh

# 期待されるエラー
# "Type error: unsupported binop Add on Integer(20) and Void"
```

### 再現コード

```nyash
# /tmp/flow_test/nyash.toml
[using.flow_utils]
path = "lib/flow_utils/"
main = "utils.nyash"

[using.aliases]
FU = "flow_utils"

[using]
paths = ["lib"]
```

```nyash
# /tmp/flow_test/lib/flow_utils/utils.nyash
box Flow {
  birth() {
    from IntegerBox.birth()
  }

  stringify(x) {
    return "ok"
  }
}
```

```nyash
# /tmp/flow_test/main.nyash
using "flow_utils" as FU
using FU.Flow as FlowBox

static box Main {
  main() {
    local f = new FlowBox()
    local s = f.stringify(10)
    local result = 20 + s  # ← エラー: s が Void
    return result
  }
}
```

```bash
# 実行
cd /tmp/flow_test
export NYASH_ENABLE_FLOW=1
../../hakorune-selfhost/target/release/hako main.nyash 2>&1
```

### デバッグ手順

1. **using解決のトレース**:
   ```bash
   NYASH_USING_TRACE=1 ./target/release/hako main.nyash 2>&1 | grep "resolve\|alias"
   ```
   - 確認: `using "flow_utils"` が解決されているか
   - 確認: `FU.Flow` が正しく解決されているか

2. **型推論のトレース**:
   ```bash
   NYASH_CLI_VERBOSE=1 ./target/release/hako main.nyash 2>&1 | grep "recv_cls_hint\|UnknownBox"
   ```
   - 出力: `recv_cls_hint=UnknownBox`
   - 確認: FlowBox が UnknownBox として扱われている

3. **MIR確認**:
   ```bash
   ./target/release/hako --dump-mir main.nyash 2>&1
   ```
   - 確認: `FlowBox.stringify()` の callee が正しく解決されているか

### 修正案

```rust
// src/frontend/using_resolver.rs
fn resolve_nested_alias(&mut self, base: &str, nested: &[&str]) -> Result<ResolvedBox> {
    let mut current = self.resolve_module(base)?;

    for segment in nested {
        // ネストされたエイリアスを正しく解決
        current = self.resolve_in_module(&current, segment)?;
    }

    Ok(current)
}
```

---

## パターンC: Static Singleton未具現化

### 最小再現例

```bash
# テスト実行
SMOKES_DEV_LOG=1 tools/smokes/v2/profiles/quick/core/namespace_module_first_json_utils_string_vm.sh

# 期待されるエラー
# "Invalid instruction: Method router missing receiver (static singleton not materialized)"
```

### 再現コード

```nyash
# /tmp/static_test/lib/json_native/string_utils.nyash
static box StringUtilsBox {
  size(s) {
    return s.length()
  }
}
```

```nyash
# /tmp/static_test/main.nyash
using json_utils.string as StringUtilsBox

static box Main {
  main() {
    local n = StringUtilsBox.size("hello")  # ← エラー: receiver missing
    print(n)
    return 0
  }
}
```

```bash
# 実行
cd /tmp/static_test
../../hakorune-selfhost/target/release/hako main.nyash 2>&1
```

### デバッグ手順

1. **MIR確認**:
   ```bash
   ./target/release/hako --dump-mir main.nyash 2>&1 | grep "method_call\|StringUtilsBox"
   ```
   - 確認: `method_call` の receiver が設定されているか

2. **VM実行トレース**:
   ```bash
   HAKO_VM_TRACE="op=method_call" ./target/release/hako main.nyash 2>&1
   ```
   - 確認: receiver の値

3. **static box materialization 確認**:
   ```bash
   ./target/release/hako --dump-mir main.nyash 2>&1 | grep "newbox.*StringUtilsBox\|const.*StringUtilsBox"
   ```
   - 期待: static box の singleton allocation 命令
   - 実際: 命令が存在しない

### 修正案

```rust
// src/frontend/mir_builder.rs
fn build_static_box_call(&mut self, box_name: &str, method: &str) -> Result<Reg> {
    // static box の singleton を materialization
    let singleton_reg = self.get_or_create_singleton(box_name)?;

    // method call with receiver
    let result_reg = self.emit_method_call(singleton_reg, method, args)?;
    Ok(result_reg)
}

fn get_or_create_singleton(&mut self, box_name: &str) -> Result<Reg> {
    if let Some(reg) = self.singletons.get(box_name) {
        return Ok(*reg);
    }

    // 初回: singleton を作成
    let reg = self.emit_newbox(box_name, vec![])?;
    self.singletons.insert(box_name.to_string(), reg);
    Ok(reg)
}
```

---

## パターンD-1: 循環依存検出失敗

### 最小再現例

```bash
# テスト実行
SMOKES_DEV_LOG=1 tools/smokes/v2/profiles/quick/core/using_workspace_cycle_strict_fail_vm.sh

# 期待: exit code != 0
# 実際: exit code == 0
```

### 再現コード

```nyash
# /tmp/cycle_test/a.nyash
using "b.Bar" as Bar

static box Foo {
  main() {
    return 0
  }
}
```

```nyash
# /tmp/cycle_test/b.nyash
using "a.Foo" as Foo

static box Bar {
  test() {
    return 0
  }
}
```

```bash
# 実行
cd /tmp/cycle_test
export NYASH_USING_STRICT=1
../../hakorune-selfhost/target/release/hako a.nyash 2>&1
echo "Exit code: $?"  # 期待: != 0, 実際: 0
```

### デバッグ手順

1. **循環依存検出のトレース**:
   ```bash
   NYASH_USING_TRACE=1 ./target/release/hako a.nyash 2>&1 | grep "cycle\|circular"
   ```
   - 確認: 循環依存検出ログがあるか

2. **strict mode 確認**:
   ```bash
   NYASH_USING_TRACE=1 NYASH_USING_STRICT=1 ./target/release/hako a.nyash 2>&1 | grep "strict"
   ```
   - 確認: strict mode が有効化されているか

### 修正案

```rust
// src/frontend/using_resolver.rs
struct UsageChain {
    stack: Vec<String>,
}

impl UsageChain {
    fn push(&mut self, module: String) -> Result<()> {
        if self.stack.contains(&module) {
            // 循環依存検出
            let cycle = self.stack.iter()
                .skip_while(|m| *m != &module)
                .chain(std::iter::once(&module))
                .collect::<Vec<_>>();
            return Err(Error::CircularDependency {
                cycle: cycle.into_iter().map(|s| s.clone()).collect()
            });
        }
        self.stack.push(module);
        Ok(())
    }

    fn pop(&mut self) {
        self.stack.pop();
    }
}
```

---

## パターンD-2: ログ漏出

### 最小再現例

```bash
# テスト実行
tools/smokes/v2/profiles/quick/core/using_modules_alias_timer_static_vm.sh

# 期待出力: "ok"
# 実際出力: "[using/alias] push pair alias=TimerBox canon=...\nok"
```

### デバッグ手順

1. **ログ出力の確認**:
   ```bash
   ./target/release/hako main.nyash 2>&1 | grep "\[using/alias\]"
   ```

2. **ログレベルの確認**:
   ```bash
   NYASH_USING_TRACE=0 ./target/release/hako main.nyash 2>&1 | grep "\[using/alias\]"
   ```
   - 確認: `NYASH_USING_TRACE=0` でもログが出ているか

### 修正案

```rust
// src/frontend/using_resolver.rs
macro_rules! using_log {
    ($($arg:tt)*) => {
        if std::env::var("NYASH_USING_TRACE").unwrap_or_default() == "1" {
            eprintln!($($arg)*);
        }
    };
}

// 使用例
using_log!("[using/alias] push pair alias={} canon={}", alias, canon);
```

---

## 検証チェックリスト

### パターンA修正の検証
- [ ] `module.hako` がTOML候補に含まれない
- [ ] "invalid key" エラーログが出ない
- [ ] 5件のテストが PASS

### パターンB修正の検証
- [ ] workspace module resolution が正常動作
- [ ] nested alias が正しく解決される
- [ ] `UnknownBox` が出現しない
- [ ] 3件のテストが PASS

### パターンC修正の検証
- [ ] static box の singleton が materialization される
- [ ] method_call に receiver が設定される
- [ ] "Method router missing receiver" エラーが出ない
- [ ] 1件のテストが PASS

### パターンD-1修正の検証
- [ ] 循環依存が検出される
- [ ] エラーメッセージが明確
- [ ] exit code != 0
- [ ] 1件のテストが PASS

### パターンD-2修正の検証
- [ ] デバッグログが本番出力に混入しない
- [ ] `NYASH_USING_TRACE=1` でのみログが出る
- [ ] 2件のテストが PASS

---

## 回帰テストコマンド

```bash
# P0修正後
tools/smokes/v2/profiles/quick/core/flow_using_alias_vm.sh
tools/smokes/v2/profiles/quick/core/using_nested_alias_selfhost_common_vm.sh
tools/smokes/v2/profiles/quick/core/using_modules_alias_selfhost_common_string_scan_vm.sh
tools/smokes/v2/profiles/quick/core/namespace_module_first_json_utils_string_vm.sh

# P1修正後
tools/smokes/v2/profiles/quick/core/using_workspace_cycle_strict_fail_vm.sh

# P2修正後
tools/smokes/v2/profiles/quick/core/using_missing_strict_vm.sh
tools/smokes/v2/profiles/quick/core/using_modules_alias_entry_selfhost_vm.sh
tools/smokes/v2/profiles/quick/core/using_auto_dir_namespace_vm.sh
tools/smokes/v2/profiles/quick/core/using_private_strict_vm.sh
tools/smokes/v2/profiles/quick/core/using_modules_alias_hakorune_common_cursor_vm.sh
tools/smokes/v2/profiles/quick/core/using_modules_alias_timer_static_vm.sh

# 全テスト
tools/smokes/v2/run.sh --profile quick
```

---

**作成日**: 2025-10-16
**関連**:
- [using_failures_classification_report.md](using_failures_classification_report.md)
- [using_failures_quick_summary.md](using_failures_quick_summary.md)
- [using_failures_flowchart.md](using_failures_flowchart.md)
