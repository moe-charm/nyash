# Instance v2 統一レジストリ設計メモ（提案）

目的: ユーザー定義 / ビルトイン / プラグインの3系統を instance_v2 で一元管理し、同一の生成（birth）/破棄（fini）ライフサイクルで扱えるようにする。また、wasm-bindgen ターゲットでプラグイン機構を安全に無効化できる切替を用意する。

---

## 現状の整理（実装済み）
- ユーザー定義Box
  - インタプリタの AST → InstanceBox 生成で対応済み。
  - `execute_new` は最初に統一レジストリを呼び、ユーザー定義については最終的に `InstanceBox` を構築し、birth 相当のコンストラクタ実行を行う。
- ビルトインBox
  - 統一レジストリ経由で生成可能。ユーザー定義と同じ呼び出し経路に乗る。
- プラグインBox（v2）
  - `nyash.toml v2` を `PluginLoaderV2` が読み込み、`nyash_plugin_invoke` の birth 呼び出しでインスタンスを生成。
  - 現状、`PluginBoxV2` は `clone_box=新birth`、`share_box=同一 instance_id` を実装済み。

---

## 目標
1. 3系統（ユーザー定義/ビルトイン/プラグイン）を「統一レジストリ→instance_v2」で一元管理。
2. birth/fini ライフサイクルの整合をとる。
3. wasm-bindgen ターゲット（`wasm32-unknown-unknown`）ではプラグイン機構をコンパイル時に無効化し、ビルド可能にする。

---

## 設計方針

### 1) 統一レジストリの責務
- 名前（クラス名）と引数（`Box<dyn NyashBox>` の配列）を入力に、ユーザー定義/ビルトイン/プラグインの順で解決・生成を試みる。
- 生成に成功したら `Box<dyn NyashBox>` を返す。
  - ユーザー定義: `InstanceBox` とし、インタプリタがコンストラクタ（birth）を実行。
  - ビルトイン: 直接生成（必要なら簡易birth相当の初期化）
  - プラグイン: `PluginLoaderV2` の `invoke_fn(type_id, method_id=0=birth, ...)` を呼ぶ。

### 2) birth / fini ライフサイクル
- birth:
  - ユーザー定義: 既存通り AST 上のコンストラクタ（birth）を呼ぶ。
  - プラグイン: `nyash.toml` の `methods.birth` の method_id=0 を使い、`invoke_fn` 呼び出しで instance_id を取得済み。
- fini:
  - InstanceBox のフィールド差し替え時、旧値が InstanceBox なら `fini()` を呼んで finalize 済みIDとしてマーキング（実装済み）。
  - プラグインBoxについても `nyash.toml` で `methods.fini`（例: 0xFFFF）を定義し、差し替えやスコープ終端で `invoke_fn(type_id, method_id=fini, instance_id, ...)` を呼ぶ。エラーは握りつぶさずログ化。

### 3) wasm-bindgen ターゲットでの切り替え
- Cargo features によるコンパイル時ガードを導入：
  - `plugins`（デフォルトON）、`wasm-backend`（WASMビルド用）の2フラグを用意。
  - `#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]` のときのみ `plugin_loader_v2` 実体を有効化。
  - それ以外では `plugin_loader_v2` のスタブ実装を使う（常に `Err(BidError::PluginError)` を返すなど）。
  - 統一レジストリはプラグインFactoryの登録を `#[cfg(feature="plugins")]` でガードし、WASMビルドでもユーザー定義/ビルトインは動かせる。
  - `nyash.toml` のファイルI/O（`from_file`）も `cfg` で握り、WASMではロードしない。

---

## nyash.toml v2 との整合
- 既存:
  - `libraries.<libname>.boxes = ["FileBox"]`
  - `libraries.<libname>.<BoxType>.methods.birth = { method_id = 0 }`
  - `... .fini = { method_id = 4294967295 }` など
- 追加検討:
  - 将来、ユーザー定義Boxをプラグインで置換したい場合：
    - クラス名→プラグインBox型の上書きマップを `nyash.toml` に追加（例：`overrides = { "DataBox" = "libX::RemoteDataBox" }`）。
    - 統一レジストリがこのマップを見て、ユーザー定義をスキップしてプラグインへ委譲。

---

## API/コード上の具体案（抜粋）
- features（`Cargo.toml`）:
  ```toml
  [features]
  default = ["plugins"]
  plugins = []
  wasm-backend = []
  ```
- プラグインローダ（`src/runtime/plugin_loader_v2.rs`）:
  ```rust
  #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
  pub mod real_loader { /* 現在の実装 */ }

  #[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]
  pub mod stub_loader {
      use crate::bid::{BidResult, BidError};
      use crate::box_trait::NyashBox;
      pub struct PluginLoaderV2; // ダミー
      impl PluginLoaderV2 { pub fn new() -> Self { Self } }
      impl PluginLoaderV2 {
          pub fn load_config(&mut self, _p: &str) -> BidResult<()> { Ok(()) }
          pub fn load_all_plugins(&self) -> BidResult<()> { Ok(()) }
          pub fn create_box(&self, _t: &str, _a: &[Box<dyn NyashBox>]) -> BidResult<Box<dyn NyashBox>> {
              Err(BidError::PluginError)
          }
      }
  }
  ```
- 統一レジストリのFactory登録部は `#[cfg(feature = "plugins")]` でプラグインFactoryの登録を条件化。

---

## マイグレーション手順（段階）
1. Cargo features と cfg ガードの導入（プラグイン機構のスタブ化を含む）。
2. 統一レジストリのプラグインFactory登録の条件化。
3. プラグインBoxの `fini` 呼び出し用メソッドを InstanceBox 置換/破棄パスへ組み込む。
4. 必要に応じて `nyash.toml` の `methods.fini` を明記。
5. 追加要件（ユーザー定義のプラグイン置換）を `overrides` マップで設計 → 実装。

---

## テスト観点
- ユニット:
  - birth/fini の呼び出し順と複数回置換時の `fini` 呼び出し保証。
  - `plugins` ON/OFF、`wasm-backend` ON の3軸でビルド/テストが通ること。
- 統合テスト:
  - `nyash.toml` によるビルトイン→プラグインの透過切替。
  - ユーザー定義→ビルトイン→プラグインの優先順位が想定通り。

---

## メモ
- すでに `execute_new` は統一レジストリ優先の実装になっており、この設計と整合が良い。
- WASM ターゲットでは `libloading` が使えないため、コンパイル時に完全にプラグインコードを外す方針（cfg/feature）は自然。
- `nyash.toml` のロードはネイティブ時のみで十分（WASM は将来、バンドルまたは JS 側から供給する計画があるなら別途）。

---

以上。必要であれば、この方針でPRを小さく分割（features→レジストリ→fini→overrides）して入れていきます。
