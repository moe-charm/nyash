# Codex先生のNyash ABI実装戦略 (2025-09-01)

## 質問内容

Nyash ABIという統一ブリッジ規格の実装について技術的相談です。

【背景】
Gemini先生から以下の評価をいただきました：
- COM/GObject/WASM Component Modelとの類似性
- 3つのu64は十分な表現力
- GC言語との循環参照が最大の課題
- 非同期処理はmetadataフラグで統一可能
- 「間に挟むだけ」だが世界を変える可能性あり

【Nyash ABIの基本設計】
```c
typedef struct NyashValue {
    uint64_t type_id;     // 型識別子
    uint64_t box_handle;  // Arc<T>のポインタ
    uint64_t metadata;    // フラグ・インライン値
} NyashValue;

typedef NyashValue (*NyashFunc)(
    uint32_t argc,
    NyashValue* args,
    void* context
);
```

【具体的な実装相談】
1. MIR統合: ExternCallにabi_type追加で十分？それとも新命令？
2. バインディング生成: #[nyash_abi]マクロの実装戦略は？
3. 型レジストリ: type_idの生成と管理方法（ハッシュ？連番？）
4. インライン最適化: metadata活用で整数をbox化回避する設計
5. 移行戦略: 既存のC ABIプラグインとの共存方法

実装の観点から、最もシンプルで段階的に導入できる方法を教えてください。

## Codex先生の回答

以下は「まず壊さず、少しずつ置き換える」ための、最小実装から拡張していく導入方針です。現行の MIR/ランタイム構成を踏まえ、後方互換を維持しつつ Nyash ABI を段階導入できます。

**最小導入方針**
- 入口統一: ランタイム側で常に `NyashFunc`（3×u64）に呼び出しを揃える。既存C ABIはランタイム内の薄いトランポリンで包んで `NyashFunc` に見せる。
- MIRは無変更: まずは `MirInstruction::ExternCall` のままで十分。ABI種別はレジストリが関数ポインタ解決時に吸収。
- 段階移行: プラグインが新旧どちらでも動くよう、`nyash.toml` で ABI 種別を宣言。未宣言は旧C ABI扱いにし、ランタイムが自動でトランポリン生成。

**1. MIR統合（ExternCallの扱い）**
- 現状の `MirInstruction::ExternCall { iface_name, method_name, args, effects }` と `Semantics::extern_call(&str, &str, &[Val])` をそのまま使用。
- ランタイムの「外部呼び出し解決レイヤ（ユニファイドレジストリ）」で `(iface, method)` → 事前に登録済みの「NyashFuncポインタ」を返す。
- 既存C ABI実装はロード時に「C→Nyash ABI 変換トランポリン」を自動作成し、そのポインタを登録。JIT/LLVM/VM の各バックエンドは常に同じ `NyashFunc` 呼び出しシグネチャだけを生成すればよい。
- 将来的に ABI ヒントが必要なら、`ExternCall` にオプショナルな `abi: AbiKind` 追加は可能だが、初期段階では不要。

**2. バインディング生成（#[nyash_abi]）**
- 戦略: プロシージャルマクロで「Rust普通関数 ↔ NyashFuncシグネチャ」間の自動ラッパを生成。
- 生成内容:
  - `#[no_mangle] extern "C" fn exported(argc, *args, ctx) -> NyashValueABI` の外側関数（NyashFunc）。
  - 内側で ABI 配列 `NyashValueABI` を内部 `crate::value::NyashValue` にデコード、Rust関数を呼び、戻りを再度 ABI へエンコード。
  - 型境界は `FromAbi/ToAbi` トレイトで実装（i64/bool/string/box など）し、`#[nyash_abi] fn f(a: i64, b: String) -> bool` のような自然な記述で書けるようにする。
- 配布: `crates/nyrt/` に `nyash_abi.h`（C向けヘッダ）と `abi.rs`（Rust向け）を置く。プラグインはこのヘッダ/クレートにだけ依存。

**3. 型レジストリ（type_idの生成と管理）**
- 方式: 64bit固定ハッシュ（安定化のための固定シード）で「完全修飾名」をハッシュ。
  - 例: `"nyash.core/Integer"`, `"plugin.math/BigInt@1"` のように namespace + 型名 + バージョンを連結して xxhash/siphash で 64bit に。
