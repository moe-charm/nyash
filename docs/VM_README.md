# Nyash VM 実行基盤ガイド（更新）

- プラグインBox引数の最小対応を追加（TLV: BoxRef）
- TLVタグ: 1=Bool, 2=I32, 3=I64, 4=F32, 5=F64, 6=String, 7=Bytes, 8=Handle(BoxRef)
  - BoxRefはプラグインBox参照（type_id:u32, instance_id:u32）を8バイトでエンコード
  - ユーザー定義Box・複雑なビルトインは当面非対応（toStringフォールバック）

現状のルーティング:
- User-defined: MIR関数（{Box}.{method}/{N}) にCall化（関数存在時）。それ以外はBoxCall。
- Builtin: BoxCall → VM内の簡易ディスパッチ。
- Plugin: BoxCall → PluginLoaderV2.invoke_instance_method。

今後のタスク:
- VM側のfrom Parent.method対応（Builder/VM両対応）
- TLVの型拡張（Float/配列/BoxRef戻り値など）

## 🧮 VM実行統計（NYASH_VM_STATS / JSON）

VMは命令カウントと実行時間を出力できます。

使い方（CLIフラグ）:
```bash
# 人間向け表示
nyash --backend vm --vm-stats program.nyash

# JSON出力
nyash --backend vm --vm-stats --vm-stats-json program.nyash
```

環境変数（直接指定）:
```bash
NYASH_VM_STATS=1 ./target/debug/nyash --backend vm program.nyash
NYASH_VM_STATS=1 NYASH_VM_STATS_JSON=1 ./target/debug/nyash --backend vm program.nyash
# 代替: NYASH_VM_STATS_FORMAT=json
```

出力は `total`（総命令数）, `elapsed_ms`（経過時間）, `counts`（命令種別→回数）, `top20`（上位20種）を含みます。
