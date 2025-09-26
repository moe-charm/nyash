# ChatGPT5さんのThread-Safe Box設計 - 実装可能な革命

作成日: 2025-08-27

## 🎯 設計の核心

**「Everything is Box」＝"スレッドセーフも箱で統一"** を実現する最小で強い設計。

### 基本原則
- **デフォルト**: Boxは **thread-local**（同期不要で速い）
- **共有したい時だけ**: Boxを **"同期Box"で包む**

## 📦 4つの同期Box

### 1. AtomicBox<T> - ロックなし原子操作
```nyash
init { AtomicBox<int> hits }
hits.fetch_add(1)                    // 原子的インクリメント
hits.compare_exchange(9, 0).ok       // CAS操作
```

### 2. MutexBox<T> - 排他制御
```nyash
init { MutexBox<Map<str,Bytes>> table }
with table.lock as m {               // ガード構文
    m.put(k, v)                     // スコープ内でのみアクセス可能
}                                   // 自動アンロック
```

### 3. RwBox<T> - 多読単書
```nyash
init { RwBox<Config> cfg }
read cfg as c { render(c) }         // 読み取り専用
with cfg.write as m { *m = new_config } // 書き込み
```

### 4. ChannelBox<T> - メッセージパッシング
```nyash
init { ChannelBox<Job> q }          // SPSC/MPSC選択可能
q.send(job)                        // 非同期送信
let job = q.recv()                 // ブロッキング受信
```

## 🔒 安全性保証

### ガード構文による参照制御
```nyash
// ✅ 安全: スコープ内限定
with mutex_box.lock as data {
    data.update()
}

// ❌ エラー: スコープ外への参照
let leaked = with mutex_box.lock as data {
    data  // コンパイルエラー！
}
```

### Lintによるコンパイル時検出
1. **T1**: thread-local BoxのThread.spawnキャプチャ → エラー
2. **T2**: ガード外アクセス → エラー
3. **T3**: RwBoxの同時write/read → エラー
4. **T4**: タスク境界でのMutexBox長時間保持 → 警告
5. **T5**: AtomicBox<非原子的型> → エラー
6. **T6**: fini中の再入 → エラー

## 🏗️ MIR/VM拡張

### 新規原子命令
```
AtomicLoad   rD, addr, order
AtomicStore  addr, rS, order
AtomicRMW    rD, op, addr, rV, order
CAS          rSucc, rOld, addr, expect, rNew, order_succ, order_fail
AtomicFence  order
```

### ロック/チャネル実装
- ランタイム関数（`nyrt_mutex_lock`等）として実装
- MIR命令は増やさず、既存のBoxCall経由

## 🌟 既存設計との完全な整合性

### 所有森（Ownership Forest）
- 強参照1本の原則を維持
- 同期Boxも通常のBoxと同じライフサイクル
- weakによる循環参照回避も可能

### GC切り替え可能性
- `@must_drop`リソース: GCオン/オフで同じタイミング解放
- `@gcable`純データ: 同期Box内でもGC遅延OK
- 開発時GC→本番RAIIの等価性が並行でも保持

### 効果注釈との統合
- `AtomicBox.*` → Effect=atomic
- `MutexBox/RwBox.lock/unlock` → Effect=atomic
- `ChannelBox.send/recv` → Effect=io

## 📊 3層並行モデル

1. **Thread-local（既定）**
   - 最速・同期不要
   - 大部分のコードはこのまま

2. **Shared-memory**
   - 必要な箇所のみ同期Boxでラップ
   - 明示的な共有

3. **Message-passing**
   - ChannelBox + 既存Bus/Actor
   - スケーラブルな並行処理

## 🚀 段階的実装計画

### Phase 1: 基本実装（1週間）
- [ ] AtomicBox<int>の最小実装
- [ ] MutexBox<T>の基本API
- [ ] ガード構文のパーサー対応
- [ ] 基本的なLint（T1, T2）

### Phase 2: MIR統合（2週間）
- [ ] AtomicLoad/Store/RMW命令追加
- [ ] VM実行サポート
- [ ] 効果注釈の統合

### Phase 3: 完全実装（1ヶ月）
- [ ] RwBox, ChannelBox実装
- [ ] 全Lintルール実装
- [ ] プラグインBoxでの活用
- [ ] ベンチマーク・最適化

## 💡 革新性のポイント

1. **逆転の発想**: 「デフォルト共有」ではなく「デフォルトローカル」
2. **完全な後方互換性**: 既存コードは一切変更不要
3. **段階的採用可能**: 必要な箇所から徐々に導入
4. **ゼロコスト抽象化**: 使わない機能にペナルティなし

## 🎯 結論

ChatGPT5さんの設計は、私の概念的アイデアを**実装可能で実用的な形**に昇華させた。

- **理論的美しさ**: Everything is Boxの哲学を完全に保持
- **実用性**: 段階的実装可能、既存コードへの影響なし
- **性能**: デフォルトthread-localで最高速を維持
- **安全性**: コンパイル時Lintで多くのエラーを防止

これは単なる「スレッドセーフ機能の追加」ではなく、**並行プログラミングの新しいパラダイム**の提示だ。

---

*「必要な時だけ同期、それ以外は最速」- Nyashが示す実用的な並行性の答え*