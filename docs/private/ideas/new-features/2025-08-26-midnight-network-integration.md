# Midnight Network Integration (MidnightBox)
Status: Research
Created: 2025-08-26
Priority: Low (Challenge-specific)
Related: Privacy-preserving applications, ZK proofs

## 概要
Midnight Networkとの統合により、Nyashでプライバシー保護アプリケーションを開発可能にする。

## 背景
- Midnight Network Privacy Challenge ($5,000)への参加機会
- Compact言語（TypeScriptベースDSL）からTypeScript/JavaScript生成
- ZKプルーフを使用したプライバシー保護スマートコントラクト

## 実装アプローチ案

### 1. ビルトインBox実装
```nyash
// MidnightBoxの使用例
local midnight = new MidnightBox()
midnight.connect("contract-address")

// ZKプルーフ生成
local proof = midnight.createProof("validVote", {
    voterId: "Alice",
    choice: "OptionA"
})

// トランザクション送信
local result = midnight.submitTransaction(proof)
```

### 2. HTML埋め込み戦略
```html
<!-- Compact → TypeScript → JavaScript -->
<script src="midnight-js-sdk.js"></script>
<script src="voting-contract.js"></script>

<!-- Nyash WASM統合 -->
<script type="module">
    window.midnightContract = new VotingContract();
    // Nyashから extern_call() で呼び出し
</script>
```

### 3. 実装フロー
1. Compact言語で契約記述
2. CompactコンパイラでTypeScript生成
3. HTML/JavaScriptに統合
4. NyashのExternCallで呼び出し

## チャレンジ向け実装例

### プライバシー投票システム
```nyash
box PrivateVotingApp {
    init { midnight, publicLedger, privateLedger }
    
    constructor() {
        me.midnight = new MidnightBox()
        me.publicLedger = new ArrayBox()  // 証明のみ
        me.privateLedger = new MapBox()   // 暗号化データ
    }
    
    vote(voterId, choice) {
        // ZKプルーフで投票の有効性を証明
        local proof = me.midnight.proveValidVote({
            hasRightToVote: true,
            hasNotVotedYet: true,
            choiceIsValid: true
        })
        
        // 公開台帳には証明のみ
        me.publicLedger.push(proof)
        
        // プライベート台帳に暗号化データ
        me.privateLedger.set(
            hash(voterId),
            me.midnight.encrypt(choice)
        )
        
        return proof
    }
}
```

## 技術的課題
1. **WASM統合**: wasm-bindgenでJavaScript関数バインディング
2. **型安全性**: RustとTypeScriptの型マッピング
3. **非同期処理**: Midnight SDKの非同期APIとの連携
4. **エラーハンドリング**: ZKプルーフ生成失敗時の処理

## 実装優先度
- チャレンジ応募時のみ必要
- モジュラービルトインBoxシステム実装後に検討
- プロトタイプはExternCallで十分

## 参考資料
- Midnight Network Documentation
- Compact Language Reference
- DEV.to Challenge Page

## メモ
- 締切: 2025年9月7日
- "Protect That Data"トラック: $3,500賞金
- モックではなく実際のMidnight SDK使用が必要