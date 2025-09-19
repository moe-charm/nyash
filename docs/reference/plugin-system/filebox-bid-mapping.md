FileBox × BID-FFI 対応表（Nyash API ↔ Plugin ABI）

概要
- 目的: Nyash言語における `FileBox` のAPIを、BID-FFIプラグイン実装（C ABI）と正確に対応付ける。
- 設置: C:\git\nyash-project\nyash\docs\説明書\reference\box-design\filebox-bid-mapping.md（Windowsパス例）

前提
- BID-FFI v1（2段階応答/ShortBuffer=-1）
- TLVヘッダ: `u16 version(=1)`, `u16 argc`
- TLVエントリ: `u8 tag`, `u8 reserved(0)`, `u16 size`, payload
- 主要タグ: 1=Bool, 2=I32, 3=I64, 4=F32, 5=F64, 6=String, 7=Bytes, 8=Handle(u64), 9=Void

メソッドID（プラグイン側）
- 0: birth(instance生成) → 戻り値: u32 instance_id（暫定）
- 1: open(String path, String mode) → Void
- 2: read(I32 size) → Bytes
- 3: write(Bytes data) → I32（書込バイト数）
- 4: close() → Void
- 0xFFFF_FFFF: fini（破棄）

Nyash API ↔ Plugin ABI 対応
- 構築: `new FileBox(path: string)`
  - 既定動作: プラグイン設定が有効な場合、birth→open(path, "rw") を内部実行
  - フォールバック: プラグインが無効/未設定ならビルトインFileBoxを使用

- 書込: `FileBox.write(data: string)`
  - 変換: String → Bytes（UTF-8）
  - 呼出: method_id=3（write）
  - 戻り: I32 を受け取り、Nyash側は "ok" を返却（将来は書込サイズも返せる拡張余地）

- 読取: `FileBox.read([size: integer])`
  - 変換: 省略時デフォルト 1MB（1_048_576）を指定
  - 呼出: method_id=2（read）
  - 戻り: Bytes → String（UTF-8として解釈、失敗時はlossy）

- 閉じ: `FileBox.close()`
  - 呼出: method_id=4（close）
  - 戻り: Void → Nyash側は "ok"

エラーモデル（戻り値）
- 0: 成功
- -1: ShortBuffer（2段階応答。副作用なしで必要サイズを *result_len に返却）
- -2: InvalidType
- -3: InvalidMethod
- -4: InvalidArgs
- -5: PluginError
- -8: InvalidHandle

例（Nyashコード）
```
// プラグイン優先で FileBox を生成
local f
f = new FileBox("/tmp/nyash_example.txt")
f.write("Hello from Nyash via plugin!")
print("READ=" + f.read())
f.close()
```

実装メモ（現在の挙動）
- コンストラクタ: プラグイン有効時は birth→open("rw")。指定モードでの open は将来のAPI拡張候補（例: `FileBox.open(mode)`）。
- read(size): Nyashからサイズを指定するAPIは次段で追加予定。現状は既定1MBで読み取り。
- write: 書込サイズはプラグインからI32で返るが、Nyash側APIは簡便化のため "ok" を返却（将来拡張余地）。

関連ドキュメント
- plugin-ABI: docs/説明書/reference/box-design/ffi-abi-specification.md
- plugin system: docs/説明書/reference/box-design/plugin-system.md
- plugin-tester: docs/説明書/reference/plugin-tester.md

