# Quick Smokes — 開発系のゲート方針（ENV で opt‑in）

目的
- quick プロファイルは「高速・安定」を最優先にする。
- 開発途上のスモーク（Pipeline V2 / LocalSSA 等価性など）は、環境変数で opt‑in した時だけ実行する。

ゲート変数
- `NYASH_PIPELINE_V2=1`
  - Pipeline V2 系の自動生成スモークを有効化する。
  - 既定では SKIP（未整備環境でも quick を緑に保つ）。
- `NYASH_LOCALSSA_ENABLE=1`
  - LocalSSA 等価性スモーク（ensure_cond の前後一致）を有効化する。
  - 既定では SKIP（Mini‑VM の機能が最小構成のため）。

補足
- 代表の core/router（Timer/Array/Map）や LLVM ハーネス系は常時有効（ハーネス未検知時は SKIP）。
- ゲートの有効化はテスト対象が増えるため、quick の総時間に影響することがある。

---

## Core Kernel Baseline（Quick の既定ルール）

目的
- quick は「コア常在・依存最小」で揺れを無くす。プラグインや using/new（スクリプト定義）に依存しない。

ルール
- `NYASH_DISABLE_PLUGINS=1` を基本にする（プラグイン未整備でも緑を保つ）。
- Box の動作検証は「静的カーネル面」を叩く。
  - 例: `TimerBox.now_ms()` を優先（`new TimerBox()`/using に依らない）。
- 文字列/数値の混在 print は避け、判定は固定文字列（`ok`/`ng`）で行う。
- 実行不可な環境では SKIP を徹底（プローブ後に SKIP 理由を明示）。

開発トレース（任意）
- `NYASH_STATIC_CALL_TRACE=1` … Builder の経路観測（Extern 直行など）
- `NYASH_VM_TRACE=1` … VM ExternAdapter 呼び出し観測

備考
- Router/Adapter の拡大は dev ゲート（opt‑in）で段階導入。常時系は「READ/ゼロ引数・安全」から 1 件ずつ。
