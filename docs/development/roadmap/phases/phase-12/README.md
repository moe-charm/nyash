# Phase 12: Nyashコード共有エコシステム - Everything is Box の実現

## 🚀 最新ブレイクスルー (2025-09-02)

### 🔥 セルフホスティングへの道 - ABIすらBoxとして扱う！
「Everything is Box」哲学の究極形態：**ABIそのものをBoxとして実装**することで、Rust依存を完全排除！

```c
// Nyash ABIもTypeBoxとして提供（C言語実装）
typedef struct {
    uint32_t abi_tag;           // 'NABI'
    const char* name;           // "NyashABIProvider"
    void* (*create)(void);      // ABIプロバイダ生成
    
    // Nyash ABI操作（Rust非依存）
    struct {
        nyash_status (*call)(nyash_ctx*, void* obj, nyash_selector, ...);
        void (*retain)(void* obj);
        void (*release)(void* obj);
    } nyash_ops;
} NyashABITypeBox;
```

**AI専門家たちの深い考察**:
- **Gemini**: 「技術的妥当性が高く、言語哲学とも合致した、極めて優れた設計」
- **Codex**: 「Feasible and attractive: 16バイトアライメント、セレクターキャッシング等の具体的実装提案」
- **ChatGPT5**: 「統合ABI設計に10の改善点を提供」（反映済み）

### TypeBox統合ABI - プラグイン革命の実現！
「Everything is Box」哲学：**型情報すらBoxとして扱う**TypeBoxにより、C ABI + Nyash ABIの完全統合を達成！

## 🎯 重要な変更 (2025-09-01)

Phase 12の議論とビルトインBox廃止により、プラグインシステムが進化：

**新しい3層プラグインシステムが確立されました！**

```nyash
# Nyashスクリプトプラグイン（ユーザー定義Box）
box DataProcessor {
    init {
        me.file = new FileBox()    # C ABIプラグイン使用
        me.math = new MathBox()    # C ABIプラグイン使用
        me.cache = new MapBox()    # これもC ABIプラグイン（ビルトイン廃止）
    }
    
    process(data) {
        local result = me.math.sin(data)
        me.file.write("log.txt", result.toString())
        return result
    }
}

# 使用例
local processor = new DataProcessor()
processor.process(3.14)  # すべてプラグインで動作！
```

## 📝 なぜ誤解が生まれたのか

「プラグイン」という言葉から、特別な仕組みが必要だと考えてしまいましたが、Nyashの「Everything is Box」哲学により、ユーザー定義Boxこそが最高のプラグインシステムでした。

詳細な分析：[なぜ天才AIたちは間違えたのか](./WHY-AIS-FAILED.md)

## 🚀 Phase 12の真の価値：コード共有エコシステム

### 本当に必要なもの

1. **export/import構文**
   ```nyash
   # math_utils.ny
   export box MathUtils {
       factorial(n) { ... }
       fibonacci(n) { ... }
   }
   
   # main.ny
   import { MathUtils } from "math_utils.ny"
   local utils = new MathUtils()
   ```

2. **パッケージマネージャー**
   ```bash
   nyash install awesome-math-utils
   nyash publish my-cool-box
   ```

3. **ドキュメント生成**
   ```nyash
   # @doc 素晴らしい数学ユーティリティ
   # @param n 計算したい数値
   # @return 階乗の結果
   export box MathUtils { ... }
   ```

## 📊 新しい3層プラグインシステム

```
Nyashエコシステム（ビルトインBox廃止後）：
├── Nyashスクリプトプラグイン（ユーザー定義Box）← .nyashファイル
├── C ABIプラグイン（既存のまま使用）← シンプル・高速・安定
│   └── **TypeBox**: プラグイン間Box生成の最小機構 🆕
└── Nyash ABIプラグイン（必要時のみ）← 言語間相互運用・将来拡張
    └── MIR命令は増やさない（BoxCallにabi_hint追加のみ）
```

### 💡 TypeBox：シンプルなプラグイン間連携

MapBox.keys()がArrayBoxを返したい場合：

```c
// TypeBox構造体（型情報をBoxとして扱う）
typedef struct {
    uint32_t abi_tag;       // 'TYBX'
    const char* name;       // "ArrayBox"
    void* (*create)(void);  // Box生成関数
} NyrtTypeBox;

// MapBox.keys()実装
void* map_keys(void* self, void* array_type_box) {
    NyrtTypeBox* array_type = (NyrtTypeBox*)array_type_box;
    void* array = array_type->create();  // ArrayBox生成
    // ... キーを追加
    return array;
}
```

