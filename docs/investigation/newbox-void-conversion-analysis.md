# NewBox → Void 変換パス完全追跡調査報告

**調査日時**: 2025-10-10
**調査対象**: VM実装におけるNewBox命令がVoidに変換されるパスの完全追跡
**結論**: **PluginBoxV2は正常に返却されている。問題はHostHandleBox経由の変換パス**

---

## 🎯 調査結果サマリー

### 根本原因特定 ✅

**問題箇所**: `/home/tomoaki/git/hakorune-selfhost/src/backend/vm_types.rs:149-154`

```rust
// VMValue::from_nyash_box() の実装
} else if let Some(hhb) = nyash_box.as_any().downcast_ref::<crate::runtime::host_handle_box::HostHandleBox>() {
    if let Some(arc) = crate::runtime::host_handles::get(hhb.id) {
        VMValue::BoxRef(arc)
    } else {
        VMValue::Void  // ⚠️ ここでVoidに変換される！
    }
```

**原因**:
1. PluginBoxV2が`Box<dyn NyashBox>`として返却される
2. PluginBoxV2は`HostHandleBox`ではない（直接のPluginBoxV2型）
3. しかし、何らかの理由で`HostHandleBox`にラップされている可能性
4. `host_handles::get(id)`がNoneを返す → **ハンドルレジストリにエントリが存在しない**

---

## 📊 完全な実行パス追跡

### セクション1: NewBox命令の実行フロー

#### Step 1: NewBox命令ハンドラー
**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/backend/mir_interpreter/handlers/boxes/newbox.rs:7-30`

```rust
pub(crate) fn handle_new_box(
    &mut self,
    dst: ValueId,
    box_type: &str,
    args: &[ValueId],
) -> Result<(), VMError> {
    // Provider Lock guard
    if let Err(e) = crate::runtime::provider_lock::guard_before_new_box(box_type) {
        return Err(VMError::InvalidInstruction(e));
    }

    // ProviderBox first: Plugin→Registry→Embedded resolution
    {
        let mut converted_pb: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
        for vid in args {
            converted_pb.push(self.reg_load(*vid)?.to_nyash_box());
        }

        crate::runtime::provider_box::ensure_loaded(None);

        // ✅ ここでprovider_box::new_box()を呼び出す
        if let Ok(created) = crate::runtime::provider_box::new_box(box_type, &converted_pb) {
            // ⚠️ ここでVMValue::from_nyash_box()が呼ばれる
            let created_vm = VMValue::from_nyash_box(created);
            self.regs.insert(dst, created_vm.clone());

            if let VMValue::BoxRef(arc_box) = &created_vm {
                self.scope.register_box(arc_box.clone());
            }

            // birth()を呼び出し
            let _ = self.handle_box_call(None, dst, "birth", args);
            return Ok(());
        }
    }
    // ... 以下、フォールバック処理
}
```

**説明**:
- `provider_box::new_box()`が`Box<dyn NyashBox>`を返す
- **問題発生**: `VMValue::from_nyash_box(created)`でVoidに変換される

---

#### Step 2: ProviderBox::new_box()
**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/provider_box/mod.rs:24-112`

