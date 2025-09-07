# MIR Dumper Output Guide

MIRダンプ出力を正しく読み解くためのガイドです。

## BoxCall vs 通常のCall の見分け方

### BoxCall形式（プラグイン/ビルトインBoxのメソッド）
```mir
%8 = call %7.cloneSelf()
%17 = call %7.open(%14, %16)
%22 = call %7.write(%21)
%23 = call %8.copyFrom(%7)
```

**特徴：**
- `call %値.メソッド名(引数)` の形式
- 値（%7, %8など）に対して直接メソッドを呼ぶ
- プラグインBoxやビルトインBoxで使用される

### 通常のCall形式（ユーザー定義Boxのメソッド）
```mir
%func = const "UserBox.calculate/2"
%result = call %func(%me, %arg1)
```

**特徴：**
- 事前に `const "クラス名.メソッド名/引数数"` で関数値を取得
- `call %関数値(%me, 引数...)` の形式で呼び出し
- 第1引数は常に `%me`（self相当）

## 実例での比較

### plugin_boxref_return.nyashのMIRダンプ
```mir
11: %7 = new FileBox()
12: call %7.birth()
13: %8 = call %7.cloneSelf()        ← BoxCall（プラグインメソッド）
26: %17 = call %7.open(%14, %16)    ← BoxCall（プラグインメソッド）
33: %22 = call %7.write(%21)        ← BoxCall（プラグインメソッド）
34: %23 = call %8.copyFrom(%7)      ← BoxCall（プラグインメソッド）
```

これらはすべてBoxCall形式で、プラグインのFileBoxメソッドを直接呼び出しています。

### ユーザー定義Boxの場合（仮想例）
```mir
; ユーザー定義Box "Calculator" のメソッド呼び出し
%calc_func = const "Calculator.add/2"
%result = call %calc_func(%me, %10, %20)
```

この場合は、MIR関数として事前にlower済みのメソッドを呼び出しています。

## まとめ

- **`call %値.メソッド()`** → BoxCall（プラグイン/ビルトイン）
- **`call %関数値(%me, ...)`** → 通常のCall（ユーザー定義Box）

MIRダンプを見る際は、この形式の違いに注目することで、どのタイプのメソッド呼び出しかを判断できます。