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
