# 評価計画

## 🎯 評価の目的

1. **実証**: MIR15で本当にGUIアプリが動くことを証明
2. **性能**: 最小命令セットでも実用的な性能を達成
3. **等価性**: VM/JIT/AOT/WASMが同一動作することを検証
4. **拡張性**: 新機能追加の容易さを実証

## 📊 評価軸（ChatGPT5提案ベース）

### 1. 命令カバレッジ分析

```yaml
実験内容:
  - GUIデモアプリ実行時の命令使用頻度測定
  - 各命令の使用率をヒストグラムで可視化
  
期待結果:
  - 全15命令が実際に使用される
  - BoxCall/ExternCallが高頻度（Box哲学の実証）
```

### 2. バックエンド直交性検証

```yaml
実験内容:
  - 同一プログラムをVM/JIT/AOT/WASMで実行
  - 出力差分の検証（ログ、ハッシュ値）
  
測定項目:
  - 実行結果の一致率: 100%期待
  - 性能差: VM基準で±50%以内
```

### 3. GUI動作実証

```yaml
対象OS:
  - Ubuntu 22.04 LTS
  - Windows 11
  
デモアプリ:
  - Hello World（ウィンドウ＋ボタン）
  - Canvas描画（図形、テキスト）
  - イベント処理（クリック、キー入力）
  
検証方法:
  - スクリーンショット取得
  - イベントログ記録
  - 自動UIテスト
```

### 4. 開発効率性

```yaml
測定項目:
  - 新Box追加に必要な行数
  - 新バックエンド追加の工数
  - ビルド時間
  
比較対象:
  - LLVM（数百命令）
  - JVM（200+命令）
  - Lua（50命令）
```

## 🧪 具体的な実験手順

### 実験1: Hello GUI

```nyash
# hello-gui.nyash
box HelloApp from GuiBox {
    render() {
        return me.window("MIR15 GUI Demo", [
            me.label("15命令で動くGUI！"),
            me.button("Click Me", () => {
                print("Button clicked!")
            }),
            me.canvas(300, 200, (ctx) => {
                ctx.fillRect(50, 50, 200, 100, "#FF0000")
                ctx.drawText(100, 100, "Nyash", "#FFFFFF")
            })
        ])
    }
}

new HelloApp().run()
```

**実行コマンド**:
```bash
# VM実行
./target/release/nyash apps/gui/hello-gui.nyash

# JIT実行
./target/release/nyash --backend vm --jit apps/gui/hello-gui.nyash

# AOT実行
./target/release/nyash --aot apps/gui/hello-gui.nyash -o hello-gui
./hello-gui

# WASM実行
./target/release/nyash --compile-wasm apps/gui/hello-gui.nyash
```

### 実験2: 命令使用分析

```bash
# 命令プロファイリング有効化
NYASH_MIR_PROFILE=1 ./target/release/nyash apps/gui/hello-gui.nyash

# 結果をJSON出力
NYASH_MIR_PROFILE_JSON=1 ./target/release/nyash apps/gui/hello-gui.nyash > profile.json
```

### 実験3: 性能ベンチマーク

```yaml
ベンチマーク項目:
  1. 起動時間（ウィンドウ表示まで）
  2. イベント応答時間（クリック→処理）
  3. 描画性能（FPS測定）
  4. メモリ使用量
```

## 📈 期待される結果

### 定量的結果

1. **命令数**: 15命令で全機能実現
2. **実装規模**: ~4,000行（他言語の1/10以下）
3. **開発期間**: 30日（通常の1/12）
4. **性能**: 実用レベル（VM比でJIT 2-5倍高速）

### 定性的結果

1. **シンプルさ**: 学習曲線が緩やか
2. **保守性**: 変更の影響範囲が明確
3. **移植性**: 新環境への適応が容易
4. **拡張性**: Box追加で機能拡張

## 🔍 脅威の妥当性（Threats to Validity）

### 内的妥当性
- 最適化の配置がBox側に偏る可能性
- ベンチマークが単純すぎる可能性

### 外的妥当性
- より複雑なアプリでの検証が必要
- 他のドメイン（Web、ML等）での検証

### 構成妥当性
- 「実用的」の定義の明確化が必要
- 性能測定の公平性確保

## 🚀 実験実施計画

1. **Week 1**: 基本GUI Box実装
2. **Week 2**: デモアプリ作成・動作確認
3. **Week 3**: 性能測定・最適化
4. **Week 4**: データ分析・図表作成

## 📊 成果物

- 命令使用分布グラフ
- バックエンド比較表
- GUIスクリーンショット
- 性能ベンチマーク結果
- 実験再現用スクリプト一式