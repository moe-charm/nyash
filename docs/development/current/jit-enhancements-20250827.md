# JIT機能拡張 - 2025年8月27日

## ChatGPT5による最新実装

### 1. PHI可視化強化
- CFGダンプに「PHIがboolean(b1)かどうか」を明示表示
- 例: `dst v12 (b1) <- 1:8, 2:9`
- PHI命令の型情報が一目で分かるように改善

### 2. JIT統計の上位表示
- 統合JIT統計(JSON)に `top5` を追加
  - 関数名
  - ヒット数
  - compiled状態
  - handle番号
- 例実行で `top5` に `main` が入ることを確認

### 3. 返り値ヒントの観測
- `ret_bool_hint_count` をJIT統計に追加
- JitStatsBox/統合JIT統計の両方で確認可能
- ブール返り値の最適化機会を可視化

### 4. 新しい例の追加

#### `examples/jit_stats_bool_ret.nyash`
- 統計JSONをプリントする最小デモ
- 最後にブールを返す
- JIT統計の動作確認用

#### `examples/jit_mixed_f64_compare.nyash`
- f64比較のデモ
- **注意**: VMのf64演算/比較未対応のため、Cranelift有効環境向けサンプル

## 使い方（観測）

### 統計＋JSON出力
```bash
NYASH_JIT_STATS=1 NYASH_JIT_STATS_JSON=1 NYASH_JIT_THRESHOLD=1 \
  ./target/release/nyash --backend vm examples/jit_stats_bool_ret.nyash
```

JSONに以下が出力される：
- `abi_mode`
- `b1_norm_count`
- `ret_bool_hint_count`
- `top5`

### CFG/PHIダンプ
```bash
NYASH_JIT_DUMP=1 ./target/release/nyash --backend vm examples/phi_bool_merge.nyash
```
- b1 PHIには `(b1)` タグが付与される

## 注意事項

- VMのf64演算/比較は未対応
- `jit_mixed_f64_compare.nyash` はCranelift有効環境（JIT実行）での確認用
- VMでの実行はエラーになる

## 実装の意義

これらの機能拡張により：
1. **可視性の向上**: PHI命令の型情報が明確に
2. **統計の充実**: top5による頻繁に呼ばれる関数の把握
3. **最適化ヒント**: ブール返り値のカウントによる最適化機会の発見
4. **デバッグ支援**: より詳細な情報による問題解析の容易化

Box-First方法論の「観測箱」の具現化として、これらの機能は論文の実証例となる。