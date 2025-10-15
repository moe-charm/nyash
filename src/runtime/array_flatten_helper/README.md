Array Flatten Helper — Split Design (Phase 15.75)

責務
- CallableBox.call/callAsync の argv 配列（ArrayBox）を安全にフラット化するための薄い補助。
- 実装は「builtin 用」と「plugin 用」に分割し、共通ファサード（array_flatten_helper.rs）から委譲する。

入出力
- is_array(&VMValue) -> bool
- get_len(&VMValue) -> usize
- get_element(&VMValue, index) -> VMValue

分割方針
- builtin: legacy-boxes 機能でのみ有効。ArrayBox の items を直接参照する（高速）。
- plugin: PluginBoxV2(ArrayBox) に対して method_router で size/get を呼び出す（安全・一貫）。

ガード
- feature = legacy-boxes の有無で builtin を優先し、plugin へフォールバック。
- いずれも型不一致時は失敗せず、保守的に 0 / v.clone() を返す（呼び出し側で安全に扱う）。

撤退計画
- legacy-boxes 撤退時は builtin 実装を削除し、plugin 実装のみ残す（API 互換のためファサードは維持）。

