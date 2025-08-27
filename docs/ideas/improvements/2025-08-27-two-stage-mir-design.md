# 二段MIR設計 - 高レベル最適化と低レベル直写の両立

作成日: 2025-08-27
Status: 構想段階
Priority: Medium
Related: ChatGPT5との共同構想、MIRラウンドトリップ最適化

## 🎯 なぜ二段MIRか？

**問題**: LLVMは高レベル情報を活かせるが、Craneliftは低レベル直写が得意
**解決**: MIR1（高レベル）とMIR2（低レベル）に分離して両方の強みを活かす

## 🏗️ 役割分担

### MIR1（高レベル・意味論保持）
```
【何者】Nyashの意味をそのまま持つIR
- 所有権/weak/効果注釈/GCタグ/例外/同期/Bus
- Box/Vec/Map等の抽象型あり
- SSA + 高レベル例外（Throw/Try）、TailCall

【最適化】全バックエンドに効く中立パス
- Canonicalize、CSE/const-fold/copy-prop（pure域）
- Bus-elision（安全条件下）
- weak_load fast-path、adopt/release短絡

【出力先】LLVM（高レベル情報を属性化）
```

### MIR2（低レベル・選択済み）
```
【何者】命令選択/構造平坦化済みのIR
- ポインタ/整数/浮動/小配列/memrefに降格
- 例外はlanding-padかResultに変換済
- 3アドレス、仮想レジスタ多め

【展開】
- BoxCall→（ロック/atomic/アドレス計算）+ 関数呼び
- 集合型→memcpy/memsetとオフセット計算
- 同期→LOCK_ACQ/REL/Atomic*/Fence

【最適化】低レベル寄り・Craneliftが喜ぶ
- Peephole（load-op-store合成、cmp+br縮退）
- アドレス計算共通化、冗長ゼロ拡張除去

【出力先】Cranelift（CLIFへ直写）/ C（AOTブリッジ）
```

## 📊 パイプライン（並列＆選択）

```
                 ┌───→ Cranelift (JIT/AOT) [MIR2から]
MIR1 ──opt──→ MIR2 ┤
                 └───→ C (AOT bridge)
         └───→ LLVM (AOT/PGO/LTO)         [MIR1から]
```

- LLVMにはMIR1→LLVMで高レベル情報を残す
- Cranelift/CにはMIR2→CLIF/Cで低レベル直写
- 将来、MIR2→LLVMの補助ルートも可能（ただし最適化余地は減る）

## 🔧 具体例：下げ方（MIR1→MIR2）

### MIR1
```
BoxCall r = map.put(k, v)  // write効果
```

### MIR2
```
LOCK_ACQ  rL, map, write
pBase     = FLDADDR map, off_table
pK        = &k;  pV = &v
call nyrt_map_put(pBase, pK, pV)
LOCK_REL  rL
```

### Cranelift（概念）
```
%l = call nyrt_mutex_lock(%map)
%tbl = iadd %map, off_table
call nyrt_map_put(%tbl, &k, &v)
call nyrt_mutex_unlock(%l)
```

## 🛡️ 検証層の分割

### MIR1 Verifier
- 所有権・効果・GCタグの整合を言語意味としてチェック
- 強1本、強循環禁止、@gcableにfini禁止

### MIR2 Verifier
- ABI/アライン・アドレス計算チェック
- 例外経路の閉路・LOCKペアの整合

## 💎 長所とトレードオフ

### 長所
- LLVM向けに高レベル情報を温存（最適化の妙味が残る）
- Cranelift向けに低レベル直写（JITの安定・速いコンパイル）
- MIR最適化を中立側（MIR1）に集約→どのルートにも効く

### トレードオフ
- IRが2段になるぶんコード量/検証コストが増える
- → 生成を自動化（命令表/enum/dispatchはスクリプト生成）
- → 2層のVerifierを小さく鋭く保つ

## 📅 導入タイミング

- **CraneliftがVMを安定的に越えたら**: MIR2を本格採用
- **LLVMの最適化成果が欲しい配布物**: MIR1→LLVMを標準に
- **それ以外**: MIR1→MIR2→CraneliftでJIT/軽AOT

## ✅ チェックリスト

- [ ] mir1_ops.md / mir2_ops.md（命令と不変条件）
- [ ] Lowering表（各MIR1命令→MIR2展開レシピ）
- [ ] Verifier雛形（MIR2: LOCKペア/landing-pad整合）
- [ ] CLIF生成の最小サブセット
- [ ] 同値テスト実装

---

*「LLVMはMIR1から、CraneliftはMIR2から」- 両方の強みを活かす設計*