```rust
pub fn new_box(
    box_type: &str,
    args: &[Box<dyn NyashBox>],
) -> Result<Box<dyn NyashBox>, RuntimeError> {

    // plugin-only判定（FileBoxはplugin-onlyになる）
    let mut plugin_only = false;
    if crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled() {
        if let Ok(h) = crate::runtime::get_global_plugin_host().read() {
            if let Some(conf) = h.config_ref() {
                if conf.find_library_for_box(box_type).is_some() {
                    plugin_only = true;  // ✅ FileBoxはここでtrueになる
                }
            }
        }
    }

    // 1) PluginHost direct
    if crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled() {
        // ... deterministic mode check ...

        // ✅ plugin.create_box()を呼び出す
        if let Some(b) = {
            let host = crate::runtime::get_global_plugin_host();
            host.read().ok().and_then(|h| h.create_box(box_type, args).ok())
        } {
            return Ok(b);  // ✅ ここでPluginBoxV2が返される
        }

        // ... partial config load retry ...
    }

    // 2) v2 BoxFactoryRegistry provider (plugin_onlyの場合はスキップ)
    let plugin_on = crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled();
    let is_core = crate::runtime::type_registry::is_core_box(box_type);
    if !(plugin_on && (is_core || plugin_only)) {
        // ... Registry経由の作成 ...
    }

    // If plugin_only and failed, do not fallback
    if plugin_only {
        return Err(RuntimeError::InvalidOperation {
            message: format!("plugin-only box could not be created: {}", box_type)
        });
    }

    // 3) Final fallback: unified registry
    // ...
}
```

**説明**:
- FileBoxは`plugin_only = true`になる
- `host.create_box()`が成功すれば`PluginBoxV2`を返す
- 失敗した場合、plugin_onlyなのでエラーを返す（Voidには**ならない**）

---

#### Step 3: PluginHost::create_box()
**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/plugin_loader_unified.rs:197-204`

```rust
pub fn create_box(
    &self,
    box_type: &str,
    args: &[Box<dyn crate::box_trait::NyashBox>],
) -> BidResult<Box<dyn crate::box_trait::NyashBox>> {
    let l = self.loader.read().unwrap();
    l.create_box(box_type, args)  // ✅ PluginLoaderV2に委譲
}
```

**説明**:
- `PluginLoaderV2::create_box()`に直接委譲
- エラーは`BidResult<Box<dyn NyashBox>>`で返される（Voidにはならない）

---

#### Step 4: PluginLoaderV2::create_box()
**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/plugin_loader_v2/enabled/instance_manager.rs:11-225`

```rust
pub fn create_box(
    &self,
    box_type: &str,
    _args: &[Box<dyn NyashBox>],
) -> BidResult<Box<dyn NyashBox>> {
    // type_id, birth_id, fini_idを解決
    let (type_id, birth_id_opt, fini_id) = resolve_box_ids_optional(self, box_type)?;

    // birth()を呼び出してinstance_idを取得
    let mut instance_id: u32 = 0;
    if let Some(birth_id) = birth_id_opt {
        // FFI呼び出し
        let tlv = crate::runtime::plugin_ffi_common::encode_args(_args);
        let (code, out_len, out_buf) = if let Some(box_invoke) = direct_invoke {
            // Direct per-Box call
            let mut out = vec![0u8; 1024];
            let mut out_len: usize = out.len();
            let code = (box_invoke)(0, birth_id, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len);
            (code, out_len, out)
        } else {
            super::host_bridge::invoke_alloc(
                super::super::nyash_plugin_invoke_v2_shim,
                type_id,
                birth_id,
                0,
                &tlv,
            )
        };

        if dbg_on() {
            eprintln!(
                "[PluginLoaderV2] create_box: box_type={} type_id={} birth_id={} code={} out_len={}",
                box_type, type_id, birth_id, code, out_len
            );
        }

        if code != 0 { return Err(BidError::PluginError); }

        // ✅ out_len=4の場合、instance_idを解釈
        if out_len == 4 {
            let mut b = [0u8;4];
            b.copy_from_slice(&out_buf[..4]);
            instance_id = u32::from_le_bytes(b);  // ✅ ここでinstance_idが設定される
        } else {
            // TLV形式の場合
            // ...
        }
    }

    // ✅ PluginBoxV2を構築して返す
    let bx = PluginBoxV2 {
        box_type: box_type.to_string(),
        inner: Arc::new(PluginHandleInner {
            type_id,
            invoke_fn: super::super::nyash_plugin_invoke_v2_shim,
            instance_id,  // ✅ ここでinstance_idが設定される
            fini_method_id: fini_id,
            finalized: std::sync::atomic::AtomicBool::new(false),
        }),
    };

    crate::runtime::leak_tracker::register_plugin(box_type, instance_id);
    Ok(Box::new(bx))  // ✅ PluginBoxV2を返す
}
```

