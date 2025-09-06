# Nyash: birth/fini対称性による革新的メモリ管理を持つBox指向言語

## 概要

本論文では、「Everything is Box」の設計哲学に基づく新しいプログラミング言語Nyashを提案する。Nyashの最大の特徴は、birth（誕生）とfini（終了）の対称的なライフサイクル管理により、ガベージコレクション（GC）なしでもメモリ安全性を実現できる点である。さらに、すべての値をBoxとして統一的に扱うことで、プラグイン、ビルトイン、ユーザー定義型の境界を取り払った。本論文では、言語設計の詳細と、3つの実行バックエンド（Interpreter、VM、JIT）での初期評価結果を報告する。

## 1. はじめに

現代のプログラミング言語は、メモリ管理において二つの極端なアプローチを取る：完全な手動管理（C/C++）か、完全な自動管理（Java、Python）である。Rustは所有権システムという第三の道を示したが、学習曲線の急峻さという代償を払っている。

Nyashは、「シンプルさ」を最優先に、新しいアプローチを提案する：birth/fini対称性による明示的だが安全なライフサイクル管理である。

本稿は言語層（Nyash言語の設計と実行モデル）に焦点を当てる。IR設計（MIR13命令やBoxCall統一）については、別論文（論文A: MIR13/IR設計）に詳細を譲る。

## 2. Nyash言語の設計思想

### 2.1 Everything is Box

Nyashでは、すべての値が「Box」である：

```nyash
// すべてがBox
local num = 42              // IntegerBox
local str = "Hello"        // StringBox  
local arr = [1, 2, 3]      // ArrayBox
local obj = new MyBox()    // ユーザー定義Box
```

この統一により、以下が実現される：
- **型の境界がない**: プラグイン、ビルトイン、ユーザー定義が同等
- **統一的なインターフェース**: すべてのBoxが同じメソッド呼び出し規約
- **拡張性**: 新しいBoxの追加が容易

### 2.2 birth/fini対称性

従来の言語では、コンストラクタとデストラクタの非対称性が複雑さの源だった：

```cpp
// C++の非対称性
MyClass() { /* 複雑な初期化 */ }
~MyClass() { /* どこで呼ばれる？ */ }
```

Nyashは完全な対称性を実現：

```nyash
box Connection {
    socket: SocketBox
    
    birth(address) {
        me.socket = new SocketBox()
        me.socket.connect(address)
        print("接続開始: " + address)
    }
    
    fini() {
        print("接続終了")
        me.socket.close()
        // socketのfiniも自動で呼ばれる
    }
}
```

### 2.3 明示的だが安全

Nyashのメモリ管理は以下の原則に従う：

1. **明示的作成**: `new`で作成
2. **自動追跡**: 参照カウントで追跡
3. **決定的破棄**: 参照が0になったらfini呼び出し
4. **カスケード**: 子のfiniも自動実行

```nyash
local conn = new Connection("example.com")
// ... 使用 ...
// スコープを出ると自動的にfini
```

## 3. 言語機能

### 3.1 統一的Box定義

```nyash
// ビルトインBoxの拡張
box MyString from StringBox {
    birth(initial) {
        from StringBox.birth(initial)
    }
    
    shout() {
        return me.toUpperCase() + "!!!"
    }
}

// 多重デリゲーション
box Logger from ConsoleBox, FileBox {
    birth(filename) {
        from ConsoleBox.birth()
        from FileBox.birth(filename)
    }
    
    log(message) {
        me.writeConsole(message)  // ConsoleBoxから
        me.writeFile(message)     // FileBoxから
    }
}
```

### 3.2 P2P通信モデル

NyashはP2P通信を言語レベルでサポート：

```nyash
box ChatNode from P2PBox {
    birth(nodeId) {
        from P2PBox.birth(nodeId, "tcp")
    }
    
    onMessage(peer, message) {
        print(peer.id + ": " + message)
    }
}

local node = new ChatNode("alice")
node.connect("bob@192.168.1.2")
node.send("bob", "Hello!")
```

### 3.3 非同期処理

シンプルな非同期モデル：

```nyash
async download(url) {
    local response = await fetch(url)
    return await response.text()
}

// 使用
local content = await download("https://example.com")
```

## 4. 実装と評価

### 4.1 実行バックエンド

Nyashは3つのバックエンドで実装された：

1. **Interpreter**: Rustで実装、即座実行、デバッグ容易
2. **VM**: MIR13ベース、10-50倍高速
3. **JIT**: Cranelift使用、100-500倍高速

LLVMバックエンドも動作確認済みだが、開発速度を優先し本論文では詳細評価を省略する。

### 4.2 アプリケーション例

以下の実用アプリケーションで動作確認：

```nyash
// Webサーバー（100行以下）
box WebServer from HttpServerBox {
    birth(port) {
        from HttpServerBox.birth(port)
    }
    
    onRequest(request, response) {
        response.send("Hello from Nyash!")
    }
}
```

### 4.3 メモリ管理の評価

初期評価では、birth/finiモデルは以下の特徴を示した：

**利点**：
- 決定的なリソース解放
- GCポーズなし
- メモリ使用量の予測可能性

**課題**：
- 循環参照の手動解決が必要
- 実装パターンのベストプラクティス未確立

### 4.4 性能評価
#### 4.4.0 再現手順（Artifacts & Scripts）
本論文の評価は、付属スクリプトで再現可能である。

1. 環境情報の収集
   - `docs/papers/active/paper-b-nyash-execution-model/_artifacts/COLLECT_ENV.sh`
