# Codex先生のNyash ABI実装戦略 (2025-09-01)

## 質問内容

Nyashプラグインシステムの ABI戦略について技術的相談です。

【背景】
Phase 12でNyashスクリプトプラグインシステムを実装中です。現在、プラグインインターフェースの設計で重要な判断が必要です。

【現状】
- 既存: C ABI（シンプル、高速、実績あり）
- 提案: Nyash ABI（3×u64構造体、型情報付き、拡張性高い）

【トレードオフ】
C ABI:
- 利点: ゼロオーバーヘッド、既存資産活用、シンプル
- 欠点: 型情報なし、拡張性限定、エラー処理が原始的

Nyash ABI:
- 利点: 型安全、メタデータ活用、将来性（async等）
- 欠点: pack/unpackのオーバーヘッド、新規学習コスト

【具体的な選択肢】
1. Nyash ABIのみ（統一性重視）
2. C ABI + Nyash ABI両方（実用性重視）
3. その他の革新的アプローチ？

【質問】
1. どちらの戦略が長期的に見て賢明でしょうか？
2. 両方サポートする場合の複雑性は管理可能でしょうか？
3. 他の言語での成功例・失敗例から学べることは？
4. VM/JIT最適化の観点から、どちらが有利でしょうか？

実装の実践的観点から、C ABIとNyash ABIの選択について深く分析してください。特にVM/JIT最適化や実装の複雑性を考慮して。

## Codex先生の回答

**結論**
- 長期的には「C呼出規約 × Nyash値表現（3×u64）」を中核に、C ABIとNyash ABIの両方をサポートする二層戦略が最も堅実。
- まずは二者併存で運用し、JIT/VMでの最適化余地と実用性を確保しながら、将来的にNyash ABIを第一級に育てるのが現実的。

**推奨戦略**
- 両対応（2）を採用。ただし"呼出規約はC、値表現はNyash"を共通土台にする。
  - 共通の値ABI: `#[repr(C)] struct NyValue { u64 tag; u64 payload; u64 aux; }`
  - C ABI関数でもNyValueを受け渡しできるため、型安全・メタデータ拡張の道を確保しつつ、Cの安定性・高速性を得る。
- プラグイン初期化は「バージョン付き関数テーブル（vtable）＋機能フラグ」の交渉型にする。
  - 例: `NyPluginInitV1(NyRtFnsV1* rt, NyPluginV1* out)` → `NyPluginV1 { u32 abi_mask; u32 size; NyFnDesc* fns; ... }`
  - `abi_mask`で `NY_ABI_C` と `NY_ABI_NYV1` の両方可否を示す。将来の `NY_ABI_NYV2` も追加可能。
- エラーモデルは二段階:
  - 速経路: `NyStatus`（enum, small int）を戻す。失敗時は`NyValue`でエラーオブジェクトをout-param返却。
  - 詳細情報は`NyError`ヘルパ（`NyValue`として表現）に格納。FFI境界でpanic禁止・例外横断なし。
- メモリ/所有権はハンドル/関数テーブル経由で統一（NyRTのalloc/free、string/slice作成API）。内部構造体への直接依存は不可。

**複雑性と管理**
- 追加コストは主にローダとアダプタ（トランポリン）実装＋テスト行列の増加。ただし次の工夫で管理可能:
  - 機能交渉を一本化（init関数とマニフェスト/nyash.tomlで表明）。ロード時に一度だけ決定。
  - 呼出アダプタをテンプレート化／JIT時に生成・キャッシュ（型配列→シグネチャ→thunkアドレス）。
  - 型表（`NyFnDesc`に引数/戻りの型ID配列、純関数/readonly/aliasing属性）を持ち、最適化と検証を兼用。
- 見積り：ローダ＋vtable＋アダプタの骨格は数百行〜千行弱で収まり、以降は関数種別追加が足し算。

**他言語の事例からの学び**
- 成功例
  - Node N-API: バージョン化された関数テーブルでABIを安定化。実装側の進化と互換の両立に有効。
  - Lua C API: C呼出規約＋独自値表現で長期安定。ただしstackベースは学習コストあり。
- 反省点
  - CPython C API: 内部構造体露出で進化の足枷に。HPyはハンドル化で再設計。
  - JNI: 汎用だが冗長・低速傾向。最適化しづらい。
- 教訓
  - 内部レイアウト非公開（ハンドル/関数経由）・バージョン化vtable・拡張フラグが長期安定の鍵。
  - 値ABIは固定サイズ・レジスタ渡し最適化可能な形が吉（3×u64は合理的）。

