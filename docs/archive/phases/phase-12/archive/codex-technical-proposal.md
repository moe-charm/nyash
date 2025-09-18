# Codex先生の技術提案：Nyashスクリプトプラグインシステム実装

## エグゼクティブサマリー

Nyashスクリプトをプラグインとして使用する提案は、技術的に極めて実現可能であり、Nyashエコシステムに革命的な価値をもたらします。「Everything is Box」哲学の究極の実現として、実装言語に依存しない統一インターフェースを提供することで、開発の民主化とエコシステムの爆発的成長が期待できます。

## 技術アーキテクチャ提案

### 1. 統一Box ABIの詳細設計

```rust
// コアインターフェース定義
pub trait UnifiedBoxInterface: Send + Sync {
    // 基本メソッド
    fn invoke(&self, ctx: &mut Context, method_id: u32, args: &[NyashValue]) -> Result<NyashValue, BoxError>;
    fn get_metadata(&self) -> BoxMetadata;
    
    // ライフサイクル管理
    fn initialize(&mut self, config: &BoxConfig) -> Result<(), BoxError>;
    fn shutdown(&mut self) -> Result<(), BoxError>;
    
    // 動的機能（オプション）
    fn hot_reload(&mut self, new_code: &str) -> Result<(), BoxError> {
        Err(BoxError::NotSupported)
    }
}

// メタデータ構造
pub struct BoxMetadata {
    pub name: String,
    pub version: String,
    pub methods: Vec<MethodInfo>,
    pub capabilities: Vec<Capability>,
    pub dependencies: Vec<Dependency>,
}
```

### 2. プラグインレジストリアーキテクチャ

```rust
pub struct PluginRegistry {
    // ネイティブプラグイン
    native_plugins: HashMap<u32, Arc<dyn UnifiedBoxInterface>>,
    
    // スクリプトプラグイン
    script_plugins: HashMap<u32, ScriptPlugin>,
    
    // 動的ID管理
    id_allocator: IdAllocator,
    
    // 依存関係グラフ
    dependency_graph: DependencyGraph,
}

impl PluginRegistry {
    pub fn register_native(&mut self, plugin: impl UnifiedBoxInterface + 'static) -> u32 {
        let id = self.id_allocator.allocate();
        self.native_plugins.insert(id, Arc::new(plugin));
        id
    }
    
    pub fn register_script(&mut self, source: &str) -> Result<u32, RegistryError> {
        let plugin = ScriptPlugin::compile(source)?;
        let id = self.id_allocator.allocate();
        self.script_plugins.insert(id, plugin);
        Ok(id)
    }
}
```

### 3. スクリプトプラグインラッパー実装

```rust
pub struct ScriptPlugin {
    vm: NyashVM,
    box_instance: NyashValue,
    method_cache: HashMap<u32, MethodHandle>,
}

impl UnifiedBoxInterface for ScriptPlugin {
    fn invoke(&self, ctx: &mut Context, method_id: u32, args: &[NyashValue]) -> Result<NyashValue, BoxError> {
        // メソッドキャッシュから高速検索
        if let Some(handle) = self.method_cache.get(&method_id) {
            return self.vm.call_cached(handle, args);
        }
        
        // 動的メソッド解決
        let method = self.resolve_method(method_id)?;
        self.vm.call_method(&self.box_instance, &method, args)
    }
}
```

## 実装戦略

### Phase 1: MVP実装（2-3週間）

1. **基本インターフェース実装**
   - UnifiedBoxInterfaceトレイトの実装
   - 既存FFIプラグイン1つを移行（MathBox推奨）
   - ScriptPluginラッパーの基本実装

2. **export box構文の実装**
   ```nyash
   export box MyPlugin {
       init { _version = "1.0.0" }
       
       // 必須：プラグインメタデータ
       get_metadata() {
           return {
               name: "MyPlugin",
               version: me._version,
               methods: ["process", "transform"]
           }
       }
       
       // ビジネスロジック
       process(data) { ... }
       transform(input) { ... }
   }
   ```

3. **基本的なレジストリ**
   - 静的登録のみ
   - 依存関係解決なし

### Phase 2: 動的機能（3-4週間）

1. **動的ロード/アンロード**
   ```nyash
   local registry = new PluginRegistry()
   local id = registry.load_script("path/to/plugin.ny")
   registry.unload(id)
   ```

2. **ホットリロード**
   ```nyash
   registry.enable_hot_reload("path/to/plugin.ny")
   // ファイル変更時に自動リロード
   ```

3. **依存関係管理**
   - 循環依存検出
   - バージョン互換性チェック

### Phase 3: 最適化とセキュリティ（4-6週間）

1. **パフォーマンス最適化**
   - メソッドキャッシング
   - JITコンパイル統合
   - プリコンパイルオプション