詳細: [C ABI TypeBox設計仕様書](./C-ABI-BOX-FACTORY-DESIGN.md)

### プラグイン選択の指針
- **C ABIで済むなら、C ABIを使う**（シンプルイズベスト）
- Nyash ABIは以下の場合のみ：
  - 他言語（Python/Go等）からの呼び出し
  - 複雑な型の相互運用が必要
  - 将来の拡張性を重視する場合

### 📝 MIR命令統合（Phase 12での変更）
- **PluginInvoke → BoxCall 統合**
  - ビルトインBox廃止によりフォールバックがなくなる
  - BoxCallとPluginInvokeの区別が不要に
  - VM層でC ABI/Nyash ABI/Scriptを自動判定
  - Core-15 → Core-14 へ（命令数削減）

## 🛣️ 実装ロードマップ（セルフホスティング対応版）

### Phase 12.0: TypeBox統合ABI実装（1週間）🆕
- [ ] nyrt_typebox.h完全ヘッダー定義（16バイトアライメント）
- [ ] セレクターキャッシング機構
- [ ] MapBox両ABI実装（実証テスト）
- [ ] 所有権ファズテスト
- 📄 **[統合ABI設計仕様書](./UNIFIED-ABI-DESIGN.md)**

### Phase 12.0.5: Nyash ABI C実装（2週間）🔥🆕
- [ ] C Shim実装（既存Rustへのラッパー）
- [ ] 基本型のC完全実装（Integer/String/Bool）
- [ ] アトミック参照カウント + 弱参照
- [ ] 適合性テストスイート
- 📄 **[Nyash ABI C実装設計書](./NYASH-ABI-C-IMPLEMENTATION.md)**

---

## 現状サマリ（2025-09-02）

- C ABI（TLV: 1/2/3/5/6/7/8）でのプラグイン呼び出しはVMで安定運用中。`returns_result` も `nyash.toml` で制御可能。
- JIT は VM と同じBox境界で動作（フォールバック含む）。Cranelift AOT のオブジェクト出力は未配線（スケルトン）。
- MapBox を拡張（stringキー、remove/clear/getOr/keysStr/valuesStr/toJson）。`keys()/values()` はランタイムシムで暫定提供。
- Phase 12 設計（TypeBox + Unified Dispatch）は破壊的変更不要で段階導入可能と判断。

詳細タスクは [TASKS.md](./TASKS.md) を参照。


### Phase 12.1: export/import構文（2週間）
- [ ] exportキーワードのパーサー実装
- [ ] importステートメントの実装
- [ ] モジュール解決システム
- 📄 **[詳細仕様書](./export-import-spec.md)**

### Phase 12.2: パッケージ管理（3週間）
- [ ] nyash.tomlのdependencies対応
- [ ] 中央リポジトリ設計
- [ ] CLIツール（install/publish）
- 📄 **[パッケージマネージャー設計書](./package-manager-design.md)**

### Phase 12.3: 開発者体験向上（継続的）
- [ ] ドキュメント生成ツール
- [ ] VSCode拡張（補完・定義ジャンプ）
- [ ] サンプルパッケージ作成

## 📚 関連ドキュメント

### 🎯 主要設計ドキュメント
- **[統合ABI設計仕様書](./UNIFIED-ABI-DESIGN.md)** ← 🆕🚀 C ABI + Nyash ABI統合の完全設計！**3大AI専門家検証済み**
- **[C ABI TypeBox設計仕様書](./C-ABI-BOX-FACTORY-DESIGN.md)** ← 🆕 シンプルなプラグイン間Box生成！
- **[Nyash ABI C実装設計書](./NYASH-ABI-C-IMPLEMENTATION.md)** ← 🆕🔥 セルフホスティング実現！**Gemini/Codex絶賛**
- **[Nyash ABI統合設計図](./NYASH-ABI-DESIGN.md)** ← 将来拡張用の高度なABI
- [export/import仕様](./export-import-spec.md)
- [パッケージマネージャー設計](./package-manager-design.md)
- [なぜ天才AIたちは間違えたのか](./WHY-AIS-FAILED.md)

### 📂 議論の過程

- ABI戦略議論: `abi-strategy-discussion/`
- Nyash ABI詳細: `nyash-abi-discussion/`
- 初期提案アーカイブ: `archive/`

---

*AIたちがなぜ複雑な解決策を提案したのか、その議論の過程は `archive/` ディレクトリに保存されています。良い教訓として残しておきます。*
