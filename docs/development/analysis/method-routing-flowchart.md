# メソッド降下フローチャート (Visual)

**目的**: `s.substring(j, j+1)` の降下経路を視覚的に理解する

---

## 🎨 完全フローチャート (Mermaid)

```mermaid
flowchart TD
    Start([Method Call: recv.method args])

    Start --> Phase1{Phase 1: Early Routing}

    %% Phase 1: handle_standard_method_call
    Phase1 --> Birth{method == birth?}
    Birth -->|YES| LegacyCall[emit_legacy_call<br/>ModuleFunction]
    LegacyCall --> End1([✅ 終了])

    Birth -->|NO| HasOrigin{origin_get recv<br/>存在?}

    %% Origin が存在する場合
    HasOrigin -->|YES| TryTable1[try_lower_via_table<br/>origin_cls, method, arity]
    TryTable1 --> TableMatch1{lowering table<br/>マッチ?}
    TableMatch1 -->|YES| Extern1[Extern nyrt.string.substring]
    Extern1 --> End2([✅ 終了 - 最速])

    TableMatch1 -->|NO| CoreBox{cls in<br/>Array/Map/String?}
    CoreBox -->|YES| MethodId{resolve_builtin<br/>method_id?}
    MethodId -->|Some id| BoxCall1[BoxCall<br/>method_id]
    BoxCall1 --> End3([✅ 終了])
    MethodId -->|None| Fallback1[Phase 2へ]

    CoreBox -->|NO| Fallback1

    %% Origin が存在しない場合
    HasOrigin -->|NO| InferString{value_types<br/>== String?}
    InferString -->|YES| TryTable2[try_lower_via_table<br/>StringBox, method, arity]
    TryTable2 --> TableMatch2{table<br/>マッチ?}
    TableMatch2 -->|YES| Extern2[Extern nyrt.string.substring]
    Extern2 --> End4([✅ 終了])

    TableMatch2 -->|NO| LengthCheck
    InferString -->|NO| LengthCheck{method in<br/>length/len<br/>arity==0?}

    LengthCheck -->|YES| TryTable3[try_lower_via_table<br/>StringBox, method, arity]
    TryTable3 --> TableMatch3{table<br/>マッチ?}
    TableMatch3 -->|YES| Extern3[Extern nyrt.string.length]
    Extern3 --> End5([✅ 終了])

    TableMatch3 -->|NO| Fallback2[Phase 2へ]
    LengthCheck -->|NO| Fallback2

    %% Phase 2: emit_unified_call
    Fallback1 --> Phase2{Phase 2: Unified Routing}
    Fallback2 --> Phase2

    Phase2 --> Infer[infer_receiver<br/>box_hint, method, recv]
    Infer --> InferResult[class_name<br/>certainty]

    InferResult --> EarlyRewrite{Early Rewrite?}
    EarlyRewrite --> StrLike[try_early_str_like]
    StrLike --> Equals[try_special_equals]
    Equals --> KnownUnique[try_known_or_unique]

    KnownUnique --> ShouldRewrite{should_rewrite?}
    ShouldRewrite -->|cls==StringBox<br/>method==substring| Skip[❌ rewrite しない]
    ShouldRewrite -->|その他| Rewrite[✅ ModuleFunction]
    Rewrite --> End6([✅ 終了])

    Skip --> Convert[convert_target_to_callee]
    Convert --> CalleeMethod[Callee::Method<br/>box_name, method, recv]

    CalleeMethod --> Router[choose_route<br/>box_name, method, certainty]

    Router --> RouteUnknown{box_name<br/>== UnknownBox?}
    RouteUnknown -->|YES| RouteBoxCall[Route::BoxCall]
    RouteBoxCall --> EmitBox[emit_box_or_plugin_call]
    EmitBox --> BoxCall2[BoxCall instruction]
    BoxCall2 --> VMError([❌ 実行時エラー<br/>method_id なし])

    RouteUnknown -->|NO| RouteCore{is_core_box<br/>&&<br/>NOT length?}
    RouteCore -->|YES| RouteBoxCall

    RouteCore -->|NO| RouteUnified[Route::Unified]
    RouteUnified --> Normalize[apply_all<br/>normalize]

    Normalize --> NormString{normalize_string?}
    NormString -->|length/size/len| ExternNorm[Extern nyrt.string.length]
    ExternNorm --> EmitCall

    NormString -->|substring/indexOf| NoNorm[❌ normalize 対象外]
    NoNorm --> StayMethod[Callee::Method のまま]

    StayMethod --> EmitCall[emit_instruction<br/>Call callee]
    EmitCall --> VMResolve([VM で解決<br/>不安定])

    style Extern1 fill:#90EE90
    style Extern2 fill:#90EE90
    style Extern3 fill:#90EE90
    style ExternNorm fill:#90EE90
    style BoxCall1 fill:#FFD700
    style BoxCall2 fill:#FF6B6B
    style VMError fill:#FF0000,color:#FFF
    style VMResolve fill:#FFA500
```