2. **セキュリティサンドボックス**
   ```rust
   pub struct Sandbox {
       memory_limit: usize,
       cpu_quota: Duration,
       allowed_capabilities: HashSet<Capability>,
   }
   ```

3. **ケイパビリティベースセキュリティ**
   - ファイルアクセス制限
   - ネットワーク制限
   - システムコール制限

## パフォーマンス考察

### ベンチマーク予測

```
操作                    | ネイティブ | スクリプト | 比率
--------------------|-----------|-----------|-----
単純メソッド呼び出し    | 10ns      | 100ns     | 10x
複雑な計算（1000ops）  | 1μs       | 5μs       | 5x
I/O操作               | 100μs     | 102μs     | 1.02x
```

### 最適化戦略

1. **ホットパスの識別**
   - 頻繁に呼ばれるメソッドを自動検出
   - JITコンパイル優先度付け

2. **ハイブリッドアプローチ**
   - コア機能：ネイティブ実装
   - カスタマイズ層：スクリプト実装

## エコシステムへの影響

### 開発者体験の革新

1. **即座のフィードバックループ**
   ```bash
   # 編集
   vim my_plugin.ny
   
   # 即座にテスト（ビルド不要）
   nyash test_plugin.ny
   ```

2. **プラグインマーケットプレイス**
   - GitHubから直接インストール
   - バージョン管理統合
   - 自動更新機能

### コミュニティ成長予測

- **現在**: 10-20人のコアコントリビューター（Rust必須）
- **1年後**: 100-500人のプラグイン開発者（Nyashのみ）
- **3年後**: 1000+のプラグインエコシステム

## リスクと緩和策

### 技術的リスク

1. **パフォーマンス劣化**
   - 緩和策：重要部分のネイティブ実装維持
   - プロファイリングツール提供

2. **セキュリティ脆弱性**
   - 緩和策：デフォルトサンドボックス
   - 署名付きプラグイン

### エコシステムリスク

1. **品質のばらつき**
   - 緩和策：公式プラグインガイドライン
   - 自動品質チェックツール

2. **互換性問題**
   - 緩和策：セマンティックバージョニング強制
   - 自動互換性テスト

## 結論と推奨事項

### 即時実行すべきアクション

1. **Box ABI仕様書の作成**（1週間）
2. **export box構文の実装**（2週間）
3. **MathBoxの統一インターフェース移行**（1週間）

### 長期ビジョン

Nyashスクリプトプラグインシステムは、単なる機能追加ではなく、Nyashを**プログラミング言語**から**拡張可能なプラットフォーム**へと進化させる革命的な一歩です。

「Everything is Box」の哲学が、実装言語の壁を超えて真に実現される時、Nyashは次世代のプログラミングエコシステムのモデルケースとなるでしょう。

## 付録：実装例

### A. 完全なスクリプトプラグイン例

```nyash
# advanced_math_plugin.ny
export box AdvancedMathPlugin {
    init {
        _math = new MathBox()
        _cache = new MapBox()
        _stats = new MapBox()
    }
    
    // プラグインメタデータ（必須）
    get_metadata() {
        return {
            name: "AdvancedMathPlugin",
            version: "1.0.0",
            methods: ["cached_sin", "cached_cos", "fibonacci", "factorial"],
            capabilities: ["compute"],
            dependencies: [{
                name: "MathBox",
                version: ">=1.0.0"
            }]
        }
    }
    
    // キャッシュ付き三角関数
    cached_sin(x) {
        local key = "sin:" + x.toString()
        if me._cache.has(key) {
            me._update_stats("cache_hit")
            return me._cache.get(key)
        }
        
        local result = me._math.sin(x)
        me._cache.set(key, result)
        me._update_stats("cache_miss")
        return result
    }
    
    // 再帰的フィボナッチ（メモ化）
    fibonacci(n) {
        if n <= 1 { return n }
        
        local key = "fib:" + n.toString()
        if me._cache.has(key) {
            return me._cache.get(key)
        }
        
        local result = me.fibonacci(n-1) + me.fibonacci(n-2)
        me._cache.set(key, result)
        return result
    }
    
    // 統計情報
    get_stats() {
        return me._stats
    }
    
    // プライベートメソッド
    _update_stats(event) {
        local count = me._stats.get(event) or 0
        me._stats.set(event, count + 1)
    }
}
```

### B. ネイティブとスクリプトの透過的利用

```nyash
// 使用側のコード（プラグインの実装言語を意識しない）
local math1 = new MathBox()           // ネイティブプラグイン
local math2 = include("advanced_math_plugin.ny")  // スクリプトプラグイン

// 同じインターフェースで利用
print(math1.sin(3.14))    // ネイティブ実装
print(math2.cached_sin(3.14))  // スクリプト実装

// 動的に切り替え可能
local math = get_config("use_cached") ? math2 : math1
print(math.sin(1.57))
```

---
*"Write plugins in Nyash, for Nyash, by Nyash!"*