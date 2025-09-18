# CorePy IR 最小スキーマ（C2草案）

目的: Phase 1 の End-to-End を最短で通すための暫定IR。将来は構造化・拡張（with/try/comp/async等）。

## JSON 形式（暫定）
```json
{
  "module": {
    "functions": [
      {
        "name": "main",                 // 省略可（既定: "main"）
        "return_value": 0,               // 省略可（bodyと排他）
        "body": [                        // 省略可（return_valueと排他）
          { "Return": { "value": 0 } }
        ]
      }
    ]
  }
}
```

ショートカット（デバッグ/ブリッジ用）
```json
{ "nyash_source": "static box Generated { main() { return 0 } }" }
```

## 変換規則（最小）
- module.functions[0] だけを見る（複数関数は将来対応）
- name があれば `static box Generated { <name>() { ... } }`
- return_value が数値/文字列なら `return <value>` を生成
- body があれば先頭の Return.value を探し、`return <value>` を生成
- 上記が無ければ `return 0`

## 将来（予約）
- statements: If/While/For/Assign/Expr などの節を追加
- expressions: BinOp/Call/Name/Constant などを構造化
- functions配列の複数対応、クロージャは別Box化の方針を検討

注意: All-or-Nothing 原則のもと、未対応ノードはCompiler側で明示的にエラーにする（現段階では未実装のため、return 0にフォールバックするが、C2終盤でStrict化する）。
