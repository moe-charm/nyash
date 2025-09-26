# MIRラウンドトリップ最適化 - 永続チューニング装置

作成日: 2025-08-27
Status: 構想段階
Priority: High
Related: ChatGPT5との共同構想

## 🎯 核心コンセプト

**nyash → MIR → VM → Cranelift → nyash（再出力）**

このループを回すことで、最適化の恩恵を全ルートに一括で波及させる「永続チューニング装置」を実現。

## 🏗️ ラウンドトリップの芯（揺るがない3点）

### 1. 唯一の真実＝MIR
- すべての最適化はMIR上に集約
- VM/Cranelift/WASM/LLVMはMIRの投影でしかない
- ルート固有の最適化は「MIRヒント（属性）」として逆流

### 2. MIRは常に正規形（canonical）へ
```
MIR → Canonicalizer → MIR*
```
- 順序・冗長・等価命令を正規化
- どのルートを通っても再度正規形に戻す
- 振動（最適化の行ったり来たり）を防ぐ

### 3. 再出力nyashは「意味等価の整形」
- MIR → nyash_printerで人が読めるnyashに戻す
- 目的は可読な再配布・差分レビューと最適化の可視化
- コメント/マクロは期待しない

## 💎 必須コンポーネント（超ミニマム）

### MIR Canonicalizer
- SSA再構築
- 冗長Phi削除
- `pure`のCSE
- `mut`の順序制約保持
- `io`は並べ替え禁止

### MIR Verifier
- 強1本・強循環禁止
- `weak`失効チェック
- `@gcable`に`fini`禁止
- 効果整合性

### nyash_printer
- public/privateフィールド再構成
- 関数、with/readガード、Sync<T>復元
- ソースマップ（MIR↔nyash）保存

## 🚀 最適化の入口（MIR上だけ）

### ローカル最適化
- peephole, const-fold, copy-prop
- dead-branch, LICM(pure)

### 所有/弱参照最適化
- adopt/release短絡
- weak_loadのnull先行分岐

### Bus最適化
- elision（pure/read & 単一受信なら直呼び）

### 同期最適化
- `read`→共有ロック
- `write/io`→排他ロック

### プロファイル誘導
```
VM収集データ → MIR属性に反映
- op_count
- hot_bb
- bytes_copied
- send_count
```

## 🛡️ 無限最適化にしない「固定点」の作法

### 三段階で必ず終了
```
正規化 → 最適化 → 正規化
```

### 単調コスト
以下を悪化させたら採用しない/ロールバック：
- `instr_count(pure域)↓`
- `bytes_copied↓`
- `send_count↓`
- `alloc_freeバランス≥0`

### 等価性ガード
- I/Oログ完全一致
- fini順序一致（@must_drop対象）

### ハッシュ鍵
- MIR*の構造ハッシュを成果物キャッシュキーに
- 同形なら二度目以降はスキップ

## 📊 実行フロー（今日から回せる）

```bash
nyash  --emit-mir     src.ny   > a.mir
mir    --canonicalize a.mir    > a*.mir
mir    --optimize     a*.mir   > b.mir
vm     --bc           b.mir    > b.bc
vm     --run          b.bc            # 基準実行＋プロファイル採取
jit    --cranelift    b.mir    > code # JIT or AOT
mir    --pretty       b.mir    > b.ny # 最適化後nyash出力
```

## ✅ CIに入れる「黄金テスト」

- **同値**: `interp == vm == cranelift`（結果＆I/Oログ）
- **GC等価**: `--gc=on/off`でログ差分0
- **資源確定**: `open==fini(@must_drop)`
- **ゼロコピー**: `dup()`以外で`bytes_copied == 0`
- **サイズ**: `send_count / alloc_count`が基準値以下

## 🎯 まず入れるべきパス（効果大きい順）

1. **Bus-elision**（安全条件つき）
2. **CSE/Copy-prop/Dead-branch**（`pure`域）
3. **WeakLoad先読み**（null fast-path）
4. **Mutex/Rwロック折り畳み**（単一呼出シーケンス統合）

※これだけでVM/Cranelift/WASMの全部が速くなる

## 📝 直近ToDo

- [ ] mir canonicalizer（Phi縮退/純粋域の順序固定）
- [ ] bus-elision（安全条件実装）
- [ ] profile schema定義
- [ ] cache key実装
- [ ] nyash_printer v0

---

*「どのルートを通っても、最後は同じMIRへ戻る」- 一箇所直せば全部に効く世界*