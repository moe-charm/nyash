# Option/Result モジュール化・配置戦略（Box-First原則）

**作成日**: 2025-10-08
**目的**: apps/lib/boxes/ への配置、using経路、依存関係の完全設計
**Box-First原則**: 設定・状態・橋渡しはBox化し、境界を明確にして戻せる足場を積む

---

## 📊 現状分析

### ✅ 既存の Result 実装

**場所**: `/home/tomoaki/git/hakorune-selfhost/selfhost/vm/boxes/result_box.hako`

```nyash
// result_box.hako — Result / ResultBox
// 責務: 処理結果の統一表現（成功値 or エラーメッセージ）
// 使い方: Result.ok(val) / Result.err(msg) → ResultBox

box ResultBox {
  _val: Box
  _err: StringBox
  _ok: IntegerBox  // 1=ok, 0=err（BoolBox 非依存）

  birth() { me._val = null  me._err = ""  me._ok = 0 }

  is_ok() { return me._ok }
  value() { return me._val }
  error() { return me._err }

  unwrap_or(def) { if me._ok == 1 { return me._val } return def }
}

static box Result {
  ok(v) {
    local r = new ResultBox()
    r._val = v
    r._ok = 1
    return r
  }
  err(msg) {
    local r = new ResultBox()
    r._err = msg
    r._ok = 0
    return r
  }
}
```

**現在の使用箇所** (3ファイル):
- `selfhost/vm/boxes/phi_decode_box.hako` - PHI命令デコード結果
- `selfhost/vm/boxes/mir_vm_min.hako` - Mini-VM内部
- `apps/hakorune/vm/boxes/hakorune_vm_min.hako` - Hakorune VM内部

**現在の using パターン**:
```nyash
using "selfhost/vm/boxes/result_box.hako" as Result
```

### ❌ Option 実装は存在しない

Option型は現在実装されていない。null チェックを直接コードで行っている。

---

## 🎯 設計方針（Box-First原則）

### 1️⃣ ファイル配置戦略

#### 📁 推奨配置: `apps/lib/boxes/`

**理由**:
1. **既存パターンとの統一**: `array_std.hako`, `console_std.hako`, `map_std.hako`, `string_std.hako` と同じディレクトリ
2. **標準ライブラリ位置**: 汎用的な基礎型は `apps/lib/` に集約
3. **using パス簡略化**: `apps/lib/boxes/` は `hako.toml` の `[using] paths = ["apps", "lib", "."]` でカバー済み
4. **循環依存回避**: VM実装から独立した位置（VM→lib は OK、lib→VM は NG）

#### 🚫 移動が必要な理由

**現在の場所**: `selfhost/vm/boxes/result_box.hako`

**問題点**:
- VM内部の実装詳細に見える（実際は汎用型）
- Mini-VM以外での再利用が難しい（using パスが長い）
- selfhost専用に見えてしまう（Hakoruneでも使いたい）

#### ✅ 提案ファイル構成

```
apps/lib/boxes/
├── array_std.hako      # 既存
├── console_std.hako    # 既存
├── map_std.hako        # 既存
├── string_std.hako     # 既存
├── option.hako         # 新規 (Option<T> 型)
└── result.hako         # 移動 (Result<T,E> 型)
```

**命名規則**:
- `_std.hako` サフィックス: 既存Boxのヘルパー関数集（ArrayStd, ConsoleStd等）
- サフィックスなし: 独立した型定義（Option, Result）

---

### 2️⃣ using 経路設計

#### ✅ 推奨パターン（Phase 15.7 準拠）

**基本形（相対パス、dev 推奨）**:
```nyash
using "apps/lib/boxes/option.hako" as Option
using "apps/lib/boxes/result.hako" as Result
```

**package形式（prod 推奨）**:
```toml
# hako.toml
[using.std_types]
path = "apps/lib/boxes/"

[using.aliases]
Option = "std_types/option.hako"
Result = "std_types/result.hako"
```

```nyash
using Option as Opt
using Result as Res
```

#### 🔍 パス解決順序（Phase 15.7）

1. **呼び出し元ファイルのディレクトリ**
2. **`$NYASH_ROOT`**
3. **実行バイナリからのプロジェクトルート推定** (`target/release/hako` の3階層上)
4. **`hako.toml` の `[using.paths]`** (`["apps", "lib", "."]`)

**実例**:
```nyash
// selfhost/vm/boxes/phi_decode_box.hako から
using "apps/lib/boxes/result.hako" as Result
// ↓ 解決順序
// 1. selfhost/vm/boxes/apps/lib/boxes/result.hako (存在しない)
// 2. $NYASH_ROOT/apps/lib/boxes/result.hako (成功!)
```

#### 🚨 Fail-Fast: 循環参照対策

