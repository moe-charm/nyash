# Hakorune VM完成計画 - MirCall実装詳細

**Status**: Implementation Plan
**Created**: 2025-10-13
**Purpose**: Phase 1 - Hakorune VM MirCall実装の完全な計画
**Priority**: P0 (最高優先)
**Duration**: 1週間（7日間）

---

## 🎯 概要

### 目標
**Hakorune VMの16命令完全実装を達成する**（現在15/16 = 93% → 100%）

### 現状
- **実装済み命令**: 15/16 (93%)
- **未実装**: MirCall（統一呼び出し命令）
- **行数**: 4,998行（41ファイル）

### 期待される成果
- ✅ 16命令完全実装（100%）
- ✅ Rust VMからの完全独立
- ✅ セルフホスティングの完全実現
- ✅ 509テストすべてPASS維持

---

## 📚 1. MirCallとは

### 1.1 概要
**MirCall**は、Hakoruneのすべての呼び出しを統一する単一の命令です。

**統合される従来の命令**:
- Call (関数呼び出し)
- BoxCall (メソッド呼び出し)
- ExternCall (Extern呼び出し)
- NewBox (Box生成)
- NewClosure (クロージャ生成)
- PluginInvoke (プラグイン呼び出し)

### 1.2 Callee型の定義
**Callee型**は、呼び出し先を表現する列挙型です。

```rust
// Rust VMでの定義（参考）
pub enum Callee {
    // グローバル関数（例: JSON.stringify）
    Global(String),

    // Extern呼び出し（例: "env.console.log"）
    Extern(String),

    // モジュール関数（例: "MyClass.method/2"）
    ModuleFunction(String),

    // メソッド呼び出し
    Method {
        box_name: String,      // レシーバーのBox名
        method: String,        // メソッド名
        receiver: ValueId,     // レシーバーのValueId
        certainty: Certainty,  // 確実性（Certain/Uncertain）
    },

    // コンストラクタ（Box生成）
    Constructor {
        box_type: String,      // Box型名
    },

    // クロージャ
    Closure {
        params: Vec<LocalId>,  // パラメータ
        captures: Vec<ValueId>,// キャプチャー
        me_capture: bool,      // meキャプチャーフラグ
    },

    // 値呼び出し（CallableBox経由）
    Value(ValueId),
}
```

### 1.3 Hakorune VMでの実装方針
**Callee型をHakoruneで実装する**:
- JSON形式で表現（MIR JSONに埋め込み）
- MapBoxで実装（type, name, receiver等のフィールド）
- @matchマクロで分岐（将来的な実装）

**例**:
```json
{
  "op": "mircall",
  "dst": 1,
  "callee": {
    "type": "Global",
    "name": "JSON.stringify"
  },
  "args": [0]
}
```

---

## 📅 2. 実装計画（7日間）

### Day 1: Callee型の設計と基礎実装

#### 2.1.1 CalleeBox作成
**場所**: `selfhost/hakorune-vm/callee_box.hako`

**実装内容**:
```hakorune
// Callee型を表現するBox
static box CalleeBox {
  // Callee型のパース
  parse(callee_json) {
    // JSON → Callee Map
    // type, name, receiver, certainty等を抽出
    local result = new MapBox()

    // typeフィールド取得
    local type = me._extract_field(callee_json, "type")
    result.set("type", type)

    // 各typeに応じたフィールド抽出
    if type == "Global" {
      local name = me._extract_field(callee_json, "name")
      result.set("name", name)
    } else if type == "Extern" {
      local name = me._extract_field(callee_json, "name")
      result.set("name", name)
    } else if type == "ModuleFunction" {
      // ... 同様の実装
    } else if type == "Method" {
      // ... 同様の実装
    } else if type == "Constructor" {
      // ... 同様の実装
    } else if type == "Closure" {
      // ... 同様の実装
    } else if type == "Value" {
      // ... 同様の実装
    }

    return Result.Ok(result)
  }

  // Callee typeの判定
  is_global(callee) { return callee.get("type") == "Global" }
  is_extern(callee) { return callee.get("type") == "Extern" }
  is_module_function(callee) { return callee.get("type") == "ModuleFunction" }
  is_method(callee) { return callee.get("type") == "Method" }
  is_constructor(callee) { return callee.get("type") == "Constructor" }
  is_closure(callee) { return callee.get("type") == "Closure" }
  is_value(callee) { return callee.get("type") == "Value" }

  // フィールド抽出（内部ヘルパー）
  _extract_field(json, field_name) {
    // JSONから指定フィールドを抽出
    // 実装は JsonFieldExtractor を参考に
  }
}
```

