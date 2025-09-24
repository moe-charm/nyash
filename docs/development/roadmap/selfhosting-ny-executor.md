# 自己ホスティング Ny 実行器（Nyash→MIR→Ny 実行）計画

更新日: 2025-09-22

## 目的
- 既存の Python PyVM を参照器として維持しつつ、Nyash 自身で MIR(JSON) を実行できる最小実行器（Ny Executor）を段階的に立ち上げる。
- LLVM ライン/既存 CI を壊さず、機能追加ポーズの原則（仕様不変・既定OFF）を守る。

## 原則
- 意味論の決定は MIR 側。PHI は MIR が決め、llvmlite は IR の phi を具現化するだけ。
- 小粒・ガード付き。新経路は既定OFFのフラグで明示、ロールバック容易に。
- パリティ最優先。PyVM/LLVM をオラクルにして、ゴールデンとスモークで一致を確認。
- 観測可能性（TRACE）と安全弁（STEP_MAX）。

## フラグ（既定OFF）
- `NYASH_SELFHOST_EXEC=1`: Ny Executor を有効化（PyVM の代わりに Ny スクリプトへ委譲）。
- `NYASH_SELFHOST_TRACE=1`: Ny Executor の構造化ログ（JSON lines or 整形文字列）。
- `NYASH_SELFHOST_STEP_MAX=<int>`: 1 実行あたりの最大命令数（既定 200000 相当）。
- `NYASH_SELFHOST_STRICT=1`: 厳格モード（型/値のチェックを強化、未知 extern を拒否）。
- 参考: `NYASH_MIR_NO_PHI=1`（開発用。既定PHI-onからレガシー edge-copy 経路へ切り替えるときに使用）

## 構成（新規 Ny ファイル）
- `apps/selfhost-runtime/`
  - `mir_loader.nyash`: MIR(JSON v0) ローダ
  - `ops_core.nyash`: const/binop/compare/branch/jump/ret/phi
  - `ops_calls.nyash`: call/externcall/boxcall（MVP）
  - `boxes_std.nyash`: String/Array/Map/Console の最小メソッド
  - `runner.nyash`: エントリ/ディスパッチ/ステップガード/TRACE

Rust ランナー側は PyVM 経路にて `NYASH_SELFHOST_EXEC=1` を検出した場合のみ Ny Executor に MIR(JSON) を渡す（既定は従来どおり PyVM）。

## ステージ計画と受け入れ基準

### Stage 0 — スカフォールド（1–2日）
- フラグ/エントリ配線のみ。Ny Executor は no-op のまま 0 で終了。
- 受け入れ: ビルド緑・既定挙動不変・フラグONで no-op 実行可。

### Stage 1 — MIR ローダ（2–3日）
- `mir_loader.nyash` で JSON v0 を読み込み、関数/ブロック/命令の構造体に展開（最初は要約のみ）。
- 受け入れ: ロードのみのスモーク（構文要素の個数検証）。
- 備考: 立ち上げ初期は PyVM ハーネス用 MIR JSON（`{"functions":…}`）も受理し、要約（functions数）だけ行う（既定OFF）。

### Stage 2 — コア命令（3–5日）
- `ops_core.nyash` で `const/binop/compare/branch/jump/ret/phi` を実装。
- `runner.nyash` にステップ budget と TRACE を実装。
- 受け入れ: 小さな MIR プログラム群で PyVM と stdout/exit code が一致。

### Stage 3 — call/externcall/boxcall（4–7日）
- `ops_calls.nyash` で関数呼び出し/外部呼び出し/Box メソッド呼び出し（MVP）を実装。
- String/Console の最小メソッド揃え。未知 extern は STRICT=1 で拒否。
- 受け入れ: 既存の小スモーク（文字列・print 系）でパリティ緑。

### Stage 4 — コレクション/JSON アダプタ整合（4–7日）
- Array/Map の最小メソッド（size/len/get/set/push 等）を Ny 側で実装。
- 受け入れ: 既存 `apps/tests/` のコレクション系スモークでパリティ緑。

### Stage 5 — using/seam の安定化（3–5日）
- using 解決は従来どおり Rust 側。Ny Executor は MIR(JSON) のみを消費。
- seam ガード（`NYASH_RESOLVE_FIX_BRACES`/`NYASH_RESOLVE_DEDUP_BOX`）ON で代表例を通す。
- 受け入れ: using 混在スモークでハングなし・出力一致。

### Stage 6 — パリティハーネス/CI（5–7日）
- `tools/parity.sh --lhs selfhost --rhs pyvm` を追加。
- CI に非ブロッキングで Ny Executor ジョブを追加（緑安定後にブロッキング昇格）。
- 受け入れ: 代表セットで selfhost=PyVM（必要環境で LLVM もオプション比較）。

### Stage 7 — 速度/診断/堅牢化（継続）
- ホットパス最適化、TRACE 圧縮、エラーメッセージ充実、境界ケーステストの拡充。

## ロールバック/安全策
- 既定は PyVM。`NYASH_SELFHOST_EXEC=1` を明示しなければ Ny Executor は有効にならない。
- すべての追加は最小差分・既定OFFで導入。戻しはフラグOFFで即時原状復帰可能。

## リスクと対策
- MIR JSON スキーマの変化: ローダでバージョンを検査。未知フィールドは安全に無視または明示的に失敗。
- PHI/ループの隅: MIR が意味論を決める（PHI or Edge Copy）。PyVM/LLVM とゴールデンで監視。
- Boxcall 乖離: MVP のみ実装。STRICT で未知を拒否。スモークで毎回比較。

## タイムライン（概算）
- Stage 0–1: 3–5日 / Stage 2: 3–5日 / Stage 3–4: 8–14日 / Stage 5–6: 8–12日 / Hardening: 継続

## 受け入れ指標（簡易）
- stdout/exit code の一致、ハングなし、TRACE（任意）で差分ゼロ。
- CI 緑維持（既存ジョブ不変）+ 新規 selfhost ジョブは当初非ブロッキング。
