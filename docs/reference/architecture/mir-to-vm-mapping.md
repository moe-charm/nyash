# MIR → VM Mapping (Draft)

最終更新: 2025-08-23

目的: 生成されたMIR命令がVMでどう実行されるかを1枚で把握し、欠落や暫定実装を洗い出す。26命令ダイエット検討の足場にする。

記法: 状態 = Implemented / Partial / No-op / TODO。

## コア命令
- Const: Implemented
  - 定数を `VMValue` に格納。
- BinOp: Partial
  - Integer: Add/Sub/Mul/Div 実装（Divは0割チェック）。他の算術/論理/ビット演算は TODO。
  - String: `+` 連結のみ対応。その他は TypeError。
- UnaryOp: Partial
  - `Neg`(int), `Not`(bool) のみ。`BitNot` は TODO。
- Compare: Partial
  - Integer/ String の Eq/Ne/Lt/Le/Gt/Ge 対応。その他型は TypeError。
- Load / Store: Implemented
  - 現状はVM内の値スロット操作（簡易）。
- Copy: Implemented
  - 値コピー＋クラス名/内部参照印の伝播。

## 制御フロー
- Branch / Jump / Return: Implemented
- Phi: Implemented
  - `LoopExecutor` による選択実装（前BB情報を利用）。

## 呼び出し/Box関連
- Call: Implemented
  - 関数名を `Const String` として解決しVM内ディスパッチ。
- BoxCall: Partial
  - InstanceBox: `{Class}.{method}/{argc}` へ降格呼び出し（MIR関数）。
  - PluginBoxV2: cfg(feature="plugins")下でLoader経由invoke（引数: NyashBox配列）。
  - Builtinの簡易ディスパッチ: `StringBox.length/substr/concat`, `IntegerBox.toString/abs` 等の最小対応。
  - birth 特例: user-definedの `birth` はMIR関数へ直呼。
- NewBox: Implemented
  - `runtime.box_registry` から生成。`scope_tracker` に登録。クラス名マップ更新。
- TypeCheck: No-op (常にtrue)
  - TODO: 正式な型チェックに置換。
- Cast: No-op (コピー)
  - TODO: 正式な型変換に置換。

## 配列
- ArrayGet: TODO（一時的に0を返す）
- ArraySet: TODO（現在はno-op）

## デバッグ/出力
- Debug: No-op（性能優先）
- Print: Implemented（`to_string()`して標準出力）
- Nop: No-op

## 例外/安全ポイント
- Throw: Partial
  - 例外値を表示してVMErrorで中断。ハンドラ探索なし。
- Catch: No-op
  - 例外値スロットを `Void` セットのみ。制御遷移の実装は未対応。
- Safepoint: No-op

## 参照/弱参照/バリア
- RefNew / RefGet / RefSet: Partial
  - `object_fields` に簡易格納。`object_class` と `box_declarations` を用いた可視性（public/private）簡易検査あり。
- WeakNew / WeakLoad: No-op相当（通常コピー/取得と同値）
  - TODO: 実際の弱参照生存判定を導入。
- BarrierRead / BarrierWrite: No-op
  - 効果注釈のみ（将来の最適化/並行実行基盤に備えた形）。

## 非同期
- FutureNew / FutureSet / Await: Implemented
  - `boxes::future::FutureBox` を利用し、同期ブロッキングで結果取得。

## 外部呼び出し
- ExternCall: Implemented
  - `runtime::get_global_loader_v2().extern_call(iface, method, args)` にルーティング。Some/Noneで戻り値void扱いも考慮。

---

## 既知の課題（抜粋）
- BinOp/UnaryOp/Compare の型拡張（浮動小数・Bool/Box等）。
- ArrayGet/ArraySet の実装。
- TypeCheck/Cast の正規化（型表現と整合）。
- 例外ハンドリング（Throw/Catchの制御フロー接続）。
- WeakRef/Barrier の実体化（必要性評価の上、命令ダイエット候補）。
- PluginBoxV2 のVM側統合強化（引数/戻り値のTLV全型対応、Handle戻り値→BoxRef化）。

## VM統計（計測）
- `--vm-stats` / `--vm-stats-json` で命令ごとの使用回数と時間(ms)を出力。
- ホット命令抽出によりダイエット候補を定量化。

---

## 実測結果サマリー（初回プローブ）
出所: `local_tests/vm_stats_hello.json`, `local_tests/vm_stats_loop.json`, `simple_math.nyash`

- ループ系（139命令 / 0.158ms）トップ:
  - Const: 25, BoxCall: 23, NewBox: 23, BinOp: 11, Branch: 11, Compare: 11, Jump: 11, Phi: 11, Safepoint: 11
  - 所見: ループloweringで Branch/Jump/Phi/Safepoint が並び、Box初期化とBoxCallが多い。