**見積もり**: 4時間

#### 2.1.2 レガシー命令のマッピング
**場所**: `selfhost/hakorune-vm/legacy_to_mircall_mapper.hako`

**実装内容**:
```hakorune
// レガシー命令（Call/BoxCall/ExternCall/NewBox）を
// MirCall形式に変換するマッパー
static box LegacyToMirCallMapper {
  // Call → MirCall(Global)
  map_call(call_json) {
    // "func" フィールドを取得
    // Const値の場合、Global calleeに変換
    local func_name = me._extract_func_name(call_json)

    local callee = new MapBox()
    callee.set("type", "Global")
    callee.set("name", func_name)

    return callee
  }

  // BoxCall → MirCall(Method)
  map_boxcall(boxcall_json) {
    // receiver, method, method_id を取得
    local receiver = me._extract_receiver(boxcall_json)
    local method = me._extract_method(boxcall_json)

    local callee = new MapBox()
    callee.set("type", "Method")
    callee.set("method", method)
    callee.set("receiver", receiver)
    callee.set("certainty", "Uncertain")  // デフォルト

    return callee
  }

  // ExternCall → MirCall(Extern)
  map_externcall(externcall_json) {
    // interface, method を結合して "iface.method" を生成
    local iface = me._extract_interface(externcall_json)
    local method = me._extract_method(externcall_json)
    local name = iface + "." + method

    local callee = new MapBox()
    callee.set("type", "Extern")
    callee.set("name", name)

    return callee
  }

  // NewBox → MirCall(Constructor)
  map_newbox(newbox_json) {
    // box_type を取得
    local box_type = me._extract_box_type(newbox_json)

    local callee = new MapBox()
    callee.set("type", "Constructor")
    callee.set("box_type", box_type)

    return callee
  }
}
```

**見積もり**: 4時間

**Day 1完了条件**:
- ✅ CalleeBox実装完了
- ✅ LegacyToMirCallMapper実装完了
- ✅ 基本テスト PASS（Calleeパース、レガシー変換）

---

### Day 2: Callee型の完全実装とテスト

#### 2.2.1 Callee型の全variant実装
**実装内容**:
- Global: ビルトイン関数呼び出し（JSON.stringify等）
- Extern: Extern呼び出し（env.console.log等）
- ModuleFunction: モジュール関数（Class.method/N）
- Method: メソッド呼び出し
- Constructor: Box生成
- Closure: クロージャ呼び出し
- Value: CallableBox経由の呼び出し

#### 2.2.2 テストケース作成
**場所**: `selfhost/hakorune-vm/tests/test_callee_box.hako`

**テスト内容**:
```hakorune
static box TestCalleeBox {
  run_all() {
    me.test_global()
    me.test_extern()
    me.test_module_function()
    me.test_method()
    me.test_constructor()
    me.test_closure()
    me.test_value()
    print("All Callee tests PASS")
  }

  test_global() {
    local json = r#"{"type":"Global","name":"JSON.stringify"}"#
    local callee = CalleeBox.parse(json)
    if callee.is_Err() { print("FAIL: parse global")  return }
    local c = callee.as_Ok()
    if !CalleeBox.is_global(c) { print("FAIL: is_global")  return }
    if c.get("name") != "JSON.stringify" { print("FAIL: name mismatch")  return }
    print("PASS: test_global")
  }

  // ... 他のテストも同様
}
```

**見積もり**: 6-8時間

**Day 2完了条件**:
- ✅ Callee型の全variant実装完了
- ✅ 全テストケース PASS

---

### Day 3-4: MirCallハンドラー実装

#### 2.3.1 MirCallHandlerBox作成
**場所**: `selfhost/hakorune-vm/mircall_handler.hako`

