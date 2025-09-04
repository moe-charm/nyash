CraneliftAotBox インタフェース草案（Phase 15 準備）

前提
- 本ブランチでは「仕様化（ドキュメント）」のみ行い、実装は別ブランチ `phase-15/self-host-aot-cranelift` で行う。
- Cargo フィーチャ: `cranelift-aot = ["dep:cranelift-object"]` を追加し、同フィーチャ時のみ AOT モジュールを有効化する。

モジュール構成（案）
- パス: `src/backend/cranelift/aot_box.rs`
- 依存: `cranelift-object`（オブジェクト出力）、既存のオブジェクトビルダ/ヘルパを再利用可能なら抽象化して流用。

公開型（案）
- `pub struct CraneliftAotConfig {`
  - `pub opt_level: u8`  // 0..3 程度（実装は後続）
  - `pub target: Option<String>` // target triple 等（未指定でホスト）
`}`

- `pub struct CraneliftAotBox {`
  - `obj: <object builder>`
  - `cfg: CraneliftAotConfig`
`}`

- `#[derive(Debug)] pub enum CraneliftAotError {`
  - `Codegen(String)`, `IO(String)`
`}`

主要メソッド（案）
- `impl CraneliftAotBox {`
  - `pub fn new(cfg: CraneliftAotConfig) -> Result<Self, CraneliftAotError>`
  - `pub fn compile_stub_ny_main_i64(&mut self, val: i64, out_obj: impl AsRef<Path>) -> Result<(), CraneliftAotError>`
    - 役割: PoC。`ny_main` 関数を定義し、即値 `val` を返すオブジェクトを生成。
  - `pub fn compile_mir_to_obj(&mut self, mir: MirModule, out_obj: impl AsRef<Path>) -> Result<(), CraneliftAotError>`
    - 役割: P1〜。最小 MIR（`const_i64`/`add_i64`/`ret`）から CLIF を組み立てて出力。
`}`

使用例（PoC フロー）
1) NyRT ビルド: `cargo build -p nyrt --release`
2) オブジェクト出力（CLIイメージ）:
   - `nyash --backend cranelift-aot --poc-const 42 apps/hello/main.nyash -o ny_main.o`
3) リンク:
   - Linux: `cc -o app ny_main.o target/release/libnyrt.a -ldl -lpthread`
   - Windows: `link ny_main.obj nyrt.lib /OUT:app.exe`
4) 実行: `./app` → `Result: 42` を確認。

エラーモデル（案）
- 環境・設定: フィーチャ未有効や未対応ターゲット → 分かりやすいメッセージ。
- 生成・出力: `CraneliftAotError::Codegen(_)`／`CraneliftAotError::IO(_)` で大別。

補助スクリプトの仕様（設計のみ）
- ファイル: `tools/aot_smoke_cranelift.sh`
- 目的: `.o/.obj` を生成→リンク→実行して PoC を自動検証。
- 主要引数: `apps/APP/main.nyash -o app`、必要に応じ `--const` を透過的に渡す。

今後の拡張（非ブロッキング）
- NyRT の外部関数呼び出し（checkpoint など）の導入。
- MIR 命令カバレッジの拡大、BoxCall/Plugin 経由の I/O。
- ターゲットトリプルとオブジェクト形式の明示的制御。

