Nyash ネイティブビルド計画（Native Plan）

- 目的: 開発者向けに「ビルド計画・段階的タスク・設計上の要点」を集約。
- 利用者向けの具体的なビルド手順は docs/説明書/native-build/README.md を参照。

重要リンク
- 現在のタスク: docs/CURRENT_TASK.md
- コア概念（速習）: docs/nyash_core_concepts.md
- フェーズ課題一覧: docs/予定/native-plan/issues/
- 相互参照: docs/予定/native-plan/copilot_issues.txt（Phase 0–10 の下書き）

要点サマリ（統合）
- ビルド方針
  - デフォルトは CLI 最小構成（`cargo build --bin nyash`）。GUI/Examples は feature で任意有効化。
  - Windows ネイティブ: MSVC または WSL + cargo-xwin によるクロスコンパイルを推奨。
- MIR/VM の段階的導入
  - Phase 5.2: `static box Main` → MIR への lowering 経路を実装済み。
  - Phase 6: 参照/弱参照の最小命令（RefNew/RefGet/RefSet, WeakNew/WeakLoad, BarrierRead/Write=no-op）。
  - 例外/Async は薄く導入、先に snapshot/verify の安定化を優先。
- 弱参照の意味論（実装で壊れにくく）
  - WeakLoad は Option<Ref> を返す（生存時 Some、消滅時 None）。PURE 扱い（必要に応じ READS_HEAP）。
  - `fini()` 後の使用禁止・weak 自動 null・cascading 順序（weak はスキップ）を不変として扱う。
- Safepoint と Barrier
  - 関数入口・ループ先頭・呼出直後に safepoint。Barrier は最初は no-op 命令として実装可。
- テスト戦略
  - 黄金テスト：ソース→MIR ダンプのスナップショットで後退検出。
  - VM/JIT 一致：同入力で VM と JIT の結果一致（将来の AOT でも同様）。
  - 弱参照の確率テスト：alloc→weak→drop→collect→weak_load の順序/タイミングを多様化。

進行フェーズ（抜粋）
- Phase 0: CLI 最小ビルド安定化（Linux/Windows）。
- Phase 5.2: static Main lowering（実装済み）。
- Phase 6: 参照/弱参照（Barrier は no-op で開始）。
- Phase 7: nowait/await（スレッドベース、FutureBox 連携）。

アーカイブ（長文メモ・相談ログ）
- docs/予定/native-plan/archive/chatgptネイティブビルド大作戦.txt
- docs/予定/native-plan/archive/追記相談.txt
  - 上記2ファイルの要点は本 README に統合済み。詳細経緯はアーカイブを参照。

備考
- デバッグ補助: `--debug-fuel` でパーサーの燃料制御。`--dump-mir`/`--verify` で MIR の可視化・検証。
- 一部の開発用ログ出力（/mnt/c/...）は存在しない環境では黙って無視されます（問題なし）。