**禁止パターン**:
```nyash
// ❌ result.hako の中で
using "apps/lib/boxes/option.hako" as Option

// ❌ option.hako の中で
using "apps/lib/boxes/result.hako" as Result
```

**対策**: Option/Result は完全独立（相互依存なし）

---

### 3️⃣ 依存関係管理

#### ✅ 依存グラフ（一方向のみ）

```
Core Boxes (StringBox, IntegerBox, BoolBox, ArrayBox)
    ↑
    |
Option.hako / Result.hako  (Core Boxesのみ依存)
    ↑
    |
Mini-VM / JsonCursorBox / その他のアプリケーション
```

#### 📦 Option.hako の依存

**許可**:
- なし（null, 整数比較のみ使用、BoolBoxも不使用）

**実装方針**:
```nyash
// option.hako
box OptionBox {
  _value: Box
  _has: IntegerBox  // 1=Some, 0=None (BoolBox非依存)

  birth() { me._value = null  me._has = 0 }
  is_some() { return me._has }
  is_none() { return me._has == 0 ? 1 : 0 }
  unwrap() { return me._value }
  unwrap_or(def) { if me._has == 1 { return me._value } return def }
}

static box Option {
  some(v) {
    local opt = new OptionBox()
    opt._value = v
    opt._has = 1
    return opt
  }
  none() {
    local opt = new OptionBox()
    return opt
  }
}
```

#### 📦 Result.hako の依存

**許可**:
- `StringBox` (エラーメッセージ格納用)
- `IntegerBox` (状態フラグ用)

**現在の実装は維持**（既に依存最小化されている）

---

### 4️⃣ 統一ネームスペース戦略

#### ✅ 採用パターン: static box ファクトリ

**理由**:
1. **既存パターンとの一貫性**: `Result.ok()`, `Result.err()` は既存実装
2. **型安全**: コンストラクタを static box で集約
3. **名前空間明確化**: `Option.some()` vs 単なる `some()` グローバル関数

**実装例**:
```nyash
// ❌ グローバル関数パターン（非推奨）
function some(v) { ... }
function none() { ... }

// ✅ static box パターン（推奨）
static box Option {
  some(v) { ... }
  none() { ... }
}
```

#### 🚫 名前空間衝突回避

**既存の ResultBox との関係**:

**Rust側**（参考）:
- `box_trait::ResultBox` (legacy)
- `boxes::result::NyashResultBox` (新型)
- エイリアス: `pub type ResultBox = NyashResultBox;`

**Hakorune側**（本提案）:
- `ResultBox` (instance box) - エラー情報を保持
- `Result` (static box) - ファクトリメソッド (`.ok()`, `.err()`)

**衝突なし**: Rust側とHakorune側は別名前空間

---

### 5️⃣ Mini-VMでの使用例

#### Phase 1（基盤構築）での使用サンプル

**JsonCursorBox統合時のResult<T,E>活用例**:

```nyash
// selfhost/shared/json/json_cursor.hako
using "apps/lib/boxes/result.hako" as Result

static box JsonCursorBox {
  // Before (null返却)
  seek_array_end_old(text, lbracket_pos) {
    // ...
    if error { return -1 }  // エラーを-1で表現（曖昧）
    return end_pos
  }

  // After (Result<T,E>)
  seek_array_end(text, lbracket_pos) {
    // ...
    if error {
      return Result.err("seek_array_end: unmatched bracket at pos " + pos)
    }
    return Result.ok(end_pos)
  }
}
```

**呼び出し側**:
```nyash
// selfhost/vm/boxes/phi_decode_box.hako
local result = JsonCursorBox.seek_array_end(seg, arr_br)
if result.is_ok() == 0 {
  me._tprint("[ERROR] " + result.error())
  return Result.err("phi:scan-failed")
}
local endp = result.value()
```

#### InstructionDispatcherでのOption<T>活用例

```nyash
// selfhost/vm/boxes/instruction_scanner.hako
using "apps/lib/boxes/option.hako" as Option

static box InstructionScannerBox {
  // Before (null返却)
  find_next_inst_old(mjson, pos) {
    // ...
    if not_found { return null }
    return inst_pos
  }

  // After (Option<T>)
  find_next_inst(mjson, pos) {
    // ...
    if not_found { return Option.none() }
    return Option.some(inst_pos)
  }
}
```

**呼び出し側**:
```nyash
local maybe_inst = InstructionScannerBox.find_next_inst(mjson, i)
if maybe_inst.is_some() == 1 {
  local inst_pos = maybe_inst.unwrap()
  // ... 処理
} else {
  // not found - early return
  return 0
}
```

---

## 🚀 移行計画（80/20ルール適用）

### Phase 1: 基盤構築（80%で完了）

