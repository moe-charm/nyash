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
