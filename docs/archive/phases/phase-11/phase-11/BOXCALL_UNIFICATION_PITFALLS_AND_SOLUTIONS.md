# BoxCall統一の落とし穴と対策（ChatGPT5分析）

Date: 2025-08-31
Status: Technical Advisory
From: ChatGPT5

**結論：「RefNew/RefGet/RefSet全削除→すべてBoxCallに統一」は成立する！**
ただし、いくつかの落とし穴があるので、それぞれに対策を打つ必要がある。

## 🚨 落とし穴と対策

### 1. メガモーフィック呼び出しでの失速
**症状**: 同じ`BoxCall("setField")`でも実行時の型/shapeが激しく変わると、ディスパッチが重くなる。

**対策**: **PIC（Polymorphic Inline Cache）**をコールサイトごとに持つ
- 2〜4種のshapeを直列ジャンプで捌く
- 溢れたらインタプリタ/汎用スローへ
- JITなしでもAOT段階で形状統計から事前特化（事前ガード＋直アクセス）を埋め込める

### 2. GCバリアの見落とし・過剰挿入
**症状**: write barrier忘れ＝世代間参照漏れ／逆に全部に入れて過剰オーバーヘッド

**対策**: 
- Lowering時に**フィールドの"ポインタ／非ポインタ"メタ**を参照して自動挿入
- **世代同一・同アリーナ最適化**でbarrier省略
- `ExternCall`には**境界バリア**を必ず付与
- **Barrier Verifier**（IRパス）で「必要箇所に必ず入ってるか」を機械検証

### 3. 読み取りバリア（Read Barrier）が必要なGCを選ぶ場合
**症状**: 動くGC（移動/並行）でread barrierが必須だと、Get系もコスト上がる

**対策**: 
- まずは**世代別・停止＋並行マーク（SATB）**など「write側主体」の方式を選ぶ
- **read barrierなし運用**で始めるのが無難
- 将来read barrierが要る場合は、`getField` Loweringに条件付き埋め込み設計

### 4. 例外・再入・ファイナライザ再入
**症状**: `setField`中に例外→ファイナライザ→別の`BoxCall`で再入…地雷

**対策**:
- **安全点（safepoint）設計**を決める
- `BoxCall`中は原則再入禁止（or 明示的許可フラグ）
- `fini`相当のコールは**再入ガード**と**順序保証**（トポロジカルな破棄順）を実装

### 5. ExternCall/FFI境界
**症状**: 外部コードが「未トラッキングの生ポインタ」を握るとGC・最適化が壊れる

**対策**:
- **ハンドル化**（OpaqueHandle/PinBox）＋**寿命契約**
- ExternCallの属性（`noalloc`/`nothrow`/`readonly`/`atomic`等）を宣言させ、最適化に渡す
- 未注釈の呼び出しでは保守的にバリア＆逃避扱い

### 6. 形状（shape）変更とレイアウト安定性
**症状**: フィールド追加/順序変更が既存の特化コードを壊す

**対策**:
- **ShapeIDを永続化**
- フィールドに**安定スロットID**を割り当て
- ABI的に「追加のみ」「削除は新shape」とする
- Lowering済みのガードは `if (shape==X) { direct store } else { slowpath }` で守る

### 7. 脱箱（unboxing）とコードサイズ膨張
**症状**: 激しいモノモルフィック特化や整数Boxの脱箱で**コード肥大**

**対策**:
- **基本型はSROA/Scalar-Replaceの閾値**を設定
- ホット領域のみ特化（**PGO**やプロファイル使用）
- 低頻度パスは共通スローに集約

### 8. 並行性・メモリモデル
**症状**: `setField`の可視性がスレッド間で曖昧だと事故

**対策**:
- **既定は単一スレッド＋Actor（Mailbox）**に寄せる
- 共有可変を解禁するAPIは `nyash.atomic.*` で**Acquire/Release**を明示
- `BoxCall` Loweringで**必要時のみフェンス**
- 箱ごとに「可変・不変・スレッド送受可」など**能力（capability）ビット**を持たせ最適化条件に使う

### 9. 反射・動的呼び出しの混入
**症状**: なんでも動的だと最適化が崩れる