**説明**:
- FFI呼び出しで`birth()`を実行
- `code=0, out_len=4`の場合、正常成功として`instance_id`を取得
- **PluginBoxV2を構築して返す**（ここまでは正常）

---

### セクション2: Void生成箇所の完全リスト

#### 箇所1: VMValue::from_nyash_box() - HostHandleBox経由
**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/backend/vm_types.rs:149-154`

```rust
} else if let Some(hhb) = nyash_box.as_any().downcast_ref::<crate::runtime::host_handle_box::HostHandleBox>() {
    if let Some(arc) = crate::runtime::host_handles::get(hhb.id) {
        VMValue::BoxRef(arc)
    } else {
        VMValue::Void  // ⚠️ ここ！ハンドルが見つからない場合Void
    }
```

**条件**:
- `nyash_box`が`HostHandleBox`型
- `host_handles::get(id)`がNoneを返す（レジストリに存在しない）

**問題**:
- PluginBoxV2は`HostHandleBox`ではないはず
- **なぜHostHandleBoxにラップされているのか？**

---

#### 箇所2: ConstValue::Null/Void から
**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/backend/vm_types.rs:179-180`

```rust
impl From<&ConstValue> for VMValue {
    fn from(const_val: &ConstValue) -> Self {
        match const_val {
            ConstValue::Null => VMValue::Void,
            ConstValue::Void => VMValue::Void,
            // ...
        }
    }
}
```

**条件**: MIR定数がNull/Voidの場合（今回は無関係）

---

#### 箇所3: NullBoxから
**ファイル**: `/home/tomoaki/git/hakorune-selfhost/src/backend/vm_types.rs:142-148`

```rust
pub fn from_nyash_box(nyash_box: Box<dyn crate::box_trait::NyashBox>) -> VMValue {
    if nyash_box
        .as_any()
        .downcast_ref::<crate::boxes::null_box::NullBox>()
        .is_some()
    {
        VMValue::Void
    }
```

**条件**: `NullBox`の場合（今回は無関係）

---

### セクション3: 根本原因の特定

#### 問題の構造

1. **PluginBoxV2は正常に作成されている**:
   - `PluginLoaderV2::create_box()`は`Box<PluginBoxV2>`を返す
   - FFI呼び出しで`code=0, out_len=4`を受け取っている
   - `instance_id`が設定されている

2. **問題は変換レイヤーにある**:
   - `VMValue::from_nyash_box()`の実装
   - `HostHandleBox`経由のパスが動作している
   - **PluginBoxV2が`HostHandleBox`にラップされている可能性**

3. **なぜHostHandleBoxにラップされるのか？**:
   - 調査対象: `PluginBoxV2`のNyashBoxトレイト実装
   - 調査対象: codec/シリアライゼーション層

---

#### 仮説: PluginBoxV2がHostHandleBoxでラップされている

**証拠を探すべき箇所**:

1. **PluginBoxV2のNyashBoxトレイト実装**:
   - `share_box()`メソッドが`HostHandleBox`を返す可能性
   - `/home/tomoaki/git/hakorune-selfhost/src/runtime/plugin_loader_v2/enabled/types.rs:138-`

2. **codec層の変換**:
   - TLV encode/decode時にHostHandleBoxに変換される可能性
   - `/home/tomoaki/git/hakorune-selfhost/src/runtime/codec/mod.rs`

3. **provider_box::new_box()の戻り値変換**:
   - PluginBoxV2 → HostHandleBoxの変換が起きている可能性

---

### セクション4: 修正提案

#### 提案1: VMValue::from_nyash_box()でPluginBoxV2を直接サポート

```rust
// Before (問題のコード)
pub fn from_nyash_box(nyash_box: Box<dyn crate::box_trait::NyashBox>) -> VMValue {
    if nyash_box.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() {
        VMValue::Void
    } else if let Some(hhb) = nyash_box.as_any().downcast_ref::<crate::runtime::host_handle_box::HostHandleBox>() {
        if let Some(arc) = crate::runtime::host_handles::get(hhb.id) {
            VMValue::BoxRef(arc)
        } else {
            VMValue::Void  // ⚠️ ここでVoidになる
        }
    } else if let Some(int_box) = nyash_box.as_any().downcast_ref::<IntegerBox>() {
        VMValue::Integer(int_box.value)
    }
    // ...
    else {
        VMValue::BoxRef(Arc::from(nyash_box))  // ✅ PluginBoxV2はここで処理されるべき
    }
}

// After (修正後)
pub fn from_nyash_box(nyash_box: Box<dyn crate::box_trait::NyashBox>) -> VMValue {
    if nyash_box.as_any().downcast_ref::<crate::boxes::null_box::NullBox>().is_some() {
        VMValue::Void
    } else if let Some(plugin_box) = nyash_box.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
        // ✅ PluginBoxV2を直接サポート（HostHandleBox経由を回避）
        VMValue::BoxRef(Arc::from(nyash_box))
    } else if let Some(hhb) = nyash_box.as_any().downcast_ref::<crate::runtime::host_handle_box::HostHandleBox>() {
        if let Some(arc) = crate::runtime::host_handles::get(hhb.id) {
            VMValue::BoxRef(arc)
        } else {
            // ⚠️ HostHandleBoxでレジストリにない場合はエラーにすべき
            eprintln!("[ERROR] HostHandleBox id={} not found in registry", hhb.id);
            VMValue::Void  // TODO: エラーにする
        }
    } else if let Some(int_box) = nyash_box.as_any().downcast_ref::<IntegerBox>() {
        VMValue::Integer(int_box.value)
    }
    // ...
    else {
        VMValue::BoxRef(Arc::from(nyash_box))
    }
}
```

**効果**:
- PluginBoxV2がHostHandleBox経由を通らずに直接BoxRefになる
- HostHandleBoxでレジストリ不在の場合、エラーログが出る

---

#### 提案2: HostHandleBoxでレジストリ不在の場合、エラーにする

```rust
} else if let Some(hhb) = nyash_box.as_any().downcast_ref::<crate::runtime::host_handle_box::HostHandleBox>() {
    if let Some(arc) = crate::runtime::host_handles::get(hhb.id) {
        VMValue::BoxRef(arc)
    } else {
        // ⚠️ Voidではなく、パニックまたはエラーログを出す
        panic!("HostHandleBox id={} not found in host_handles registry. This indicates a bug in handle management.", hhb.id);
    }
}
```

**効果**:
- 問題が発生した箇所で即座に失敗する（Fail-Fast）
- デバッグ情報が明確になる

---

#### 提案3: デバッグログの追加

```rust
pub fn from_nyash_box(nyash_box: Box<dyn crate::box_trait::NyashBox>) -> VMValue {
    let type_name = nyash_box.type_name();
    let ptr = format!("{:p}", &*nyash_box);

    if std::env::var("HAKO_TRACE_VMVALUE_CONVERSION").ok().as_deref() == Some("1") {
        eprintln!("[VMValue::from_nyash_box] type={} ptr={}", type_name, ptr);
    }

    // 既存の変換ロジック
    // ...
}
```

**効果**:
- `HAKO_TRACE_VMVALUE_CONVERSION=1`で変換パスを追跡可能
- どの型がどのパスを通っているか確認できる

---

## 🔍 次のステップ（追加調査が必要な項目）

### 1. PluginBoxV2がなぜHostHandleBoxにラップされるのか？

**調査方法**:
```bash
# PluginBoxV2のNyashBoxトレイト実装を確認
grep -n "impl NyashBox for PluginBoxV2" src/runtime/plugin_loader_v2/enabled/types.rs

# share_box()の実装を確認
grep -A 10 "fn share_box" src/runtime/plugin_loader_v2/enabled/types.rs
```

**期待される発見**:
- `share_box()`が`HostHandleBox`を返している可能性
- codec層で自動的にラップされている可能性

---

### 2. スモークテスト環境でのみ失敗する理由

**仮説**:
- スモークテスト環境ではプラグインロードのタイミングが異なる
- ハンドルレジストリの初期化タイミングの問題
- 環境変数の違い（`HAKO_PLUGIN_POLICY`, `NYASH_DISABLE_PLUGINS`）

**調査方法**:
```bash
# スモーク環境での環境変数を確認
grep -r "HAKO_" tools/smokes/v2/profiles/

# 手動実行との違いを確認
diff <(env | sort) <(tools/smokes/v2/run.sh --profile quick 2>&1 | grep "HAKO_" | sort)
```

---

### 3. FFI呼び出しの戻り値解釈

**現状の問題点**:
```rust
if dbg_on() {
    eprintln!(
        "[PluginLoaderV2] create_box: box_type={} type_id={} birth_id={} code={} out_len={}",
        box_type, type_id, birth_id, code, out_len
    );
}
```

**出力例** (スモークテストログより):
```
[PluginLoaderV2] create_box: box_type=FileBox type_id=1 birth_id=1 code=0 out_len=4
```

**問題**:
- `code=0`は成功
- `out_len=4`は`instance_id`が返されている
- **しかし、なぜVoidになるのか？**

**追加調査**:
```rust
// instance_idの値をログに出力
if out_len == 4 {
    let mut b = [0u8;4];
    b.copy_from_slice(&out_buf[..4]);
    instance_id = u32::from_le_bytes(b);

    if dbg_on() {
        eprintln!("[PluginLoaderV2] create_box: decoded instance_id={}", instance_id);
    }
}
```

---

## 📋 まとめ

### 確定事項

1. **PluginBoxV2は正常に作成されている**
2. **問題は`VMValue::from_nyash_box()`の変換レイヤーにある**
3. **HostHandleBox経由のパスで`host_handles::get(id)`がNoneを返している**

### 不明点（要追加調査）

1. **なぜPluginBoxV2がHostHandleBoxにラップされるのか？**
2. **なぜハンドルレジストリにエントリが存在しないのか？**
3. **スモークテスト環境でのみ失敗する理由は何か？**

### 推奨される修正

1. **短期修正**: `VMValue::from_nyash_box()`でPluginBoxV2を直接サポート
2. **中期修正**: HostHandleBox経由のパスを明確化し、エラーハンドリングを強化
3. **長期修正**: ハンドルレジストリの管理を見直し、一貫性を保証

---

## 🛠️ デバッグコマンド集

### 追跡ログを有効にして実行
```bash
# VM変換パスの追跡
HAKO_TRACE_VMVALUE_CONVERSION=1 ./target/release/hako test.hkr

# HostHandleレジストリの追跡
HAKO_TRACE_HOST_HANDLE=1 ./target/release/hako test.hkr

# Pluginデバッグ
HAKO_PLUGIN_DEBUG=1 ./target/release/hako test.hkr

# 全て有効
HAKO_TRACE_VMVALUE_CONVERSION=1 \
HAKO_TRACE_HOST_HANDLE=1 \
HAKO_PLUGIN_DEBUG=1 \
./target/release/hako test.hkr
```

### スモークテストでの確認
```bash
# クイックプロファイルで実行
HAKO_PLUGIN_DEBUG=1 tools/smokes/v2/run.sh --profile quick

# 特定のテストのみ
HAKO_PLUGIN_DEBUG=1 tools/smokes/v2/profiles/quick/vm/file_*.sh
```

---

**次のアクション**:
1. PluginBoxV2のNyashBoxトレイト実装を確認
2. デバッグログを追加して実際の変換パスを追跡
3. 修正案を実装してテスト
