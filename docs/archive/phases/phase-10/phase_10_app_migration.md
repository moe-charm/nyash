# Phase 10: Classic C Applications Migration to Nyash

## 🎯 概要
3つの著名なCアプリケーションをNyashに移植し、新実装された高度なメモリ管理機能を実戦テストする。

## 📦 移植対象アプリケーション（優先順位順）

### 1. 🌐 **Tinyproxy** - ゼロコピー判定機能の実証
**元実装**: https://github.com/tinyproxy/tinyproxy
**サイズ**: ~5000行C、軽量HTTPプロキシサーバー
**Nyash移植目標**: `apps/tinyproxy_nyash/`

#### 🔍 **ゼロコピー判定テストケース**
```nyash
// HTTPリクエスト転送でのメモリ効率検証
static box ProxyServer {
    init { upstream_buffer, downstream_buffer }
    
    relay_data(client_data) {
        // ⭐ ゼロコピー判定：バッファーが共有されているかチェック
        if (me.upstream_buffer.is_shared_with(client_data)) {
            console.log("✅ Zero-copy achieved!")
        } else {
            console.log("❌ Unnecessary copy detected")
        }
        
        // 大量データ転送での最適化確認
        return me.upstream_buffer.share_reference(client_data)
    }
}
```

#### 📋 **実装要件**
- HTTPプロキシの基本機能（GET/POST転送）
- `SocketBox`でのクライアント・サーバー接続
- `BufferBox`での効率的なデータ転送
- **ゼロコピー判定API**の実装・テスト

---

### 2. 🎮 **Chip-8エミュレーター** - fini伝播とweak生存チェック
**元実装**: https://github.com/mattmikolay/chip-8 (参考)
**サイズ**: ~1000行C、8ビットゲーム機エミュレーター
**Nyash移植目標**: `apps/chip8_nyash/`

#### 🔍 **メモリ管理テストケース**
```nyash
// CPU・メモリ・グラフィックスの相互参照関係でのfini伝播テスト
static box Chip8CPU {
    init { memory, graphics, sound }
    
    fini() {
        // ⭐ fini伝播：依存オブジェクトの自動クリーンアップ
        console.log("🔄 CPU cleanup triggered")
        me.memory.cleanup()  // メモリバンクの解放
        me.graphics.cleanup()  // VRAM解放
    }
}

static box Chip8Memory {
    init { ram, weak_cpu_ref }  // CPUへの弱参照
    
    read_byte(address) {
        // ⭐ weak生存チェック：CPUがまだ生きているか確認
        if (me.weak_cpu_ref.is_alive()) {
            return me.ram.get(address)
        } else {
            console.log("⚠️ CPU destroyed, memory access blocked")
            return null
        }
    }
}
```

#### 📋 **実装要件**
- Chip-8命令セット実装（35命令）
- 64x32ピクセルグラフィックス（`WebCanvasBox`使用）
- サウンド出力（`SoundBox`使用）
- **fini伝播システム**と**weak参照**の実戦テスト

---

### 3. ✏️ **kilo テキストエディター** - 「うっかり全体コピー」検出
**元実装**: https://github.com/antirez/kilo
**サイズ**: ~1000行C、軽量ターミナルエディター
**Nyash移植目標**: `apps/kilo_nyash/`

#### 🔍 **メモリ効率テストケース**
```nyash
// 大きなテキストファイル編集での不必要なコピー検出
static box TextBuffer {
    init { lines, undo_stack }
    
    insert_char(row, col, char) {
        local old_lines_size = me.lines.memory_footprint()
        
        // 文字挿入操作
        me.lines.get(row).insert_at(col, char)
        
        local new_lines_size = me.lines.memory_footprint()
        local size_diff = new_lines_size - old_lines_size
        
        // ⭐ 「うっかり全体コピー」検出
        if (size_diff > 1000) {  // 1文字挿入で1KB以上増加
            console.log("🚨 INEFFICIENT COPY DETECTED!")
            console.log("Expected: 1 byte, Actual: " + size_diff + " bytes")
            me.log_memory_leak_warning()
        }
    }
    
    // 大規模な検索・置換での効率性チェック
    search_and_replace(pattern, replacement) {
        local initial_memory = me.lines.memory_footprint()
        
        // 検索・置換実行
        me.lines.replace_all(pattern, replacement)
        
        local final_memory = me.lines.memory_footprint()
        // メモリ使用量が2倍を超えた場合は問題
        if (final_memory > initial_memory * 2) {
            console.log("⚠️ Memory usage doubled during replace operation")
        }
    }
}
```

#### 📋 **実装要件**
- ターミナル操作（`ConsoleBox`での入出力）
- ファイル読み書き（`FileBox`使用）
- 基本的な編集機能（カーソル移動、挿入、削除）
- **メモリ効率監視**と**コピー検出システム**

---

## 🛠️ **技術的実装指針**

### 共通アーキテクチャ
```nyash
// 各アプリケーション共通の構造
static box AppName {
    init { core_components }
    
    main() {
        me.initialize_components()
        me.run_main_loop()
        me.cleanup_resources()
    }
    
    // メモリ効率レポート（全アプリ共通）
    memory_report() {
        return new MapBox()
            .set("zero_copy_count", me.zero_copy_operations)
            .set("unnecessary_copies", me.detected_copies)
            .set("memory_leaks", me.fini_failures)
            .set("weak_ref_cleanups", me.weak_cleanup_count)
    }
}
```

### 新API要件
1. **ゼロコピー判定API**
   - `BufferBox.is_shared_with(other)` → BoolBox
   - `BufferBox.share_reference(data)` → 参照共有

2. **fini伝播システム**
   - 自動的な依存オブジェクトクリーンアップ
   - クリーンアップチェーンの可視化

3. **weak参照システム**
   - `WeakBox.is_alive()` → BoolBox
   - 循環参照の自動検出・回避

4. **メモリ効率監視**
   - `Box.memory_footprint()` → IntegerBox
   - コピー発生の検出・警告

---

## 🎯 **期待される成果**

### パフォーマンス目標
- **Tinyproxy**: HTTP転送でのゼロコピー率 90%以上
- **Chip-8**: 60FPSエミュレーション + fini伝播の完全動作
- **kilo**: 1MB+ファイル編集でのメモリ効率 95%以上

### 学習効果
- **Copilot**: 大規模Nyashアプリケーション開発経験
- **開発者**: 新メモリ管理機能の実用性確認
- **コミュニティ**: Nyashでの実用アプリケーション事例

---

## 📅 **実装計画**

### Phase 10.1: Tinyproxy実装 (1週間)
- HTTPプロキシ基本機能
- ゼロコピー判定API実装・テスト

### Phase 10.2: Chip-8実装 (1週間)  
- エミュレーター基本機能
- fini伝播・weak参照の実戦テスト

### Phase 10.3: kilo実装 (1週間)
- テキストエディター基本機能
- メモリ効率監視システム

### Phase 10.4: 統合テスト・最適化 (1週間)
- 3アプリケーション同時実行テスト
- パフォーマンス分析・改善

---

## 🚀 **この移植プロジェクトの意義**

1. **実用性の実証**: Nyashで実際のアプリケーションが作れることを証明
2. **新機能の検証**: ゼロコピー・fini・weakの実戦テスト  
3. **開発体験の向上**: Copilotとの協調開発での生産性検証
4. **エコシステム拡充**: Nyashアプリケーションの具体例提供

**この移植が成功すれば、Nyashは「実用的なプログラミング言語」として確立されます！** 🎉