**対策**:
- 反射APIは**分離名前空間**に押し込める
- 既定は静的解決できる書き方を推奨ガイドに
- 反射使用時は**deoptガード**を挿入

## 📈 推奨の最適化パイプライン（AOT想定）

1. **型/shape解析**（局所→関数間）
2. **BoxCall脱仮想化**（モノ/ポリモーフィック化＋PIC生成）
3. **インライン化**（属性`pure`/`leaf`/`readonly`を最大活用）
4. **SROA/エスケープ解析**（脱箱、stack allocation、alloc移動）
5. **バリア縮約**（世代同一・同アリーナ・ループ内集約）
6. **境界チェック消去**（`length`不変式の伝播）
7. **ループ最適化**（LICM, unroll, vectorize）
8. **DCE/GVN**（Getter/Setter副作用ゼロなら畳み込み）
9. **コードレイアウト**（ホット先頭、コールド折り畳み）
10. **PGO（任意）**でPIC順序・インライン閾値を再調整

## 🔧 Loweringの骨格（フィールド書き込みの例）

```llvm
; High-level
obj.setField(x)

; Guarded fast-path（shapeが既知＆最頻）
if (obj.shape == SHAPE_A) {
    ; slot #k に直接store
    store x, [obj + slot_k]
    call gc_write_barrier(obj, x)   ; 必要なら
} else {
    ; PICの次候補 or 汎用ディスパッチ
    slow_path_setField(obj, x)
}
```

- `gc_write_barrier`はIR上は呼び出しに見せておく（後段で**インライン**→**条件付きno-op化**可能）
- `read barrier`が要らないGCなら`getField`は**loadのみ**に落ちる

## ✅ 実装チェックリスト（まずここまで作れば盤石）

- [ ] **Boxメタ**: shapeID、安定スロットID、ポインタ/非ポインタビット、可変/不変、送受可
- [ ] **BoxCall Lowerer**: 形状ガード→直アクセス or 汎用ディスパッチ
- [ ] **PIC**: コールサイトごとに最大N件キャッシュ＋統計（ヒット率/退避回数）
- [ ] **Barrier Verifier**: IR後段でwrite barrier必須箇所を自動検証
- [ ] **Extern属性**: `noalloc/nothrow/readonly/atomic`等を宣言・強制
- [ ] **逃避解析**でstack-alloc/arena-alloc
- [ ] **反射API分離**とdeoptガード
- [ ] **PGOフック**（簡易でOK）：shape頻度、PICヒット率、inlining成果を記録
- [ ] **ベンチ群**:
  - Field get/set（mono vs mega）
  - Vec push/pop / Map ops
  - 算術（IntBoxの脱箱効果）
  - ExternCall（`atomic.store`/`readonly`）
  - GCストレス（大量生成＋世代越し参照）

## 🎯 「簡単すぎて不安」への答え

- **正しさ**は「Lowering＋Verifier」で機械的に守る
- **速さ**は「PIC→インライン→脱箱→バリア縮約」で作る
- **拡張性**は「Everything is Box」の上に**属性**と**能力（capability）**を積む
- Ref系は**公開APIからは消す**が、**デバッグ用の隠しIntrinsic**として温存しておくと計測や一時退避に便利（将来の最適化検証にも効く）

## 🌟 結論

**落とし穴はあるけど全部"設計パターン"で踏まないようにできる**。

にゃーの「箱理論」、素朴だけど正しい地形を踏んでるにゃ。ここまでの方針なら**AOTでも十分に速い**ところまで持っていけるはず。

次は **PIC＋Barrier Verifier＋小ベンチ**の3点を先に入れて、体感を固めに行こう！

---

## 関連文書
- [BOX_SSA_CORE_15_FINAL_DECISION.md](../phase-11.5/BOX_SSA_CORE_15_FINAL_DECISION.md)
- [MIR_TO_LLVM_CONVERSION_PLAN.md](MIR_TO_LLVM_CONVERSION_PLAN.md)
- [MIR_ANNOTATION_SYSTEM.md](MIR_ANNOTATION_SYSTEM.md)