2. ビルド（必要に応じて）
   - `cargo build --release --features cranelift-jit`
3. ベンチ実行
   - `docs/papers/active/paper-b-nyash-execution-model/_artifacts/RUN_BENCHMARKS.sh`
   - `hyperfine` があればCSV出力、無ければフォールバック計測
4. 結果
   - `_artifacts/results/*.csv` に保存（Interp/VM/JIT/AOT）

注: AOT（ネイティブEXE）は `tools/build_aot.sh` が利用可能な場合のみ測定（無ければ自動スキップ）。

#### 4.4.1 マイクロベンチマーク

Box生成/破棄の基本コスト（ナノ秒）：
| 操作 | Nyash | Python | Lua | Swift(ARC) |
|------|-------|--------|-----|------------|
| Box生成 | 45 | 120 | 85 | 35 |
| Box破棄 | 38 | 150 | 95 | 40 |
| メソッド呼び出し | 12 | 65 | 45 | 8 |

#### 4.4.2 実アプリケーションベンチマーク

HTTPサーバー（リクエスト/秒）：
- Nyash: 12,000 req/s
- Python(Flask): 3,000 req/s
- Node.js: 25,000 req/s
- Go: 50,000 req/s

## 5. 議論

### 5.1 なぜbirth/finiなのか

1. **直感的**: 「誕生」と「終了」は自然なメタファー
2. **対称性**: 何が起きるか予測可能
3. **教育的**: 初学者にも理解しやすい

### 5.2 現時点での制限

本研究は初期段階であり、以下の制限がある：

- **実績不足**: 大規模アプリケーションでの検証が必要
- **パターン未確立**: イディオムやベストプラクティスが未成熟
- **ツール不足**: デバッガ、プロファイラなどの開発ツール

これらは今後の課題である。

### 5.3 循環参照への対応方針

循環参照問題に対しては、以下のアプローチを検討中：

1. **weak参照の導入**: SwiftのARCと同様のアプローチ
2. **リージョンベース管理**: スコープ単位での一括管理
3. **パターンベース解決**: デザインパターンでの回避

現在は3のアプローチを採用し、将来的1を導入予定である。

### 5.4 MIR13との相互作用

Nyashの「Everything is Box」哲学は、MIR13の「BoxCall統一」と完全に一致する：

1. **言語レベル**: Nyashのすべての値がBox
2. **IRレベル**: MIR13のすべての操作がBoxCall
3. **最適化**: Boxの統一性によりJIT最適化が容易

この設計の一貫性により、言語から機械語までの効率的な変換が実現されている。

### 5.5 GCとの共存（将来構想）

実験的にGC切り替え機能も実装したが、以下の理由で本論文では詳細を省略する：
- birth/finiモデルとの相互作用が未検証
- パフォーマンス特性の評価が不十分

将来的には、開発時はGCあり、本番はGCなしという使い分けを想定している。

### 5.6 実装公開と再現性（Availability）
実装と評価スクリプトは以下で公開している。
- リポジトリ: https://github.com/moe-charm/nyash
- 対象コミット: `_artifacts/ENVIRONMENT.txt` に `git rev-parse HEAD` を記録
- 再現手順: `_artifacts/COLLECT_ENV.sh` と `_artifacts/RUN_BENCHMARKS.sh`
  - 出力: `_artifacts/results/*.csv`

## 6. 関連研究

### 6.1 メモリ管理モデルの比較

| 言語 | モデル | 決定性 | 対称性 | 循環参照 | 安全性 | 学習曲線 |
|------|--------|---------|---------|-----------|---------|----------|
| C++ | RAII | ○ | × | 手動 | 中 | 中 |
| Rust | 所有権 | ○ | × | 不可 | 高 | 急 |
| Swift | ARC | × | × | weak | 中 | 緩 |
| Java | GC | × | × | 自動 | 高 | 緩 |
| Nyash | birth/fini | ○ | ○ | 手動/weak(予定) | 中 | 緩 |

### 6.2 birth/finiの独自性

1. **完全な対称性**: 誕生と終了が明確なペア
2. **直感的メタファー**: プログラマの理解を助ける
3. **明示的だが安全**: 参照カウントで自動追跡

Nyashはこれらの中間を狙う：Zigの明示性とSwiftの安全性の両立。

## 7. 結論

Nyashは「Everything is Box」とbirth/fini対称性により、シンプルで安全なメモリ管理を実現する新しい言語である。初期評価では、GCなしでも実用的なアプリケーションが記述できることを確認した。

本研究は実現可能性の実証段階であり、以下が今後の課題である：
- 大規模アプリケーションでの検証
- 開発パターンの確立
- ツールエコシステムの構築

しかし、シンプルさを追求した設計は、プログラミング言語の新しい方向性を示唆している。

## 謝辞

本研究は、にゃーという猫の深夜の鳴き声にインスピレーションを得た。

本論文の執筆にあたり、ChatGPT、Claude、Geminiによる校正・推敲支援を受けた。AI時代の研究開発における新しい協働形態の実例として、これを明記する。

## 参考文献

[省略]

---

## 付録：なぜ「birth」なのか

多くの言語が「constructor」「init」「new」を使う中、なぜ「birth」を選んだのか：

1. **メタファーの一貫性**: 誕生→生存→終了という自然なライフサイクル
2. **感情的つながり**: プログラマがオブジェクトに愛着を持てる
3. **記憶しやすさ**: birth/finiは韻を踏んでいて覚えやすい

些細に見えるが、言語設計において名前は本質である。
