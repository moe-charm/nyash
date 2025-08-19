# プラグイン機構と WASM ビルド切替ガイド

本書は、nyash のプラグイン機構（nyash.toml v2 / PluginLoaderV2）と、wasm-bindgen を使う WASM ビルドでの切替手順をまとめたものです。目的は「ネイティブではプラグインON、WASMでは安全にプラグインOFF」でビルド/実行できるようにすることです。

---

## 1. Cargo features 構成（現状）

- `default = ["cli", "plugins"]`
- `plugins`: プラグイン機構を有効化（`libloading` 等の動的ロードに依存）
- `wasm-backend`: WASM バックエンドを有効化（`wabt`/`wasmtime` 等）

これにより、デフォルトではネイティブ開発時にプラグインが有効になります。WASM ターゲットでプラグインを完全に外したい場合は、`--no-default-features` を使います。

---

## 2. ビルド/テスト コマンド例

### 2.1 ネイティブ（プラグインON・デフォルト）

- ビルド: `cargo build`
- テスト: `cargo test`

nyash.toml v2 によるプラグインの透過切替（ビルトイン→プラグイン）はこの構成で有効です。

### 2.2 ネイティブ（プラグインOFF）

- テスト: `cargo test --no-default-features --features cli`

`plugins` を外すとプラグインローダはスタブ化され、プラグイン経由の生成は失敗（適切なエラー）になります。

### 2.3 WASM ターゲット（プラグインOFF）

- 初回のみ: `rustup target add wasm32-unknown-unknown`
- ビルド例: `cargo build --target wasm32-unknown-unknown --no-default-features --features wasm-backend`

この構成では `plugins` が無効のため、`libloading` 等の動的ロードに依存せずビルドできます。

---

## 3. runtime 側の切替実装（概要）

`src/runtime/plugin_loader_v2.rs` は cfg で2パスを提供:

- `#[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]` … 実体実装（libloading + invoke_fn）
- `#[cfg(any(not(feature = "plugins"), target_arch = "wasm32"))]` … スタブ実装（常にエラー返却）

`src/runtime/unified_registry.rs` は、`#[cfg(feature = "plugins")]` のときだけプラグインFactoryをレジストリに登録します。WASMやpluginsオフ時は登録されず、ユーザー定義/ビルトインのみが対象になります。

---

## 4. nyash.toml v2 の取り扱い

- ネイティブ: `init_global_loader_v2()` により `nyash.toml` を読み込み、`libraries`/`methods.birth`/`methods.fini` などの設定に従ってプラグインをロードします。
- WASM/プラグインOFF: `init_global_loader_v2()` はスタブ化され、ファイルI/Oや動的ロードは行いません。

---

## 5. ライフサイクル（birth/fini）整合（設計）

- ユーザー定義: AST上のコンストラクタ（birth）実行・フィールド差し替え時に `fini()` 呼び出し（InstanceBox側実装）
- プラグイン: `methods.birth=0` で生成済み。差し替え/破棄時に `methods.fini`（例: `0xFFFF`）を `invoke_fn(type_id, method_id=fini, instance_id)` で呼ぶフックを今後統合予定。

---

## 6. よくある質問

- Q. なぜランタイム切替ではなくコンパイル時切替？
  - A. WASM ターゲットでは `libloading` が使えないため、リンク/依存段階で除去する必要があり、features/cfg によるコンパイル時切替が安全・簡潔です。

- Q. `nyash.toml` はWASMでも使える？
  - A. ファイルI/O前提のため、現状はネイティブのみ。将来、バンドルや JS 側から設定注入をする場合は別アプローチで設計します。

---

## 7. 今後の拡張

- `nyash.toml` にクラス名→プラグインBox型の `overrides` マップを追加し、ユーザー定義Boxの外部置換をサポート（任意）。
- プラグインBoxの `fini` 呼び出しを InstanceBox の破棄/置換パスへ統合し、birth/fini ライフサイクルを完全整合。

以上。