- 衝突対策: 初回登録時に「同一IDだが異なる完全名」の場合は起動時に失敗（CIで検知）。内蔵型は予約IDを定数定義（Null, Void, Bool, I64, String など）。
- レジストリ: 既存の `src/runtime/unified_registry.rs` に統合、あるいは `abi_registry.rs` を新設。
  - `id -> vtable`（dtor/trace/to_string/cast）と `name -> id` を保持。
  - `nyash.toml` で型やメソッドの宣言を受け取り、ロード時に登録。プラグイン側も `#[nyash_abi(type = "...")]` で露出。

**4. インライン最適化（metadataの使い方）**
- 目的: 小さい値は box を避け、`box_handle=0` で `metadata` に即値を入れる。
- 提案レイアウト（初期版・シンプル）:
  - 最上位4bit: タグ（0=Boxed, 1=I63, 2=Bool, 3=Null, 4=Void, 5=Reserved(Future/Async)）
  - `I63`: 下位63bitに符号付き整数。`type_id=NYASH_TID_I64`, `box_handle=0`。
  - `Bool`: bit0 に値。`type_id=NYASH_TID_BOOL`, `box_handle=0`。
  - `Null/Void`: `box_handle=0`。`type_id` はそれぞれの予約ID。
  - `Boxed`: `box_handle` に `Arc<T>` のアドレス（生ポインタまたはハンドル値）。`metadata` の低位にフラグ（weak/async/readonly 等）を拡張可能。
- 浮動小数は第1段では Boxed のまま（安全優先）。将来 NaN-boxing 等は別フェーズで検討。

**5. 移行戦略（既存C ABIとの共存）**
- プラグイン宣言: `nyash.toml` に `abi = "c" | "nyash"` を追加。未指定は `"c"` と見做す。
- ローダ:
  - `"nyash"`: 直接 `NyashFunc` を dlsym。
  - `"c"`: 既存C関数を dlsym → ランタイム側トランポリンを生成（呼び出し前後で ABI <-> 旧C の変換）。
- 呼び出し側（MIR/バックエンド）: 常に `NyashFunc` を呼ぶのでコード生成は単純化。JIT/LLVM/VM の差異を ABI 解決層が吸収。
- 段階移行:
  - Phase 1: トランポリンで全プラグインが動く状態を作る（MIR無変更）。
  - Phase 2: 新規プラグインは `#[nyash_abi]` を使うガイドを提示。
  - Phase 3: ホットパスのプラグインだけ順次Nyash ABIネイティブ化（ボックス回避の恩恵）。

**GC/循環参照（要点）**
- 既定は「外部境界のエッジは弱参照扱い」。強参照を持つ場合のみ明示 pin/unpin API（`metadata` の weak/pinned フラグで伝達）を導入。
- レジストリの vtable に `trace()`/`finalize()` を用意し、GC 言語ブリッジは自前 GC へ協調（Mark 時にクロス境界を報告）。
- 非同期や Future は、「Boxed + asyncフラグ（or 専用 type_id=Future）」で統一し、`await` 時に解決。メタデータビットで「非同期値（未解決）」を示す。

**実装ステップ（具体）**
- NyRT:
  - `crates/nyrt/include/nyash_abi.h` と `crates/nyrt/src/abi.rs` を追加（`NyashValueABI`、エンコード/デコード、予約 type_id、タグ定義）。
  - `FromAbi/ToAbi` トレイトと安全ヘルパ。
- レジストリ:
  - `src/runtime/abi_registry.rs`（または既存の unified_registry に統合）で `(iface, method) -> NyashFunc*` を解決。
  - `nyash.toml` から ABI 種別と型宣言を読み取り登録。
- ローダ/トランポリン:
  - `"c"` プラグイン呼び出しを包む小さな C→Nyash ラッパ（呼出前後で値配列を変換）。
- マクロ:
  - `crates/nyash-macros` に `#[nyash_abi]` を実装。Rust関数に対して NyashFunc エクスポート＋型ID登録。

**最小で得られる効果**
- MIRやバックエンドは変更最小（もしくは無変更）で、呼出境界が統一。
- まず整数/真偽/null/void が即値で走り、箱化コストが減少。
- 既存プラグインは壊れず、徐々に `#[nyash_abi]` へ移行可能。

必要なら、この方針で最初のスケルトン（`nyash_abi.h/abi.rs`、トランポリン、レジストリ拡張）の差分案まで作ります。どの順から着手しますか？