**実装内容**:
```hakorune
using "selfhost/hakorune-vm/callee_box.hako" as CalleeBox
using "selfhost/hakorune-vm/legacy_to_mircall_mapper.hako" as LegacyMapper
using "selfhost/shared/mir/mir_io_box.hako" as MirIoBox
using "selfhost/vm/boxes/result_box.hako" as Result

static box MirCallHandlerBox {
  // MirCall命令の実行
  handle(inst_json, regs, mem) {
    // 1. Calleeの抽出
    local callee_result = me._extract_callee(inst_json)
    if callee_result.is_Err() { return callee_result }
    local callee = callee_result.as_Ok()

    // 2. 引数の抽出
    local args = me._extract_args(inst_json, regs)

    // 3. 宛先レジスタの抽出
    local dst = me._extract_dst(inst_json)

    // 4. Callee typeに応じてディスパッチ
    local result
    if CalleeBox.is_global(callee) {
      result = me._call_global(callee, args, regs, mem)
    } else if CalleeBox.is_extern(callee) {
      result = me._call_extern(callee, args, regs, mem)
    } else if CalleeBox.is_module_function(callee) {
      result = me._call_module_function(callee, args, regs, mem)
    } else if CalleeBox.is_method(callee) {
      result = me._call_method(callee, args, regs, mem)
    } else if CalleeBox.is_constructor(callee) {
      result = me._call_constructor(callee, args, regs, mem)
    } else if CalleeBox.is_closure(callee) {
      result = me._call_closure(callee, args, regs, mem)
    } else if CalleeBox.is_value(callee) {
      result = me._call_value(callee, args, regs, mem)
    } else {
      return Result.Err("unknown callee type")
    }

    // 5. 結果をレジスタに格納
    if result.is_Ok() {
      if dst >= 0 {
        regs.set(StringHelpers.int_to_str(dst), result.as_Ok())
      }
      return Result.Ok(result.as_Ok())
    } else {
      return result
    }
  }

  // Global呼び出し
  _call_global(callee, args, regs, mem) {
    local name = callee.get("name")
    // ビルトイン関数の実装
    // 例: JSON.stringify, Array.join等
    // 既存のGlobalCallHandlerBoxを参考に実装
  }

  // Extern呼び出し
  _call_extern(callee, args, regs, mem) {
    local name = callee.get("name")
    // Extern関数の実装
    // 既存のExternCallHandlerBoxを参考に実装
  }

  // Module Function呼び出し
  _call_module_function(callee, args, regs, mem) {
    local name = callee.get("name")
    // モジュール関数の実装
    // 既存のModuleFunctionCallHandlerBoxを参考に実装
  }

  // Method呼び出し
  _call_method(callee, args, regs, mem) {
    local receiver = callee.get("receiver")
    local method = callee.get("method")
    // メソッド呼び出しの実装
    // 既存のMethodCallHandlerBoxを参考に実装
  }

  // Constructor呼び出し
  _call_constructor(callee, args, regs, mem) {
    local box_type = callee.get("box_type")
    // Box生成の実装
    // 既存のNewBoxHandlerBoxを参考に実装
  }

  // Closure呼び出し
  _call_closure(callee, args, regs, mem) {
    // クロージャ呼び出しの実装
    // 既存のClosureCallHandlerBoxを参考に実装
  }

  // Value呼び出し
  _call_value(callee, args, regs, mem) {
    local value_id = callee.get("value_id")
    // CallableBox経由の呼び出し
  }

  // 内部ヘルパー
  _extract_callee(inst_json) { /* ... */ }
  _extract_args(inst_json, regs) { /* ... */ }
  _extract_dst(inst_json) { /* ... */ }
}
```

**見積もり**: 12-16時間（2日間）

#### 2.3.2 InstructionDispatcherBoxへの統合
**場所**: `selfhost/hakorune-vm/instruction_dispatcher.hako`

**実装内容**:
```hakorune
using "selfhost/hakorune-vm/mircall_handler.hako" as MirCallHandlerBox

static box InstructionDispatcherBox {
  dispatch(inst_json, regs, mem) {
    local op = JsonFieldExtractor.extract_field(inst_json, "op")

    // 既存の命令
    if op == "const" { return ConstHandlerBox.handle(inst_json, regs) }
    else if op == "binop" { return BinOpHandlerBox.handle(inst_json, regs) }
    // ... 他の命令

    // MirCall命令（新規）
    else if op == "mircall" {
      return MirCallHandlerBox.handle(inst_json, regs, mem)
    }

    // レガシー命令（MirCallへマッピング）
    else if op == "call" || op == "boxcall" || op == "externcall" || op == "newbox" {
      // レガシー命令をMirCall形式に変換
      local callee
      if op == "call" {
        callee = LegacyMapper.map_call(inst_json)
      } else if op == "boxcall" {
        callee = LegacyMapper.map_boxcall(inst_json)
      } else if op == "externcall" {
        callee = LegacyMapper.map_extern(inst_json)
      } else if op == "newbox" {
        callee = LegacyMapper.map_newbox(inst_json)
      }

      // MirCall形式のJSONを生成
      local mircall_json = me._build_mircall_json(callee, inst_json)
      return MirCallHandlerBox.handle(mircall_json, regs, mem)
    }

    return Result.Err("unknown op: " + op)
  }

  _build_mircall_json(callee, original_json) {
    // レガシー命令からMirCall JSONを生成
  }
}
```