#### Step 1.1: Option.hako 新規作成
- [ ] `apps/lib/boxes/option.hako` を作成
- [ ] `OptionBox` (instance) + `Option` (static box factory) 実装
- [ ] メソッド: `some()`, `none()`, `is_some()`, `is_none()`, `unwrap()`, `unwrap_or()`

#### Step 1.2: Result.hako 移動
- [ ] `selfhost/vm/boxes/result_box.hako` → `apps/lib/boxes/result.hako` に移動
- [ ] 既存の3ファイルの using パス更新
  - `selfhost/vm/boxes/phi_decode_box.hako`
  - `selfhost/vm/boxes/mir_vm_min.hako`
  - `apps/hakorune/vm/boxes/hakorune_vm_min.hako`

#### Step 1.3: スモークテスト追加
- [ ] `tools/smokes/v2/profiles/quick/core/option_basic_vm.sh`
- [ ] `tools/smokes/v2/profiles/quick/core/result_basic_vm.sh`

**テストケース例**:
```nyash
// apps/tests/core/option_basic.hako
using "apps/lib/boxes/option.hako" as Option

static box Main {
  main(args) {
    local some_val = Option.some(42)
    if some_val.is_some() == 1 {
      print(some_val.unwrap())  // 42
    }

    local none_val = Option.none()
    print(none_val.unwrap_or(999))  // 999

    return 0
  }
}
```

### Phase 2: 段階的適用（残り20%は必要に応じて）

**候補箇所**（優先順位順）:

1. **JsonCursorBox** - seek系メソッドのエラー返却改善
2. **PhiDecodeBox** - 既に Result 使用中（パス更新のみ）
3. **InstructionScannerBox** - 命令探索の Option 化
4. **JsonFragBox** - get系メソッドの Option 化（`get_int()` → `Option<i64>`）

**方針**:
- ✅ 新規コードは積極的に Option/Result 使用
- ⚠️ 既存コードの書き換えは慎重に（動作確認必須）
- 📝 失敗・問題点は必ず記録

---

## 📋 hako.toml 設定例

### 最小設定（Phase 1）

```toml
# hako.toml
[using]
paths = ["apps", "lib", "."]

# 追加設定不要 - paths で apps/lib/boxes/ は解決可能
```

### 推奨設定（Phase 2: 本番環境）

```toml
# hako.toml
[using]
paths = ["apps", "lib", "."]

[using.std_types]
path = "apps/lib/boxes/"
# main 不要（個別ファイル指定）

[using.aliases]
Option = "std_types/option.hako"
Result = "std_types/result.hako"

# または module 形式
[modules]
std.option = "apps/lib/boxes/option.hako"
std.result = "apps/lib/boxes/result.hako"
```

**使用例**:
```nyash
// dev モード（相対パス）
using "apps/lib/boxes/option.hako" as Option

// prod モード（module）
using std.option as Option
```

---

## 🧪 テスト戦略

### 単体テスト

**場所**: `apps/tests/core/`

1. **option_basic.hako** - Some/None の基本動作
2. **option_unwrap.hako** - unwrap/unwrap_or の境界テスト
3. **result_basic.hako** - ok/err の基本動作
4. **result_unwrap.hako** - 成功時/失敗時の値取得

### 統合テスト

**場所**: `apps/tests/integration/`

1. **json_cursor_result.hako** - JsonCursorBox + Result 統合
2. **scanner_option.hako** - InstructionScannerBox + Option 統合
3. **phi_decode_result.hako** - PhiDecodeBox の Result 使用（既存）

### スモークテスト

**場所**: `tools/smokes/v2/profiles/quick/core/`

```bash
# option_basic_vm.sh
#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../../../lib/test_runner.sh"

TEST_FILE="apps/tests/core/option_basic.hako"
EXPECTED_OUTPUT="42
999"
BACKEND="vm"

run_test "$TEST_FILE" "$EXPECTED_OUTPUT" "$BACKEND"
```

---

## ⚠️ 既知の問題・制約

### 1. BoolBox 非依存設計

**理由**: 循環依存回避（BoolBox自体がOption/Resultを使う可能性）

**対策**: IntegerBox で代用
- `1` = true / Some / Ok
- `0` = false / None / Err

### 2. ジェネリクス未サポート

**現状**: `Option<T>` / `Result<T,E>` の型パラメータは実行時のみ（静的型チェックなし）

**影響**:
```nyash
local opt = Option.some(42)
opt._value  // Box型として格納（実行時に整数と判明）
```

**対策**: ドキュメントでの型注釈推奨
```nyash
// Option<IntegerBox>
local opt_int = Option.some(42)

// Option<StringBox>
local opt_str = Option.some("hello")
```

### 3. --dump-mir フラグの制約

**問題**: `using` 文を含むファイルは `--dump-mir` でパースエラー

**回避策**:
```bash
# ❌ 失敗
./hako --dump-mir apps/tests/core/option_basic.hako

# ✅ 成功（JSON出力）
./hako --emit-mir-json /tmp/mir.json apps/tests/core/option_basic.hako
cat /tmp/mir.json | jq .
```

