# Codex先生のMIR→LLVM技術的深掘り

Date: 2025-08-31  
Source: codex exec コマンドによる技術相談

## 質問内容

MIR15命令でEverything is Box哲学を実現。注釈システムで最適化ヒントを付与。

### 技術的課題
1. BoxCallの最適化（PIC実装の具体的なLLVM IRパターン）
2. 脱箱化戦略（IntegerBox(42) → i64への変換タイミング）
3. GCバリア最小化（世代別GCでの若い世代→若い世代のバリア省略）
4. 注釈→LLVM属性変換（安全性の保証方法）
5. インライン展開（BoxCallのインライン基準）

## Codex先生の技術的実装アプローチ

### 1. BoxCall + PIC（Polymorphic Inline Cache）

#### モノモルフィックPIC IR例
```llvm
; グローバルPIC
@pic_foo_site123 = internal global { i64, i8* } { 0, null }

; ガード + 直呼び
%cid = load i64, i64* %receiver_class_id
%pic_cls = load i64, i64* getelementptr({i64,i8*}, {i64,i8*}* @pic_foo_site123, i32 0, i32 0)
%hit = icmp eq i64 %cid, %pic_cls
%likely = call i1 @llvm.expect.i1(i1 %hit, i1 true)
br i1 %likely, label %fast, label %miss, !prof !{!"branch_weights", i32 10000, i32 1}

fast:
  %callee = load i8*, i8** getelementptr({i64,i8*}, {i64,i8*}* @pic_foo_site123, i32 0, i32 1)
  %fn = bitcast i8* %callee to %RetTy (%ObjTy*, ... )*
  %r = call fastcc %RetTy %fn(%ObjTy* %recv, ...)
  br label %cont

miss:
  ; cold, 非インライン
  %r2 = call coldcc %RetTy @nyash_pic_miss_foo(%ObjTy* %recv, i64 %method_id, ...)
  br label %cont
```

#### PIC更新の安全化
- 1-ワードのバージョンでRCU風プロトコル
- `store atomic i64 ver=odd (acq_rel)`→フィールド更新→`store atomic i64 ver=even (release)`
- リーダは一貫性確認、失敗時はmissへフォールバック

### 2. 脱箱化（Unboxing）戦略

#### 2表現SSA
- MIRで各Box値に「プリミティブSSA（i64）」と「Box化遅延ノード」の二重表現
- `IntegerBox(42)` → 直ちに`i64 42`としてSSA化
- Boxが必要な境界（汎用コンテナ格納、越関数ABI等）直前でのみBox化

#### 実装例
```llvm
; 算術は全て i64
%a = add i64 %x, %y
; 必要になった地点でのみ実体化
%box = call %ObjTy* @nyash_make_int(i64 %a)  ; ここでのみGC対象生成
call void @vector_push(%Vec* %v, %ObjTy* %box)
```

### 3. GCバリア最小化

#### Write barrier IRパターン
```llvm
; slot: i8** への書き込み
store i8* %val, i8** %slot
; TLSにNursery境界を保持
%low = load i64, i64* @nyash_tls_nursery_low
%high = load i64, i64* @nyash_tls_nursery_high
%yo_obj = and (icmp_uge %obj_i, %low), (icmp_ult %obj_i, %high)
%yo_val = and (icmp_uge %val_i, %low), (icmp_ult %val_i, %high)
%need_barrier = and (not %yo_obj), %yo_val   ; 老→若のみ
%likely0 = call i1 @llvm.expect.i1(i1 %need_barrier, i1 false)
br i1 %likely0, label %barrier, label %cont, !prof !{!"branch_weights", 1, 10000}

barrier:
  call fastcc void @nyash_card_mark(i8* %obj, i8** %slot, i8* %val) cold
  br label %cont
```

### 4. 注釈→LLVM属性変換

#### 安全性担保の原則
- 原則：Nyash注釈は「保守的に弱めに」マップ
- 検証不十分なら一段弱い属性を使用

#### マッピング例
| Nyash注釈 | LLVM属性 | 条件 |
|-----------|----------|------|
| `@no_escape` | `nocapture` | エスケープしないことを静的証明 |
| `@pure` | `readonly` | 副作用なしを保証 |
| `@pure` + 強条件 | `readnone speculatable` | メモリ不読＋例外なし |
| `@nonnull` | `nonnull` | NULL不可を型システムで保証 |

### 5. インライン展開戦略

#### BoxCallの基準
- モノモルフィックPICかつヒット率高（>90%）→ インライン
- コストモデル：call/ret + 間接分岐除去 + 逃げないBoxの削除
- メガモルフィック/低ヒット率は非インライン

#### 再帰的Box呼び出し最適化
```llvm
; 自己再帰でTCO
musttail call fastcc %RetTy @callee(%ObjTy* %recv, ...)
ret %RetTy %r
```

## 実装のこつ

1. **PICグローバル**：`dso_local`/`internal`、更新局所性を確保
2. **ABI二系統**：Box ABI/Primitive Fast-ABIを明示
3. **GC統合**：`gc "statepoint-nyash"`を関数定義に付与
4. **最適化ヒント**：`llvm.expect`と`!prof`を併用

## 結論

> 15命令は実装・最適化・GC統合の観点でよく均衡したミニマル核です。Box統一は開発生産性と実装単純性を大きく押し上げますが、性能面のボトルネックは脱箱・AA強化・適切なGC統合（バリア/セーフポイント/原子性）で解消してください。

Codex先生は具体的なLLVM IRパターンとともに、実装可能で保守性の高い設計を提示してくれました。