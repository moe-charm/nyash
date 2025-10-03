# Externs Registry — 完全疎結合への導線（設計メモ）

目的
- MIR 層から「外部境界（ExternCall）の情報」を一元管理するが、バックエンド固有の名前/ABI は持たない。
- 各バックエンドは Adapter で独自に命名/ABI を決める。Registry は意味論（インターフェース/メソッド/引数型/戻り型/効果）だけを提供する。

背景（問題）
- これまでは WASM/LLVM/VM がそれぞれ `"nyrt.time.now_ms" → 各バックエンド名` を直書きしていたため、重複と齟齬が生じやすかった。
- 初期の Registry 実装は利便上、バックエンド名（wasm_import/llvm_symbol）も持っていたが、これも結合を高める要因となる。

設計方針（Box‑First）
- Registry（共通）: 抽象的な意味論のみ
  - interface: `"nyrt.time"`
  - method: `"now_ms"`
  - params: `Vec<MirType>`（論理型）
  - returns: `MirType`
  - effects: `EffectMask`（最適化/検証の単一情報源）
- Adapter（各バックエンド固有）: 命名/ABI/呼出し規約
  - WASM: `interface.method → import名`（例: `time_now_ms`）と i32/i64/ptr によるABI決定
  - LLVM: `interface.method → シンボル名`（例: `nyrt_time_now_ms` or dotted），引数の正規化（handle/pointer）
  - VM: `interface.method → ハンドラ関数` を登録（実装は Rust のみ）

JSON エクスポート（Phase‑A 時点）
- Runner は `NYASH_EXTERN_SPEC_JSON` で LLVM ハーネスへ spec を渡す。
- フォーマットは配列（`[]`）で、要素は以下のフィールドを持つオブジェクト。
  - `interface`: 文字列。例 `"nyrt.time"`
  - `method`: 文字列。例 `"now_ms"`
  - `params`: 文字列配列。`MirType` を人可読フォーマットに変換（例 `"Integer"`, `"Box:ArrayBox"`）
  - `returns`: 文字列。戻り値の `MirType`
  - `effects`: 文字列。`pure|read|mut|io|control` のビット表現（複数の場合は `|` で連結）
- JSON が存在しない場合は Fail‑Fast（ハーネスが `Unknown extern` で停止）。Fallback シグネチャは撤去済み。
- 将来的に Schema を `docs/json-schema/externs_registry_v1.json` として切り出す予定（Phase‑B）。

### JSON Spec（最小スキーマ）
- 配列で Extern の仕様を列挙する（抽象情報のみ）。
- 各要素のフィールド（最小）:
  - `interface`: 例 `"nyrt.time"`
  - `method`: 例 `"now_ms"`
  - `params`: 例 `["Integer", "Box:ArrayBox"]`
  - `returns`: 例 `"Integer"`
  - `effects`: 例 `"read"`（複数は `"read|io"` のように `|` で連結）

例:
```
[
  {
    "interface": "nyrt.time",
    "method": "now_ms",
    "params": [],
    "returns": "Integer",
    "effects": "read"
  },
  {
    "interface": "nyrt.array",
    "method": "size",
    "params": ["Box:ArrayBox"],
    "returns": "Integer",
    "effects": "read"
  }
]
```

命名規則（LLVM）
- 既定は dotted 形式（`iface.method` → `nyrt.time.now_ms`）。
- `NYASH_LLVM_EXTERN_SYMBOL_STYLE=underscores` で `iface_method` も選択可（開発補助）。

Harness‑First と Fail‑Fast（更新）
- スモーク/実行は LLVM ハーネス（llvmlite）を第一にする（ネイティブLLVMは開発補助）。
- 未知 extern はデフォルトで Fail‑Fast（void() 宣言の暗黙フォールバックは既定で無効）。
  - 暫定: `NYASH_LLVM_UNKNOWN_EXTERN_FALLBACK=1` で一時的に許容（開発時のみ）
- MIR→JSON 変換時に Validator（`src/runner/mir_json_validate.rs`）で必須キーを確認し、欠落は即時停止。
- Python 側でも 0 値と None を誤解しない実装に整理（例: `src` 取得時の `or` を廃止）。
- LLVM シンボル命名: 既定は dotted（`iface.method` → `nyrt.time.now_ms`）で Kernel と一致。
  - `NYASH_LLVM_EXTERN_SYMBOL_STYLE=underscores` で `iface_method` スタイルを選択可能。
  - どちらの形式でも一意に解決されるよう、ハーネスは既存シンボルを再利用（重複宣言を回避）。

### Effects と最適化（CSE/DCE）
- 効果は Registry が唯一の情報源。
- `READ` は「純粋（PURE）」ではない。従って CSE の対象外（再利用不可）。並べ替えは `read_only` の範囲でのみ許可。
- ExternCall は「外部境界」扱いで、CSE は常に除外する（安全弁）。
- 実装メモ:
  - Builder は `TimerBox.now_ms()` を常に `ExternCall(nyrt.time, now_ms)` に正規化。
  - VM は ExternAdapter で `nyrt.time.now_ms` を直接処理（SystemTime→i64）。
  - quick スモークは `TimerBox.now_ms()` を静的呼び出しで確認（プラグイン/using に依存しない）。

WASM Adapter（Phase‑B 着手）
- `WasmExternAdapterBox` を導入し、`registry` の Spec から import 名・ABI（i32 固定）を生成。
- 規則: `interface = nyrt.time`, `method = now_ms` → module=`nyrt`, name=`time_now_ms`。必要に応じて overrides を定義。
- RuntimeImports は Adapter 経由で `nyrt.*` import 群を列挙。env.* 系（console/canvas など）は従来どおり個別管理。
- Codegen 側も Adapter を参照することで直書きマッピングを撤去。未知の extern は Fail-Fast。

段階導入（Phase）
1) Phase‑A（現状 → 最小緑維持）
   - CSE 安全弁: ExternCall/Extern callee は重複排除対象から除外
   - EffectResolver は `nyrt.*` を READ 等に正規化（Resolver優先+旧ヒューリスティック併用）
   - Router（既定OFF）: TimerBox.now_ms → Extern 直行、READ/ゼロ引数 getter を候補に追加

2) Phase‑B（疎結合化）
   - Registry を「抽象 spec のみ」に縮退（バックエンド名を削除）
   - WASM/LlVM/VM の Adapter を新設
     - WASM: 命名規則＋例外表 → import名を生成
     - LLVM: JSON で Registry spec を読み込み、命名規則＋例外表 → シンボル名を解決
     - VM: ハンドラ登録テーブル（Box<dyn Fn…>）に委譲
   - 未解決は Fail‑Fast（開発フラグで SKIP を選択可）

3) Phase‑C（後片付け）
   - 直書きマッピングの撤去 / フォールバック削減
   - docs/CI/smokes を Adapter 前提に更新

受け入れ基準
- quick プロファイル緑（Router=OFF/ON いずれも）
- TimerBox.now_ms の単調増加、Array/Map size の READ 一貫性
- LLVM ハーネスが準備できない環境では SKIP（方針維持）

運用ノート
- 追加する extern は Registry に 1 箇所登録（意味論のみ）→ 各 Adapter に例外表を 1 行追加 → スモーク 1 本
- Effect は常に Registry 起点。最適化（CSE/DCE/並べ替え）はここを唯一の真実として参照する。