---

## 🔍 substring 専用の簡略版

```mermaid
flowchart TD
    Start([s.substring j, j+1])

    Start --> Origin{origin_get s<br/>== StringBox?}

    Origin -->|YES ✅| Table1[try_lower_via_table]
    Table1 --> Match1{substring/2<br/>マッチ?}
    Match1 -->|YES| Fast[✅ Extern nyrt.string.substring<br/>最速・安定]

    Origin -->|NO| ValueType{value_types s<br/>== String?}
    ValueType -->|YES ✅| Table2[try_lower_via_table]
    Table2 --> Match2{substring/2<br/>マッチ?}
    Match2 -->|YES| Inferred[✅ Extern nyrt.string.substring<br/>安定]

    ValueType -->|NO| Unified[emit_unified_call]
    Unified --> InferRecv[infer_receiver]

    InferRecv --> Unknown{result<br/>== UnknownBox?}
    Unknown -->|YES ❌| BoxCallBad[❌ BoxCall<br/>実行時エラー]

    Unknown -->|NO| String{result<br/>== StringBox?}
    String -->|YES| Method[Callee::Method<br/>StringBox.substring]
    Method --> Router[choose_route]
    Router --> RouteCore{is_core_box?}
    RouteCore -->|YES| BoxCallLegacy[BoxCall<br/>legacy 優先]
    BoxCallLegacy --> VMFail([❌ method_id なし])

    RouteCore -->|NO| RouteU[Route::Unified]
    RouteU --> Norm[normalize]
    Norm --> NoChange[❌ 対象外<br/>Method のまま]
    NoChange --> VMDepend([⚠️ VM 依存<br/>不安定])

    Match1 -->|NO| NextCheck1[次の条件へ...]
    Match2 -->|NO| NextCheck2[次の条件へ...]

    style Fast fill:#00FF00,color:#000
    style Inferred fill:#90EE90,color:#000
    style BoxCallBad fill:#FF0000,color:#FFF
    style VMFail fill:#FF0000,color:#FFF
    style VMDepend fill:#FFA500,color:#000
```

---

## 📊 3つの経路比較

| 経路 | 条件 | 結果 | 速度 | 安定性 |
|------|------|------|------|--------|
| **Early Table** | origin または value_types が StringBox | `Extern("nyrt.string.substring")` | ⚡ 最速 | ✅ 完全安定 |
| **BoxCall Legacy** | origin が StringBox + method_id 存在 | `BoxCall(method_id)` | 🐢 中速 | ⚠️ 安定 (但し substring は method_id なし) |
| **Unified → VM** | origin 不明 + 推論失敗 | `BoxCall` or `Method` → VM 解決 | 🐌 最遅 | ❌ 不安定 (実行時エラーあり) |

---

## 🎯 安定化の鍵

### ✅ 安定化する条件
1. `origin_register(s, "StringBox")` が呼ばれている
2. `value_types[s] = MirType::String` が設定されている
3. `value_types[s] = MirType::Box("StringBox")` が設定されている

### ❌ 不安定になる条件
1. 引数として受け取った値 (origin なし)
2. 他のメソッドの戻り値 (origin 伝播なし)
3. 複雑な式の結果 (型推論失敗)

---

## 🔧 修正方針の視覚的比較

```mermaid
flowchart LR
    Problem([不安定降下])

    Problem --> Opt1[Option 1:<br/>Origin Propagation]
    Problem --> Opt2[Option 2:<br/>value_types Propagation]
    Problem --> Opt3[Option 3:<br/>Normalize 拡張]
    Problem --> Opt4[Option 4:<br/>Fallback Heuristic]

    Opt1 --> Best[✅ 根本解決<br/>全メソッド対応]
    Opt2 --> Good[✅ 良い<br/>既存パス活用]
    Opt3 --> OK[⚠️ まあまあ<br/>局所的対応]
    Opt4 --> Temp[⚠️ 応急処置<br/>一時的]

    Best --> Scope1[影響範囲: 大]
    Good --> Scope2[影響範囲: 中]
    OK --> Scope3[影響範囲: 小]
    Temp --> Scope4[影響範囲: 極小]

    style Best fill:#00FF00
    style Good fill:#90EE90
    style OK fill:#FFD700
    style Temp fill:#FFA500
```

---

**作成日**: 2025-10-17
**用途**: Task 1 調査結果の視覚化
