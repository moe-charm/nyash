# Phase 10.6a — Thread-Safety Audit (Checklist)

目的: NyashBox/ランタイムのスレッド安全性を棚卸しし、将来の並列化（10.6b/c以降）に備える。

## 方針
- 既定は単一スレッド実行（VM/Interpreter）。並列化は opt-in。
- 共有状態は `Arc<...>` と `RwLock/Mutex` により内的可変を確保。
- クロススレッド境界に出る型は `Send + Sync` を満たす（必要に応じてラッパで担保）。

## チェックリスト
- Box実装（src/boxes/*）
  - [ ] 共有内部状態を持つ型は `Arc<RwLock<_>>` のようにラップされているか
  - [ ] `to_string_box()` が重い処理やグローバル可変に依存しないか
  - [ ] FFI/プラグイン橋渡し時に非同期イベント/コールバックを保持しないか（保持する場合は送受戦略を文書化）
- ランタイム（src/runtime/*）
  - [ ] `NyashRuntime` のメンバは `Send + Sync` 要件を満たす（`Arc<...>`）
  - [ ] `GcHooks` 実装は `Send + Sync`（CountingGc/NullGc はOK）
  - [ ] Scheduler 実装は `Send + Sync`（SingleThreadSchedulerはOK）
- VM/Interpreter
  - [ ] MIR `Safepoint` で `runtime.scheduler.poll()` を呼ぶ（協調スケジューラの結合点）
  - [ ] Grep: `rg -n "Safepoint" src` で配置確認

## Grep支援
```bash
rg -n "Arc<|Mutex<|RwLock<|Send|Sync" src/boxes src/runtime
```

## 既知の注意点
- Python/外部DLLとの橋渡しはGIL/PATH管理で単一スレッド優先（AOT時はPATH/PYTHONHOME調整済）。
- BufferBox は共有化のために `Arc<RwLock<Vec<u8>>>` を採用済み。

## クイック監査（第一次）
- ArrayBox: `Arc<RwLock<Vec<Box<dyn NyashBox>>>>` → OK（共有＋内的可変）
- MapBox: `Arc<RwLock<HashMap<String, Box<dyn NyashBox>>>>` → OK
- BufferBox: `Arc<RwLock<Vec<u8>>>` → OK
- NyashRuntime: `box_registry: Arc<Mutex<_>>`, `box_declarations: Arc<RwLock<_>>`, `gc: Arc<dyn GcHooks>`, `scheduler: Option<Arc<dyn Scheduler>>` → OK
- Scheduler: `SingleThreadScheduler` 内部に `Arc<Mutex<VecDeque<...>>>` → OK
- GC Hooks: `NullGc/CountingGc` は `Send+Sync` 実装方針 → OK

未確認/注意:
- プラグインBox（PluginBoxV2）の内部FFIハンドルはVM/EXE側で共有参照のみ（実体はFFI側）。クロススレッド呼出しは未サポート運用（明記要）。
- 一部のBoxで外部資源（ファイル/ネット）を扱う場合、スレッド越境のI/O同期設計は別途（Phase 10.6d+）。

## 次の一手（提案）
- マーカーTraits（例: `ThreadSafeBox`）の導入は保留（破壊的）。現時点は監査＋ドキュメントで運用。
- 並列スケジューラ（M:N）の実装は `feature` フラグで段階導入。