**見積もり**: 4時間

**Day 3-4完了条件**:
- ✅ MirCallHandlerBox実装完了
- ✅ InstructionDispatcherBox統合完了
- ✅ 基本的なMirCall動作確認

---

### Day 5: テストと検証

#### 2.5.1 既存テストの実行
**実行内容**:
```bash
# Hakorune VM全テスト実行
HAKO_ALLOW_USING_FILE=1 HAKO_USING_PROFILE=dev NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 \
  ./target/release/hako selfhost/hakorune-vm/tests/test_all.hako
```

**確認項目**:
- ✅ 既存の15命令すべて動作
- ✅ 509テストすべてPASS維持

#### 2.5.2 MirCall専用テスト
**場所**: `selfhost/hakorune-vm/tests/test_mircall.hako`

**テスト内容**:
- Global呼び出し（JSON.stringify等）
- Extern呼び出し（env.console.log等）
- ModuleFunction呼び出し
- Method呼び出し
- Constructor呼び出し
- Closure呼び出し
- Value呼び出し

**見積もり**: 6-8時間

**Day 5完了条件**:
- ✅ 既存テストすべてPASS
- ✅ MirCall専用テストすべてPASS
- ✅ パリティテスト（VM/LLVM）PASS

---

### Day 6: トレース機能の実装

#### 2.6.1 トレース機能の拡張
**実装内容**:
- Rust VMと同等のトレース機能
- 環境変数HAKO_VM_TRACEの実装
- レジスタダンプ機能

**場所**: `selfhost/hakorune-vm/trace_box.hako`

```hakorune
static box TraceBox {
  // トレース有効判定
  is_enabled() {
    // HAKO_VM_TRACE環境変数をチェック
    // ※ 環境変数アクセスは Extern("env.getenv") を使用
  }

  // トレース出力
  trace_mircall(callee, args, result) {
    if !me.is_enabled() { return }

    local callee_type = callee.get("type")
    local line = "[hakorune-vm] mircall type=" + callee_type

    if callee_type == "Global" {
      line = line + " name=" + callee.get("name")
    } else if callee_type == "Extern" {
      line = line + " name=" + callee.get("name")
    }
    // ... 他のtypeも同様

    line = line + " args=" + me._format_args(args)
    line = line + " result=" + me._format_result(result)

    print(line)
  }

  _format_args(args) { /* ... */ }
  _format_result(result) { /* ... */ }
}
```

**見積もり**: 4-6時間

**Day 6完了条件**:
- ✅ トレース機能実装完了
- ✅ HAKO_VM_TRACE環境変数動作確認

---

### Day 7: ドキュメント整備とコミット

#### 2.7.1 ドキュメント作成
**作成内容**:
1. **実装ドキュメント**
   - `selfhost/hakorune-vm/docs/MIRCALL_IMPLEMENTATION.md`
   - MirCallの実装詳細
   - 各Callee typeの説明
   - レガシー命令のマッピング

2. **使用例**
   - `selfhost/hakorune-vm/examples/mircall_examples.hako`
   - 各Callee typeの使用例

3. **テストドキュメント**
   - `selfhost/hakorune-vm/tests/README_MIRCALL.md`
   - テストケースの説明

