# Phase 12: Nyashコード共有エコシステム - Everything is Box の実現

> Status note (Phase‑15): このフォルダは「統一TypeBox ABI」の長期設計（青写真）です。現在の main 実装は最小の v2 ABI に収束しており、実用重視で段階導入中です。現行仕様は `docs/reference/plugin-abi/nyash_abi_v2.md` を参照してください。Phase‑12 の要素（create/destroy、型メタ、NyValue、vtable/RC/GC）は前方互換を維持しつつ段階追加予定です。

## 🌟 最新ブレイクスルー (2025-09-02) - 統一TypeBox ABI誕生！

### 🚨 究極の発見：ユーザー定義Boxもプラグインに！

**AI先生たちの深い技術的検討により、革命的なアイデアが実現可能と判明！**

```c
// ユーザーBoxもプラグインとして動的登録
NyashTypeBox* register_user_box(const char* name, 
                                NyashBoxMethods* methods);
```

**これにより実現すること**：
- 🎯 **すべての箱をC ABI上で一つで管理**
- 🔄 **ユーザー定義Box ↔ プラグインBox 完全相互運用**
- 🚀 **Nyash側から見ると完全に統一された世界観**

詳細：[ユーザー定義Box統合](./unified-typebox-user-box.md) 🆕

### 🔥 究極の統合：すべてのプラグインがTypeBoxになる！

「Everything is Box」哲学の完成形：**C ABIもNyash ABIも統一TypeBoxに統合**！

```c
// 統一TypeBox - すべてのプラグインがこの形式に！
typedef struct {
    uint32_t abi_tag;        // 'TYBX' - すべて同じ
    uint16_t version;        // APIバージョン
    const char* name;        // "StringBox", "FileBox", etc.
    
    // 基本操作（旧C ABI互換）
    void* (*create)(void* args);
    void (*destroy)(void* self);
    
    // 高速メソッドディスパッチ（新機能）
    uint32_t (*resolve)(const char* name);    // 名前→ID変換
    NyResult (*invoke_id)(void* self,         // ID呼び出し（JIT最適化）
                         uint32_t method_id, 
                         NyValue* args, int argc);
    
    // メタ情報
    uint64_t capabilities;   // THREAD_SAFE | ASYNC_SAFE等
} NyashTypeBox;
```

**3大AI専門家の統合案への評価**:
- **Gemini**: 「技術的妥当性は非常に高い。単なるアイデアではなく、堅牢なアーキテクチャ」
- **Codex**: 「TypeBox統合は実現可能で有益。JIT最適化で33倍高速化も可能」
- **ChatGPT5**: 「『Everything is Box』哲学に最も適した設計」（設計に反映済み）

### 🎯 なぜ統合するのか？

以前は2つのシステムが混在していました：
- **C ABI**：シンプルだが拡張性に限界
- **TypeBox**：プラグイン間連携は可能だが別システム

**統合により実現したこと**：
1. **概念の統一**：すべてがTypeBoxという1つの形式
2. **プラグイン間連携が標準装備**：どのプラグインも他のBoxを作れる
3. **JIT/AOT最適化**：メソッドID化で最大33倍高速化
4. **段階的移行**：既存資産を保護しながら進化

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

詳細な分析：[なぜ天才AIたちは間違えたのか](./design/WHY-AIS-FAILED.md)

## 🚀 Phase 12の真の価値：コード共有エコシステム（同一実行の確立）

最終ゴールは「Nyashコード → VM → JIT の同一実行」。同じプログラムがVMとJITで同じ意味・結果・副作用になるよう、ディスパッチ/ABI/Barrier/Safepointの規約を共有し、差分をなくします。テストハーネスで同値性を比較できるように整備します。

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

## 📊 新しい統一プラグインシステム

```
Nyashエコシステム（統一TypeBox ABI採用後）：
├── Nyashスクリプトプラグイン ← .nyashファイル（純粋なNyashコード）
└── 統一TypeBoxプラグイン    ← .so/.dll（ネイティブ実装）
    ├── 基本機能（旧C ABI互換）
    ├── 高速ディスパッチ（JIT最適化）
    └── プラグイン間連携（標準装備）
```

