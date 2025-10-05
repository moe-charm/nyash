# Scanning Policy — Box‑First, Fail‑Fast, Minimal Surface

目的: 生文字列処理の地雷（境界/エスケープ/無限ループ/多経路）を箱で封じ込め、前進保証と Fail‑Fast を徹底する。

原則
- スキャンは必ず箱経由にする。
  - 低レベル: `ScannerBox`（peek/advance/at_end, 前進保証）
  - JSON断片: `JsonScanBox`（`seek_array_end`/`seek_obj_end`）
  - キー抽出: `JsonFragBox.get_int/get_str`（小用途）
- ループは `GuardBox` を必ず併用（無限ループの抑止。上限は小さく 256–1024）。
- 生クオート/バックスラッシュ直比較は避ける（ヘルパで表現 or JsonScanBox に委譲）。
- Map のフィールド参照は `box.get("key")` を使う（ドットはメソッド呼び出し用）。

段階的境界（seek → substring → Frag）
- まず構造の外形を決める: `seek_array_end` / `seek_obj_end` で安全な範囲を確定する。
- 次に部分文字列を切り出す: 走査対象を substring で局所化し、外側の括弧に干渉しないようにする。
- 最後に値を抽出する: `JsonFragBox.get_int/get_str` でキー単位に最小限の取得だけ行う。
- これにより「範囲外参照」「未終端」「括弧崩れ」に強くなり、Fail‑Fast の位置特定が容易になる。

実装パターン（例: PHI values[]）
1) `key = "\"values\":["` を探す → `arr_br = p + key.length() - 1`
2) `end = JsonScanBox.seek_array_end(text, arr_br)` で配列終端を取得
3) `i = arr_br + 1` から `end` まで `GuardBox("phi_values", 512)` とともに走査
4) 各オブジェクトは `ob .. JsonScanBox.seek_obj_end(..)` で substring → `JsonFragBox.get_int(obj, "pred"/"value")`
5) `pred==prev_bb` を優先、無ければ最初の `value` を fallback

Fail‑Fast
- 走査上限に達したら即中断（戻りはエラー/Result.err）。
- 取得できない/異常フォーマットは Result で上位へ明示伝播。

導入状況（2025-10-04）
- Mini‑VM: PHI デコードは `PhiDecodeBox.decode_result` → `PhiApplyBox.apply` に一本化（values[]/single 共通）。
- ユーティリティ箱: `ScannerBox`, `GuardBox`, `ResultBox` が導入済み。

チェックリスト
- [ ] 生の `indexOf` + while で配列/オブジェクトを跨いでいないか
- [ ] `seek_*` を使って範囲を限定したか
- [ ] `GuardBox` を併用しているか
- [ ] Map フィールド参照で `get("key")` を使っているか
- [ ] 失敗パスで Result を返しているか（静かなフォールバック禁止）


Error taxonomy (phi decode)
- phi:invalid-object:dst-missing — dst not found in segment.
- phi:invalid-object:value-missing — single-form missing value.
- phi:no-values:empty — values[] present but empty.
- phi:no-values:all-malformed — values[] present but none contain a value.
- phi:no-values — fallback when no incoming could be chosen.

アップデート（2025-10-06）
- `JsonCursorBox` を導入（index_of_from/scan_string_end/seek_array_end/seek_obj_end を提供）。
- 直接スキャン箇所は段階的に `JsonCursorBox` に委譲（例: minivm_probe, step_runner）。