**VM/JIT最適化の観点**
- 呼出オーバーヘッド
  - 3×u64はSysV/Win64いずれもレジスタ渡しが効きやすく、C ABIと同等のコールコストに収束。
  - 本質的ボトルネックは間接呼出ミス予測と型検査。JITで型が既知ならチェックを埋め込み/削減可能。
- 直呼 vs トランポリン
  - 署名が一致する場合は直接呼出（JITに外部シンボルとして登録）。
  - 型昇降/借用が必要な時のみthunkを噛ませる。JITでモノモーフィック化・デッドパス除去。
- 最適化ヒント（Cranelift/LLVM）
  - 関数属性: `nounwind`, `readonly`/`readnone`, `nocapture`, `nonnull`, `cold`（エラーパス）
  - 戻り値はout-paramにしてtailcall最適化（`musttail`適用可否は要検証）。
  - シンボルはAOT時`dso_local`、JITは`CallConv::SystemV/WindowsFast`で3×i64に整合。
- ホットループ対策
  - スカラー跨ぎは避け、スライス/バルクAPIを定義（例: vector add）。boundary跨ぎ回数を削減。
  - 将来は静的リンクモード（first-party plugins）でLTO許可→AOTでインライン・DCE可。

**Nyash ABIのみ vs 併存の比較**
- Nyash ABIのみ（1）
  - 一貫性と将来性は高いが、既存C資産/ツール連携コストと立上げ摩擦が大きい。
- 併存（2, 推奨）
  - 早期エコシステム拡大と最適化の両取り。複雑性はvtable＋アダプタの設計で抑え込み可能。
- その他（3）
  - N-API風ハンドル専一は安定するが、オーバーヘッド/冗長性が増える。Nyashのパフォーマンス志向には過剰。

**実装メモ（具体）**
- 値ABI
  - `tag`: 上位ビットに種別・フラグ（own/borrow、small/heap、nullable等）
  - `payload`: ポインタor即値、`aux`: len/stride/arena_id等
  - `NySlice`: `tag=SLICE`で`payload=ptr`, `aux=len`。borrow時は生存期間制約、必要なら`nyrt_clone_slice`で所有権取得。
- 初期化/交渉
  - `NyPluginInitV1(NyRtFnsV1*, NyPluginV1*) -> NyStatus`
  - `NyRtFnsV1`: alloc/free、string/slice/future、log、error造作API。
  - `NyPluginV1`: `abi_mask`, `size`, `features`, `fn_count`, `NyFnDesc*`.
- エラー/非同期
  - `NyStatus` + `NyValue err_out`。非同期は `NyFuture*`を返却、`NyRtFnsV1`で`poll/wake`を橋渡し。
  - VMのスケジューラとWakerのブリッジはvtableで隠蔽（ABI非依存）。
- 安定性
  - すべての公開構造体に`size`を先頭フィールドに持たせ将来拡張。未理解フィールドは無視。
  - ABIバージョンは数値＋機能フラグで条件分岐（ビルド時にも`#[cfg(feature)]`で絞る）。

**質問への回答**
- 1) 長期的に賢明か: 併存＋共通値ABIが最善。Cの互換性とNyashの型安全/拡張性を両立。
- 2) 複雑性は管理可能か: はい。vtableの一本化、JIT生成thunk、機能交渉で抑制できる。
- 3) 学び: 内部構造体を外部に晒さない・ハンドル経由・バージョン化テーブル・豊富な属性メタデータ。CPythonの反省とN-APIの成功を取り込む。
- 4) VM/JIT最適化: 3×u64のNyash値はレジスタ渡しでC ABIと同等に速く、型既知箇所はJITで直呼/チェック省略可能。最適化の自由度はNyash ABIの方が高い。

**次のアクション**
- 値ABIと関数テーブル仕様を`crates/nyrt/include/nyrt.h`に最小ドラフト化（`NyValue`, `NyStatus`, `NyRtFnsV1`, `NyPluginV1`, `NyFnDesc`）。
- ローダにinit/交渉実装（`nyash.toml`のabi種別・型情報読み込み→関数登録）。
- JIT/VMに外部関数署名登録とthunk生成を追加（Cranelift/LLVM兼用）。
- バルク演算プラグインのスモークを作成（C ABI版とNyash ABI版を比較ベンチ）。
- ドキュメント化（`docs/plugins/abi.md`）とサンプル（`plugins/`、`apps/`）追加。

必要なら、ドラフトの`NyValue`と`NyPluginInitV1`の最小Cヘッダ案もすぐ出します。