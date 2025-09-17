# 段階的実装の階段プラン - 壊れない設計

作成日: 2025-08-27
Status: 実装指針
Priority: High
Related: ChatGPT5との共同構想

## 🎯 設計哲学

**段階で積める設計＝強い**し、破綻しにくい

各段階で必ず「合格証（ゲート）」を取ってから次へ進む。いつでも戻せる安全網付き。

## 🏗️ 階段プラン（壊さない順）

### 1. 凍結する契約（小さく）
```
固定する最小限の仕様：
- MIR1の命令表＋効果（pure/read/write/io/atomic）
- 強1本・weak非伝播・@must_drop/@gcableの検証規則
- VM-BC（3アドレス）の呼出規約

→ 1枚のmdに固定（後は後方互換で拡張）
```

### 2. 段階ゲート（各段の合格条件）

#### VM段階
- `interp==vm`（出力/I/Oログ一致）
- `bytes_copied==0`（指定区間）

#### Cranelift v0段階
- `interp==vm==jit`一致
- CLIF verifier pass

#### GC(epoch)段階
- `--gc=on/off`でI/Oログ差分0
- @must_dropのみ監視

#### SyncBox v0段階
- `table.put()`で自動ロック
- `read/with`で借用
- Lintが違反を落とす

### 3. 最適化は「中立」をMIR1に集約

どのバックエンドにも効く系だけ先に：
- canonicalize / CSE / copy-prop / dead-branch（pure域）
- bus-elision（安全条件下）
- weak_load fast-path

### 4. 将来の二段化（MIR2）の入口だけ作る
```
準備だけ先行：
- MIR1→MIR2 loweringスケルトン（例外なし・Box展開だけ）
- MIR2のVerifier（LOCKペア/アライン）だけ先に用意
→ 当面はCraneliftだけMIR2から食わせる
```

### 5. キルスイッチ/保険
```bash
# 怪しい時に戻せるスイッチを常備
--no-elide-bus      # Bus最適化無効
--jit=off           # JIT無効化
--gc=off|epoch      # GC切替
--sync=warn-only    # 同期チェックのみ
```

### 6. CIの黄金テスト（毎回回す）

#### 互換性
```
interp == vm == jit
```

#### 資源管理
```
open == fini(@must_drop)
```

#### 性能指標
```
- ゼロコピー: dup()以外で bytes_copied==0
- 退行防止: send_count/alloc_count が基準以下
```

### 7. メトリクスの見取り図
```
収集データ：
- op_count/hot_bb（VMプロファイル→MIR属性に反映）
- alloc/free・bytes_copied・send/recv
- p95/p99レイテンシ（finiスパイク検知）
```

### 8. 小さなデモを常に1本
```
http_static_zero_copy
- 機能：HTTPサーバーでゼロコピー配信
- ゼロコピー：bytes_copied==0の確認
- GC等価：--gc=on/offで挙動一致

→ これが通れば次の段へ進む合図
```

## 💡 実装の心得

### 複雑さは増えても破綻しない理由
1. **凍結する契約**を小さく決める
2. **各段で合格証**を取る
3. **キルスイッチ**でいつでも戻せる
4. **黄金テスト**で退行を防ぐ

### 最適化の原則
- MIRに集約すれば全ルートに効く
- バックエンド固有は避ける
- 中立最適化を優先

## ✅ 直近の実装順序

1. **今日**: VM基盤の安定化
2. **明日**: Cranelift最小実装
3. **今週**: GC epochフック
4. **来週**: SyncBox MVP
5. **継続**: 黄金テストの拡充

---

*「壊れない設計」- 段階で積めば、複雑さは増えても破綻しない*