### 4. Mini-VM での制約

**制約事項**:
- Option/Result は**Hakoruneコード内**でのみ使用
- MIR命令セットには影響なし（既存16命令のまま）
- VM実行時は通常のBoxCall命令で処理

---

## 🎓 学び・ベストプラクティス

### ✅ 成功パターン

1. **static box ファクトリ**: `Option.some()`, `Result.ok()` で統一
2. **IntegerBox 状態管理**: BoolBox非依存で循環参照回避
3. **apps/lib/boxes/ 集約**: 標準ライブラリの明確な配置
4. **using パス統一**: `apps/lib/boxes/` を基準とした相対パス

### 🚫 避けるべきパターン

1. **グローバル関数での実装**: 名前空間汚染
2. **Option ⇔ Result の相互依存**: 循環参照
3. **VM内部への配置**: 汎用型は lib/ へ
4. **BoolBox への依存**: 循環依存の原因

### 📝 設計原則（Box-First）

1. **境界を作る**: Option/Result は Core Boxes のみに依存
2. **戻せる**: 既存コードへの影響最小（段階的移行可能）
3. **見える化**: スモークテストで動作保証
4. **Fail-Fast**: null返却より明示的エラー（Result.err）

---

## 📚 関連ドキュメント

### 設計・アーキテクチャ
- [using system](../../../reference/language/using.md) - 名前空間・モジュール解決
- [Box Factory設計](../../../reference/architecture/box-factory-design.md) - Box生成戦略
- [MIR Callee革新](../../../architecture/mir-callee-revolution.md) - 関数呼び出し型安全化

### 実装ガイド
- [構文早見表](../../../quick-reference/syntax-cheatsheet.md) - 基本構文
- [完全言語リファレンス](../../../reference/language/LANGUAGE_REFERENCE_2025.md) - 全仕様
- [開発プラクティス](../../../guides/development-practices.md) - 開発方針

### 移行関連
- [RESULTBOX_MIGRATION_TODO.md](../../current/RESULTBOX_MIGRATION_TODO.md) - Rust側Result移行計画

---

## 🚨 重要: 失敗報告の記録

**成功より失敗が重要**: このドキュメントは80%の設計を記述しています。実装時の失敗・問題点は以下に記録してください。

### 失敗報告フォーマット

```markdown
## ❌ Phase X 実装時の問題点

### 1️⃣ [失敗の種類]
**問題**: [何が起きたか]
**期待**: [何を期待していたか]
**実際**: [実際にどうなったか]
**原因**: [なぜ失敗したか]
**影響**: [どのくらい深刻か]
**学び**: [次回どう避けるか]
```

**記録先**:
- このファイルの末尾に追記
- または `docs/development/proposals/ideas/improvements/option-result-migration-report.md` に分離

---

## 📝 実装チェックリスト

### Phase 1: 基盤構築
- [ ] `apps/lib/boxes/option.hako` 作成
- [ ] `apps/lib/boxes/result.hako` 移動（元ファイル削除）
- [ ] 既存3ファイルの using パス更新
- [ ] スモークテスト2本追加 (option_basic, result_basic)
- [ ] スモークテスト実行・PASS確認
- [ ] git commit（「feat: Option/Result型をapps/lib/boxes/に集約」）

### Phase 2: 段階的適用（オプション）
- [ ] JsonCursorBox の seek系メソッド Result 化
- [ ] InstructionScannerBox の find系メソッド Option 化
- [ ] JsonFragBox の get系メソッド Option 化
- [ ] 統合テスト追加
- [ ] 動作確認・問題点記録

### Phase 3: ドキュメント整備（オプション）
- [ ] ユーザーガイドに Option/Result の使い方追記
- [ ] API リファレンス追加
- [ ] ベストプラクティス集更新

---

## 🎯 成果物

このドキュメントにより、以下が明確になりました：

1. ✅ **ファイル配置**: `apps/lib/boxes/option.hako`, `apps/lib/boxes/result.hako`
2. ✅ **using 経路**: 相対パス `apps/lib/boxes/` を基準、prod では module 形式推奨
3. ✅ **依存関係**: Core Boxesのみ依存、相互依存なし、BoolBox非依存
4. ✅ **ネームスペース**: static box ファクトリパターン統一
5. ✅ **使用例**: JsonCursorBox (Result), InstructionScannerBox (Option) での活用例

**Box-First原則の適用**:
- 境界明確化: Option/Result は独立した型
- 戻せる設計: 段階的移行可能
- Fail-Fast: null より明示的エラー型

---

**最終更新**: 2025-10-08
**次のアクション**: Phase 1 実装開始（option.hako作成 → result.hako移動 → スモークテスト）