- Hello系（6命令）: 期待どおりの最小構成（Const/Print/Return中心）。
- simple_math（18命令）: BinOpの使用を確認（整数加減乗除）。

補足:
- Safepoint はMIR上で挿入されるが、VMではNo-op（計測には現れる）。
- NewBox/BoxCall が上位に入るため、命令セットから外すのは不可（コア扱い）。
- Compare/Branch/Jump/Phi は制御フローのコア。26命令の中核として維持が妥当。

## 実測統計（2025-08-23）
出所: vm-stats（正常系HTTP／異常系HTTP／FileBox）

- 正常系HTTP（40命令）
  - BoxCall: 17（42.5%）/ Const: 12（30%）/ NewBox: 9（22.5%）/ Return: 1 / Safepoint: 1
- 異常系HTTP（21命令 = 正常の52.5%）
  - BoxCall: 7（33.3%）/ Const: 6（28.6%）/ NewBox: 3（14.3%）
  - Branch: 1 / Jump: 1 / Phi: 1（エラーハンドリング特有）/ Return: 1 / Safepoint: 1
- FileBox（44命令）
  - BoxCall: 17（38.6%）/ Const: 13（29.5%）/ NewBox: 12（27.3%）/ Return: 1 / Safepoint: 1

設計含意:
- BoxCallが常に最頻出（33〜42%）。呼び出しコスト最適化が最優先。
- Const/NewBoxが次点。定数・生成の最適化（定数畳み込み／軽量生成・シェアリング）が効果的。
- 異常系は早期収束で命令半減。if/phi修正は実戦で有効（Branch/Jump/Phiが顕在化）。

## 26命令ダイエット（検討のたたき台）
方針: 「命令の意味は保ちつつ集約」。代表案：
- 維持: Const / Copy / Load / Store / BinOp / UnaryOp / Compare / Jump / Branch / Phi / Return / Call / BoxCall / NewBox / ArrayGet / ArraySet
- 参照: RefNew / RefGet / RefSet（Weak/Barrierは拡張枠へ）
- 非同期: Await（FutureNew/SetはBox APIへ寄せる案も可）
- I/O: Print は開発モード限定 or ExternCall統合（ExternCall自体はBoxCallへ統合方針）
- 調整: TypeCheck/Cast はVerifier/型系に寄せる（命令から外す or 1命令に集約）
- Debug/Nop/Safepoint: メタ扱い（命令数からは外す）

次ステップ:
- サンプル/テストをVMで実行し、`vm-stats`結果から実使用命令セットを抽出。
- 上記案に対し互換影響を洗い出し、段階移行（エイリアス→削除）を設計。

---

## 26命令ダイエットの指針（実測反映）
- 維持（ホット・コア）: BoxCall / NewBox / Const / BinOp / Compare / Branch / Jump / Phi / Return / Copy / Load / Store / Call
- 実装方針: ExternCallは原則BoxCallへ集約（必要なら限定的に残す）。
- メタ降格: Debug/Nop/Safepoint（ビルドモードで制御）。
- 型系: TypeCheck/Castは折りたたみ or 検証時に処理（1命令に集約も可）。
- 参照/弱参照/バリア: 需要ベースで拡張枠へ（vm-statsに登場しない限りコア外）。

提案（ドラフト）:
- コア候補（例）: Const, Copy, Load, Store, BinOp, UnaryOp, Compare, Jump, Branch, Phi, Return, Call, BoxCall, NewBox, ArrayGet, ArraySet, RefNew, RefGet, RefSet, Await, Print, ExternCall(集約可), TypeOp(=TypeCheck/Cast), 予備2（将来枠）
- 予備はWeak*/Barrierや将来の非同期拡張等に割当（実測で常用化したら昇格）。

---

## E2E更新（VM経由の実働確認）

成功ケース（VM）:
- FileBox.open/write/read: 引数2個のTLVエンコード（String, String）で成功（HELLO往復）
- FileBox.copyFrom(handle): Handle引数（tag=8, size=8, type_id+instance_id）で成功
- HttpClientBox.get + HttpServerBox: 基本GETの往復（ResultBox経由でResponse取得）
- HttpClientBox.post + headers: Status/ヘッダー/ボディをVMで往復確認
 - HttpClientBox.get unreachable: 接続失敗時はResult.Err(ErrorBox)（ローダーがstring/bytesをErrにマップ）
 - HTTP 404/500: Result.Ok(Response)（ステータスはResponse上に保持）

デバッグ小技:
- `NYASH_DEBUG_PLUGIN=1` で VM→Plugin 呼び出しTLVの ver/argc/先頭バイトをダンプ
- Netプラグインの内部ログ: `NYASH_NET_LOG=1 NYASH_NET_LOG_FILE=net_plugin.log`