### 🔄 移行パス
- **既存C ABIプラグイン** → そのまま動作（互換レイヤー経由）
- **新規プラグイン** → 統一TypeBox形式で作成
- **段階的移行** → ツールで自動変換支援

### 📚 プラグインシステムドキュメント

- **[統一TypeBox ABI](./unified-typebox-abi.md)** 🆕 - すべてのプラグインの統一仕様
- **[移行ガイド](./migration-guide.md)** 🆕 - 既存プラグインの移行方法
- **[Nyashスクリプトプラグイン](./nyash-script-plugins.md)** - 純粋なNyashコードのプラグイン

### 📦 レガシードキュメント（参考用）
- [旧C ABI仕様](./archive/legacy-abi-docs/c-abi.md)
- [旧Nyash ABI仕様](./archive/legacy-abi-docs/nyash-abi.md)

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

詳細: [C ABI TypeBox設計仕様書](./archive/legacy-abi-docs/C-ABI-BOX-FACTORY-DESIGN.md)

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

## 🛣️ 実装ロードマップ（統一TypeBox ABI版）

### Phase 12.0: 統一TypeBox ABI実装（1週間）🆕
- [ ] nyash_typebox.h完全ヘッダー定義
- [ ] メソッドID解決・キャッシング機構
- [ ] NyValue統一値表現の実装
- [ ] 互換レイヤー（既存C ABI→TypeBox）
- 📄 **[統一TypeBox ABI仕様](./unified-typebox-abi.md)**

### Phase 12.0.5: 移行ツール開発（2週間）🔧
- [ ] プラグイン自動変換ツール
- [ ] 検証・テストツール
- [ ] パフォーマンスベンチマーク
- [ ] サンプルプラグイン集
- 📄 **[移行ガイド](./migration-guide.md)**

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

### 🎯 プラグインシステムガイド（メインドキュメント）
- **[統一TypeBox ABI](./unified-typebox-abi.md)** ← 🌟🆕 すべてのプラグインの新仕様！
- **[ユーザー定義Box統合](./unified-typebox-user-box.md)** ← 🔥🆕 革命的な完全統合！
- **[AI先生たちの技術的検討](./ai-consultation-unified-typebox.md)** ← 🤖🆕 深い分析と提言（Codex詳細版追加）
- **[技術的決定事項](./TECHNICAL_DECISIONS.md)** ← 📋🆕 確定した技術仕様まとめ
- **[実装ロードマップ](./IMPLEMENTATION_ROADMAP.md)** ← 📅🆕 詳細な実装計画
- **[移行ガイド](./migration-guide.md)** ← 🆕 既存プラグインを新形式へ
- **[Nyashスクリプトプラグイン](./nyash-script-plugins.md)** ← 純粋なNyashコードのプラグイン

### 📐 設計ドキュメント（design/）
- **[統合ABI設計仕様書](./design/UNIFIED-ABI-DESIGN.md)** ← 統合の詳細設計
- **[C ABI TypeBox設計仕様書](./archive/legacy-abi-docs/C-ABI-BOX-FACTORY-DESIGN.md)** ← TypeBoxの原点
- **[Nyash ABI C実装設計書](./design/NYASH-ABI-C-IMPLEMENTATION.md)** ← セルフホスティング構想
- **[なぜ天才AIたちは間違えたのか](./design/WHY-AIS-FAILED.md)** ← 設計プロセスの教訓

### 📋 仕様書（specs/）
- **[export/import仕様](./specs/export-import-spec.md)** ← モジュールシステムの詳細仕様
- **[パッケージマネージャー設計](./specs/package-manager-design.md)** ← パッケージ管理の設計

### 💬 議論の過程（discussions/）
- **ABI戦略議論**: `discussions/abi-strategy-discussion/`
- **Nyash ABI詳細**: `discussions/nyash-abi-discussion/`

### 📦 アーカイブ
- **初期提案**: `archive/` ← 過去の提案や古いドキュメント

---

*AIたちがなぜ複雑な解決策を提案したのか、その議論の過程は `archive/` ディレクトリに保存されています。良い教訓として残しておきます。*
