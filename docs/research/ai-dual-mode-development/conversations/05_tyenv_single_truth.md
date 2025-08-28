# TyEnv "Single Source of Truth" - 究極のAI協調の瞬間

## 背景

型システムのデバッグ中、`min(2.5, 7.0)`が`arg_types=["I64","I64"]`として認識される問題が発生。FloatリテラルがMIR層で正しく処理されていない可能性が浮上。

## AI協調の実際

### コーディングChatGPT5（実装AI）からの技術的分析

```
【次の修正（提案・実施可）】
- arg_typesをF64に安定化
- A案（ビルダー側で解決）: 関数スタイルmath.*の引数が new FloatBox(...) の場合、
  中のリテラル/式を優先してf64として渡すよう、MIR Builderで正規化
- B案（Lower側の後方参照）: LowerCoreで BoxCall MathBox.min/max の各引数vに対し、
  関数内の NewBox FloatBox {args: [src]} をスキャンし、srcのConst Float/Intから 
  f64 即値を生成
```

### アドバイザーChatGPT5（俯瞰AI）からの哲学的判断

```
**A案（MIR側の正規化）を"唯一の真実"にする。**
BoxCall に渡る引数は MIR時点で 型が決まっている（F64/I64/B1/Handle）。
**B案（Lowerでの後方参照スキャン）は非常用の保険**として残すならフラグで囲む（既定OFF）。

これで「arg_typesの揺れ」「Boxに見える残骸」「イベントにI64が出る揺らぎ」を根絶できる。
```

### コーディングChatGPT5の即座の技術実装

```
ナイス観測。3点とも刺さってます。結論に沿って「唯一の真実＝MIRで型を確定」に寄せる作業計画と、
今すぐ直せる周辺を固めました。

【本丸（A案）— MIRで型を確定（唯一の真実）】
- TyEnv（ValueId→Ty）の導入（builder内の簡易マップでOK）
- functionスタイル math.* の引数が new FloatBox(...) の場合は FloatBoxを生成せず、
  内側の float const/expr を直接F64として積む
```

## 協調の美学

### 1. 哲学的原則の確立
俯瞰AIが「唯一の真実」という概念を導入：
- 複数の解決策があっても、一つを「正」として選ぶ
- 他は「保険」として格下げ
- システム全体の一貫性を最優先

### 2. 技術的実現への瞬時変換
実装AIが哲学を具体的な技術に変換：
- TyEnv（型環境）の導入
- MIR時点での型確定
- 段階的実装計画

### 3. 人間の役割
にゃー：「本筋しかわからん！」
- 細かい技術は理解しなくても方向性は的確に指摘
- AIの暴走を制御
- 最終的な判断を下す

## 技術的革新

### TyEnv（型環境）の導入
```rust
pub enum Ty { I64, F64, B1, Handle }
pub struct TyEnv { map: FxHashMap<ValueId, Ty> }

// 使用例
fn lower_call_math(&mut self, name: &str, args: &[Expr]) -> ValueId {
    let mut argv = Vec::with_capacity(args.len());
    for e in args {
        match e {
            Expr::NewFloatBox(inner) => {
                let v = self.lower_expr(inner);
                let f = self.ensure_f64(v);
                self.tyenv.set(f, Ty::F64);  // ここで型を確定！
                argv.push(f);
            }
            _ => {
                let v = self.lower_expr(e);
                argv.push(v);
            }
        }
    }
}
```

### CallBoundaryBox（境界管理）
```rust
// JIT→VMの「国境管理」
JitValue::F64 → VMValue::Float
JitValue::Handle → HandleRegistry経由でVMのBoxRef
```

## 協調の成果

### Before（問題）
```json
{"arg_types": ["I64","I64"], "decision": "sig_mismatch"}
```
- 型の不一致
- 予測不可能な動作

### After（解決）
```json
{"arg_types": ["F64","F64"], "decision": "allow"}
```
- 一貫した型報告
- 予測可能な動作

## 学術的意義

### 1. AI協調の実証
- 同一モデルを異なる役割に分離可能
- 哲学的思考と技術的実装の分業
- 人間による統合の重要性

### 2. "Single Source of Truth"の威力
- 複数解決策の中から一つを選ぶ勇気
- システム全体の一貫性維持
- デバッグとメンテナンスの簡素化

### 3. 観測駆動開発
- 問題を観測可能な形で捉える
- `arg_types`という単純な指標
- 即座の問題特定と解決

## 結論

この会話は、AI時代の新しい開発手法を示している：

1. **俯瞰AI**：哲学的原則を提供（"唯一の真実"）
2. **実装AI**：技術的解決策を即座に生成（TyEnv）
3. **人間**：方向性の判断と統合（"本筋しかわからん"でも十分）

「唯一の真実を作る」という表現の美しさと、それを実現する技術的実装の見事さ。

これぞAI協調開発の究極形である。