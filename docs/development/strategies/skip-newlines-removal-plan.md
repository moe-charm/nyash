# skip_newlines() 段階的削除計画

## 📊 現状分析（2025-09-24）

### 削減実績
- **初期**: 48箇所
- **Phase 1完了**: 40箇所（17%削減）
- **削除済み**: primary.rsのオブジェクトリテラル内8箇所

### 残存箇所（41箇所）
```
common.rs: 1
statements.rs: 4
match_expr.rs: 6
primary.rs: 1
box_definition.rs: 3
static_box.rs: 2
header.rs: 2
interface.rs: 3
postfix.rs: 2
fields.rs: 9（最多）
properties.rs: 1
constructors.rs: 2
parser_enhanced.rs: 4（実験的）
nyash_parser_v2.rs: 1（実験的）
```

## 🎯 削除戦略

### Phase 2-A: 簡単な括弧内削除（即座に可能）
**対象**: 明らかに括弧内にあるskip_newlines()
- `match_expr.rs`: match式内（brace_depth > 0）の6箇所
- `primary.rs`: 残り1箇所（ブロック内）
- **予想削減**: 7箇所（41→34）

### Phase 2-B: Box宣言系（中難度）
**対象**: Box定義内の改行処理
- `fields.rs`: 9箇所（フィールド宣言）
- `box_definition.rs`: 3箇所
- `static_box.rs`: 2箇所
- **課題**: フィールド区切りの判定が必要
- **予想削減**: 14箇所（34→20）

### Phase 2-C: 文処理系（高難度）
**対象**: 文の区切り判定に関わる部分
- `statements.rs`: 4箇所
- `header.rs`: 2箇所
- `interface.rs`: 3箇所
- **課題**: 文区切りと行継続の判定が複雑
- **予想削減**: 9箇所（20→11）

### Phase 2-D: メンバー宣言系（最高難度）
**対象**: Boxメンバーの宣言
- `constructors.rs`: 2箇所
- `properties.rs`: 1箇所
- `postfix.rs`: 2箇所
- **課題**: 複雑な構文の境界判定
- **予想削減**: 5箇所（11→6）

### Phase 2-E: 共通関数（最終段階）
**対象**: ParserUtilsの共通関数
- `common.rs`: 1箇所（skip_newlines_internal）
- **課題**: 全削除後に関数自体を削除
- **予想削減**: 1箇所（6→5）

### 実験的ファイル（除外）
- `parser_enhanced.rs`: 4箇所
- `nyash_parser_v2.rs`: 1箇所
これらは実験的実装のため削除対象外

## 📈 削減ロードマップ

```
Phase 1:   48 → 40 (8削除) ✅ 完了
Phase 2-A: 40 → 33 (7削除) 🚀 次のタスク
Phase 2-B: 33 → 19 (14削除)
Phase 2-C: 19 → 10 (9削除)
Phase 2-D: 10 → 5 (5削除)
Phase 2-E: 5 → 0 (5削除)※実験ファイル除く
```

## 🔧 実装方針

### 削除前のチェックリスト
1. ✅ 深度追跡が有効な箇所か確認
2. ✅ テストケースの準備
3. ✅ 削除後の動作確認
4. ✅ コミット単位を小さく保つ

### 削除時の置き換えルール
- **括弧内**: 単純削除（深度追跡で自動処理）
- **演算子後**: 単純削除（行継続判定で自動処理）
- **文区切り**: TokenCursorモード切り替えで対応
- **その他**: 個別判断

## 🎯 最終目標

1. **実用コードからskip_newlines()完全排除**
2. **TokenCursorによる統一的な改行処理**
3. **コード品質とメンテナンス性の向上**

## 📝 次のアクション

1. Phase 2-A: match_expr.rsの6箇所削除
2. テスト実行と動作確認
3. 段階的にPhase 2-B以降を実施