#### 2.7.2 コミット
**コミットメッセージ**:
```
feat(hakorune-vm): implement MirCall instruction - Phase 15.75 complete

MirCall実装により、Hakorune VMの16命令完全実装を達成。
Rust VMからの完全独立を実現。

実装内容:
- CalleeBox: 7種類のCallee variant実装
- MirCallHandlerBox: 統一呼び出しハンドラー
- LegacyMapper: レガシー命令のマッピング
- TraceBox: トレース機能拡張

テスト結果:
- 509テストすべてPASS
- VM/LLVMパリティ維持

これにより、Phase 15.75 (脱Rust大作戦) のPhase 1が完了。
Hakorune VMは16命令完全実装を達成し、Rust VMから完全に独立した。

詳細: docs/development/proposals/phase-15.75/

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**見積もり**: 6-8時間

**Day 7完了条件**:
- ✅ ドキュメント完成
- ✅ コミット完了
- ✅ Phase 15.75 Phase 1完了宣言

---

## 📊 3. 受け入れ条件

### 3.1 必須条件（すべて満たす必要あり）
- ✅ MirCall実装完了（16命令100%実装）
- ✅ 509テストすべてPASS
- ✅ VM/LLVMパリティ維持
- ✅ トレース機能動作
- ✅ パフォーマンス劣化が50%以内

### 3.2 推奨条件
- ✅ ドキュメント整備完了
- ✅ 使用例作成完了
- ✅ コミット完了

---

## ⚠️ 4. リスクと対策

### 4.1 実装の複雑性
**リスク**: MirCallの実装が予想以上に複雑
**対策**:
- 既存のCall/BoxCall/ExternCall実装を参考にする
- 段階的に実装（Day 1: Callee型、Day 3-4: Handler）

### 4.2 レガシー命令のマッピング
**リスク**: レガシー命令との互換性維持が困難
**対策**:
- LegacyMapperで明示的にマッピング
- テストで互換性を徹底確認

### 4.3 パフォーマンス劣化
**リスク**: MirCallのオーバーヘッドでパフォーマンス劣化
**対策**:
- Phase 2 (AOT化) で解決
- ベンチマークで測定

### 4.4 テスト失敗
**リスク**: 509テストのいずれかが失敗
**対策**:
- 既存テストを段階的に実行
- 失敗したらすぐに修正
- Fail-Fast文化の維持

---

## 📈 5. 進捗管理

### 5.1 日次チェックリスト
**Day 1**:
- [ ] CalleeBox実装完了
- [ ] LegacyMapper実装完了
- [ ] 基本テストPASS

**Day 2**:
- [ ] Callee型の全variant実装完了
- [ ] 全テストケースPASS

**Day 3-4**:
- [ ] MirCallHandlerBox実装完了
- [ ] InstructionDispatcher統合完了
- [ ] 基本的なMirCall動作確認

**Day 5**:
- [ ] 既存テストすべてPASS
- [ ] MirCall専用テストすべてPASS
- [ ] パリティテスト（VM/LLVM）PASS

**Day 6**:
- [ ] トレース機能実装完了
- [ ] HAKO_VM_TRACE動作確認

**Day 7**:
- [ ] ドキュメント完成
- [ ] コミット完了
- [ ] Phase 1完了宣言

### 5.2 デイリースタンドアップ
**毎日の確認事項**:
1. 昨日の進捗
2. 今日の予定
3. ブロッカー・リスク
4. サポートが必要な事項

---

## 🎓 6. 成功要因

### 6.1 既存実装の活用
- GlobalCallHandlerBox
- ModuleFunctionCallHandlerBox
- MethodCallHandlerBox
- ExternCallHandlerBox
- NewBoxHandlerBox
- ClosureCallHandlerBox

これらの既存実装をMirCallに統合する。

### 6.2 Fail-Fast文化
- エラーは隠さず即座に失敗
- テスト失敗は即座に修正
- 509テストすべてPASS維持

### 6.3 段階的実装
- Day 1: 基礎
- Day 2: 完全実装
- Day 3-4: ハンドラー
- Day 5: テスト
- Day 6: トレース
- Day 7: ドキュメント

---

## ✅ 7. 完了宣言

**Phase 1完了条件**:
- ✅ 16命令完全実装（100%）
- ✅ 509テストすべてPASS
- ✅ VM/LLVMパリティ維持
- ✅ ドキュメント整備完了
- ✅ コミット完了

**Phase 1完了時の宣言**:
```
🎉 Phase 15.75 Phase 1完了！Hakorune VM 16命令完全実装達成！

実装内容:
- MirCall実装完了
- 16命令完全実装（100%）
- 509テストすべてPASS
- Rust VMからの完全独立

次のステップ:
- Phase 2: Parser/Tokenizerのセルフホスト化（1-2週間）
- Phase 3: Boxes実装のプラグイン化（4-6週間）
```

---

**最終更新**: 2025-10-13
**作成者**: Claude (detailed implementation plan)
**次のアクション**: Day 1開始 - CalleeBox実装
