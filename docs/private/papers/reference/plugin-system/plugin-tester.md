Nyash Plugin Tester - 開発者向けツールガイド

概要
- 目的: Nyash用プラグイン（BID-FFI準拠）の基本健全性を素早く診断するツール。
- 実装場所: `tools/plugin-tester`
- 想定対象: C ABIで `nyash_plugin_*` をエクスポートする動的ライブラリ（.so/.dll/.dylib）

ビルド
- コマンド: `cd tools/plugin-tester && cargo build --release`
- 実行ファイル: `tools/plugin-tester/target/release/plugin-tester`

サブコマンド
- `check <plugin>`: プラグインのロード、ABI確認、init呼び出し、型名・メソッド一覧の表示
- `lifecycle <plugin>`: birth→fini の往復テスト（インスタンスIDを返すことを確認）
- `io <plugin>`: FileBox向けE2E（open→write→close→open→read）テスト

使用例
- チェック:
  - `tools/plugin-tester/target/release/plugin-tester check plugins/nyash-filebox-plugin/target/release/libnyash_filebox_plugin.so`
  - 期待出力例:
    - `ABI version: 1`
    - `Plugin initialized`
    - `Box Type: FileBox (ID: 6)` と 6メソッド（birth/open/read/write/close/fini）の列挙
- ライフサイクル:
  - `tools/plugin-tester/target/release/plugin-tester lifecycle <path-to-plugin>`
  - 期待出力例: `birth → instance_id=1`, `fini → instance 1 cleaned`
- ファイルI/O:
  - `tools/plugin-tester/target/release/plugin-tester io <path-to-plugin>`
  - 期待出力例: `open(w)`, `write 25 bytes`, `open(r)`, `read 25 bytes → 'Hello from plugin-tester!'`

BID-FFI 前提（v1）
- 必須シンボル: `nyash_plugin_abi`, `nyash_plugin_init`, `nyash_plugin_invoke`, `nyash_plugin_shutdown`
- 返却コード: 0=成功, -1=ShortBuffer（2段階応答）, -2=InvalidType, -3=InvalidMethod, -4=InvalidArgs, -5=PluginError, -8=InvalidHandle
- 2段階応答: `result`がNULLまたは小さい場合は `*result_len` に必要サイズを設定し -1 を返す（副作用なし）

TLV（Type-Length-Value）概要（簡易）
- ヘッダ: `u16 version (=1)`, `u16 argc`
- エントリ: `u8 tag`, `u8 reserved(0)`, `u16 size`, `payload...`
- 主なタグ: 1=Bool, 2=I32, 3=I64, 4=F32, 5=F64, 6=String, 7=Bytes, 8=Handle(u64), 9=Void
- plugin-testerの `io` は最小限のTLVエンコード/デコードを内蔵

プラグイン例（FileBox）
- 実装場所: `plugins/nyash-filebox-plugin`
- メソッドID: 0=birth, 1=open, 2=read, 3=write, 4=close, 0xFFFF_FFFF=fini
- `open(path, mode)`: 引数は TLV(String, String)、返り値は TLV(Void)
- `read(size)`: 引数 TLV(I32)、返 TLV(Bytes)
- `write(bytes)`: 引数 TLV(Bytes)、返 TLV(I32: 書き込みバイト数)
- `close()`: 返 TLV(Void)

パスの指定（例）
- Linux: `plugins/nyash-filebox-plugin/target/release/libnyash_filebox_plugin.so`
- Windows: `plugins\nyash-filebox-plugin\target\release\nyash_filebox_plugin.dll`
- macOS: `plugins/nyash-filebox-plugin/target/release/libnyash_filebox_plugin.dylib`

トラブルシュート
- `nyash_plugin_abi not found`: ビルド設定（cdylib）やシンボル名を再確認
- `ShortBuffer`が返るのにデータが取れない: 2回目の呼び出しで `result` と `*result_len` を適切に設定しているか確認
- 読み出しサイズが0: 書き込み後に `close`→`open(r)` してから `read` を実行しているか確認

関連ドキュメント
- `docs/CURRENT_TASK.md`（現在の進捗）
- `docs/予定/native-plan/issues/phase_9_75g_bid_integration_architecture.md`（設計計画）

備考
- 本説明書は `C:\git\nyash-project\nyash\docs\説明書\reference\plugin-tester.md` に配置されます（Windowsパス例）。

