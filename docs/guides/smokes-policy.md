# Smokes Policy — Profiles, Noise, and Stability

目的
- 小さな変更で大量に赤くならないよう、スモークの運用を構造で安定化する。

プロファイル運用（基本）
- quick: 開発時の最小・高速（dev向け便利をON、検証は軽め）
- integration-core: VM↔LLVM パリティ（プラグイン無し・ハーネス前提）
- plugins: プラグイン依存の検証（未配置は SKIP）
- integration: apps 系（ハーネス＋VM比較）。AOTリンクはスモーク下でバイパス

出力ノイズの方針
- ランタイム末尾の `Result: <n>` は既定で抑止
  - `NYASH_NYRT_SILENT_RESULT=1`（各プロファイルenvに設定済み）
- ログは stderr に統一（stdout はプログラムの print のみ）
- 新規ノイズ（deprecate/resolve/builder/env）は smokes の filter へ集約

環境の最小主義
- プロファイルenvは「必要最小だけ」を既定ON。実験用フラグはテスト頭でON
- using（開発用）はワンノブで一括ON
  - `source tools/dev_env.sh using`（HAKO_USING=1 / STRATEGY=prelude / ALLOW_FILE=1 / PROFILE=dev）
- すぐ困ったら `tools/ny_doctor.sh` で現状確認

失敗時の再現性
- `SMOKES_CAPTURE=1` で失敗の expected/actual/env を `tmp/smokes_capture/` に自動採取
- `tools/parity_check.sh file.nyash` または `-c 'code'` で VM↔LLVM をその場比較

カテゴリ分離と SKIP
- プラグイン依存は plugins プロファイルへ。未配置は SKIP
- selfhost/using 前提は quick-selfhost / integration-core へ寄せる
- unstable は profiles/unstable/ に分離し、`SMOKES_RUN_UNSTABLE=1` のときのみ実行（将来）

時間/乱数の取り扱い
- Timer/時間は単調性/範囲で assert（具体値比較は避ける）
- 非決定順序（map/列挙）は正規化（sortや安定化）して比較

PHI の Fail‑Fast（開発時）
- `NYASH_VERIFY_PHI_STRICT=1` で PHI inputs が到達可能なすべての predecessor をカバーするか検証（InvalidPhi）
- 片側Only/ネスト else-if を quick/core/phi にスモークで常設

ベストプラクティス（テスト記述）
- テスト頭で必要な env を閉じ込める（プロファイルに広げない）
- 新規ログは stderr へ、stdout に混ぜない
- ノイズ差分は test_runner の filter に追加（バラバラなgrepを増やさない）

