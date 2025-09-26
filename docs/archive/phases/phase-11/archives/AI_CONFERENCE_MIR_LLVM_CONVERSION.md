# AI会議：MIR→LLVM変換の技術的アプローチ

Date: 2025-08-31  
Participants: Claude, Gemini, Codex

## 🎯 統合された実装戦略

### 1. BoxCall最適化（PIC実装）

#### Gemini先生の提案
- **メソッドID（スロット）ベース** + **PIC（Polymorphic Inline Cache）** の組み合わせ
- 静的解析で解決できる場合は直接呼び出し
- 動的な場合はPICでキャッシュ

#### Codex先生の具体的実装
```llvm
; グローバルPIC（モノモルフィック例）
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
  %r2 = call coldcc %RetTy @nyash_pic_miss_foo(%ObjTy* %recv, i64 %method_id, ...)
  br label %cont
```

### 2. 脱箱化戦略

#### Gemini先生の提案
- MIRレベルの最適化パスで実施
- 型推論と注釈の活用
- データフロー解析に基づく安全な範囲の特定

#### Codex先生の2表現SSA戦略
```llvm
; 算術は全てプリミティブ（i64）で
%a = add i64 %x, %y

; 必要になった地点でのみBox化
%box = call %ObjTy* @nyash_make_int(i64 %a)
call void @vector_push(%Vec* %v, %ObjTy* %box)
```

**エスケープ解析との連携**：
- 未エスケープ＋GCセーフポイントを跨がない → 完全Box削除
- 条件付きエスケープ → ブランチ内で遅延Box化

### 3. GCバリア最小化

#### 世代別GCでの最適化（両先生の統合案）

```llvm
; TLSにNursery境界を保持
@nyash_tls_nursery_low = thread_local global i64 0
@nyash_tls_nursery_high = thread_local global i64 0

; Write barrier（インライン化されたfast path）
store i8* %val, i8** %slot
%obj_i = ptrtoint i8* %obj to i64
%val_i = ptrtoint i8* %val to i64
%low = load i64, i64* @nyash_tls_nursery_low
%high = load i64, i64* @nyash_tls_nursery_high
%yo_obj = and (icmp_uge %obj_i, %low), (icmp_ult %obj_i, %high)
%yo_val = and (icmp_uge %val_i, %low), (icmp_ult %val_i, %high)
%need_barrier = and (not %yo_obj), %yo_val   ; 老→若のみ
%likely0 = call i1 @llvm.expect.i1(i1 %need_barrier, i1 false)
br i1 %likely0, label %barrier, label %cont, !prof !{!"branch_weights", 1, 10000}
```

### 4. 注釈→LLVM属性変換

#### 安全性重視の段階的アプローチ

| Nyash注釈 | LLVM属性 | 安全性条件 |
|-----------|----------|------------|
| `@no_escape` | `nocapture` | エスケープしないことを静的証明 |
| `@pure` | `readonly` | 副作用なしを保証 |
| `@pure` + 強条件 | `readnone speculatable` | メモリ不読＋例外なし |
| `@nonnull` | `nonnull` | NULL不可を型システムで保証 |
| `@range(0,255)` | `!range` | 値域制約をメタデータ化 |

### 5. LLVM最適化パス構成

#### 推奨パイプライン（両先生の合意）

```
Phase 1（基本最適化）:
  mem2reg → instcombine → gvn → sccp

Phase 2（Nyash特化）:
  BoxCall devirtualization → inline → SROA（Box消去）

Phase 3（高度な最適化）:
  licm → indvars → loop-unroll → vectorize

Phase 4（最終調整）:
  Box materialization cleanup → instcombine
```

### 6. インライン展開戦略

#### コストモデル（Codex先生）
- モノモルフィックPIC＋高ヒット率（>90%） → インライン
- コード膨張はプロファイル重みで正規化
- 再帰最適化：`musttail`によるTCO、部分インライン化

## 🚀 実装ロードマップ

### Week 1: 基礎構築
- [ ] inkwellセットアップ
- [ ] 基本的な15命令→LLVM IR変換
- [ ] 最小実行可能コード生成

### Week 2: PIC実装
- [ ] モノモルフィックPIC
- [ ] ポリモルフィックPIC（2-4 way）
- [ ] Megamorphic fallback

### Week 3: 脱箱化＋GC統合
- [ ] 2表現SSA実装
- [ ] エスケープ解析
- [ ] GCバリア最適化
- [ ] gc.statepoint統合

### Week 4: 最適化＋検証
- [ ] 注釈→属性変換
- [ ] カスタム最適化パス
- [ ] ベンチマーク検証
- [ ] 安全性テスト

## 💡 重要な洞察

### Gemini先生
> 「Everything is Box」モデルのオーバーヘッドを削減する鍵が脱箱化です。早すぎる脱箱化は再Box化のコストを生み、遅すぎると最適化の機会を逃します。

### Codex先生
> PIC更新の安全化: 1-ワードのバージョンでRCU風プロトコル。ABI二系統（Box ABI/Primitive Fast-ABI）をIRBuilderに明示し、境界でのみmaterialize_box/dematerialize_boxを発行。

## 🎉 結論

両先生の知見を統合することで、「Everything is Box」の柔軟性を保ちつつ、C++に迫る性能を実現する具体的な実装パスが明確になりました。特に：

1. **PICによる動的最適化**と**静的型推論**の組み合わせ
2. **遅延Box化**による不要なヒープ割り当ての削減
3. **世代別GC**と**インラインバリア**の協調
4. **保守的な属性付与**による安全性確保
5. **段階的最適化パイプライン**による着実な性能向上

これらにより、Nyashは「シンプルな15命令」から「高性能な機械語」への変換を実現できます。