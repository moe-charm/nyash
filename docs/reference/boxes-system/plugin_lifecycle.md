# プラグインBoxのライフサイクル（v2）と nyash.toml 定義

本書は、プラグインBox（PluginBoxV2）の生成（birth）と終了（fini）の流れ、`singleton` オプション、ならびに nyash.toml v2 における `methods` 定義の役割をまとめたものです。

---

## 1. 用語
- birth: プラグインBoxのインスタンス生成（`method_id=0`）
- fini: プラグインBoxの終了処理（任意の `method_id`。例: `4294967295`）
- invoke_fn: プラグイン側の単一エントリポイント（`nyash_plugin_invoke`）

---

## 2. 生成（birth）の流れ
1. `unified registry` が `PluginLoaderV2::create_box(box_type, args)` を呼び出す。
2. `PluginLoaderV2` は `nyash.toml` から `type_id` と `methods` を読み込む。
3. `invoke_fn(type_id, method_id=0 /* birth */, instance_id=0, ...)` を呼び、戻り値（出力TLV）の先頭4バイトから `instance_id` を取得。
4. `PluginBoxV2 { box_type, inner: Arc<PluginHandleInner> }` を生成して返す。
   - `PluginHandleInner` は `{ type_id, instance_id, invoke_fn, fini_method_id, finalized }` を保持し、参照カウント（Arc）で共有される。

補足:
- `fini_method_id` は `nyash.toml` の `methods` から `fini` の `method_id` を取り出して保持します。未定義の場合は `None`。

---

## 3. 終了（fini）の流れ（現在）
- フィールド差し替え時（代入で旧値を置き換えるとき）:
  - 旧値が `InstanceBox` の場合: インタプリタが `fini()` を呼び、finalized としてマーキングします。
  - 旧値が `PluginBoxV2` の場合: `fini_method_id` が設定されていれば `invoke_fn(type_id, fini_method_id, instance_id, ...)` を呼びます。
- プラグインBox（PluginBoxV2）:
  - すべての参照（Arc）がDropされ「最後の参照が解放」された時、`Drop`で一度だけ `fini` を呼ぶ（RAII、二重呼び出し防止）。
  - 明示finiが必要な場合は `PluginBoxV2::finalize_now()` を使える（内部的に一度だけfini実行）。
  - 代入/フィールド代入/Map.get/Array.get/slice/退避などは「PluginBoxV2は共有（share）、それ以外は複製（clone）」で統一。

---

## 4. nyash.toml v2 の定義例（methods + singleton）

```toml
[libraries]
[libraries."libnyash_filebox_plugin.so"]
boxes = ["FileBox"]
path = "./plugins/nyash-filebox-plugin/target/release/libnyash_filebox_plugin.so"

[libraries."libnyash_filebox_plugin.so".FileBox]
type_id = 6

[libraries."libnyash_filebox_plugin.so".FileBox.methods]
birth = { method_id = 0 }
open  = { method_id = 1 }
read  = { method_id = 2 }
write = { method_id = 3 }
close = { method_id = 4 }
fini  = { method_id = 4294967295 } # 任意の終端ID
```

要点:
- `methods` に `fini` を定義すれば、差し替え時などに fini が呼ばれます。
- `fini` 未定義の場合、プラグインBoxの終了処理は呼ばれません（フォールバック動作）。

### singleton例

```toml
[libraries."libnyash_counter_plugin.so".CounterBox]
type_id = 7
singleton = true

[libraries."libnyash_counter_plugin.so".CounterBox.methods]
birth = { method_id = 0 }
inc = { method_id = 1 }
get = { method_id = 2 }
fini = { method_id = 4294967295 }
```

- `singleton = true` を設定すると、ローダー初期化時に事前birthし、ローダーが共有ハンドルを保持します。
- `create_box()` は保持中の共有ハンドルを返すため、複数回の `new` でも同一インスタンスを共有できます。
- Nyash終了時（または明示要求時）に `shutdown_plugins_v2()` を呼ぶと、ローダーが保持する全シングルトンの `fini` を実行し、クリーンに解放されます。

---

## 5. WASM（wasm-bindgen）との関係
- WASMターゲットでは `libloading` が使えないため、プラグイン機構は features/cfg でスタブ化しています。
- `plugins` フィーチャを外す、または `target_arch = "wasm32"` のときは、プラグイン生成・fini 呼び出しのコードはコンパイル対象外になります（ビルド可能化のため）。

---

## 6. 将来拡張の方向
- ローカル変数のスコープ終了時（関数/メソッド呼び出しの戻りなど）に、InstanceBox/PluginBoxV2 の fini を安全に呼び出す仕組み（順序・例外耐性・二重呼び出し防止を含む）。
- `nyash.toml` にクラス名→プラグインBox型の `overrides` を加え、ユーザー定義Boxの外部置換を許可する設計（任意）。

以上。

---

## 7. v2.1: BoxRef（Box引数）サポート

目的: プラグインメソッドの引数として、他のBoxインスタンスを不透明参照で受け渡し可能にする。

- 仕様詳細: `docs/reference/plugin-system/nyash-toml-v2_1-spec.md`
- 設定例（1引数にプラグインBoxを渡す）:

```toml
[libraries."libnyash_filebox_plugin.so".FileBox.methods]
copyFrom = { method_id = 7, args = [ { kind = "box", category = "plugin" } ] }
```

注意:
- 当面は `category = "plugin"` のみ対応。ユーザー定義Boxや複雑なビルトインBoxは非対応。
- 戻り値の BoxRef は次版（v2.2）で検討。
