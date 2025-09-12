# Phase 15 セルフホスティング戦略 2025年9月版

## 🎯 セルフホスティングの段階的実現戦略

### 現在地
- ✅ v0 Nyパーサー（Ny→JSON IR）完成
- ✅ MIR生成基盤あり（Rust実装）
- 🚧 LLVM層改善中（ChatGPT5協力）
- 📅 NyコンパイラMVP計画中

### 君の提案の妥当性検証

## 📊 セルフホスティングの段階

### Stage 1: LLVM層の独立（最優先）✅
```
現在: Rustモノリス → MIR → LLVM → オブジェクト
提案: Rustモノリス → MIR(JSON) → [LLVM EXE] → ネイティブEXE
```

**実装詳細**：
1. `nyash-llvm-compiler` crateを分離
2. 入力：MIR（JSON/バイナリ）
3. 出力：ネイティブ実行ファイル
4. nyrtランタイムとのリンク

**メリット**：
- ビルド時間短縮（Rust側は軽量化）
- 独立したツールとして配布可能
- パイプライン明確化

### Stage 2: Nyashコンパイラ実装（現在計画中）✅
```
現在: Rustパーサー → MIR
提案: Nyashコンパイラ → AST/JSON → MIR生成層
```

**実装詳細**：
1. Nyashで再帰下降パーサー実装
2. AST構造をJSONで出力
3. 既存のMIR生成層に接続
4. `NYASH_USE_NY_COMPILER=1`で切替

**これでセルフホスティング達成！**
- Nyashで書いたコンパイラがNyashをコンパイル
- Rustコンパイラは不要に

### Stage 3: VM層のNyash実装（革新的）⚡
```
現在: Rust VM → MIR解釈
提案: Nyash VM → MIR解釈 → (必要時LLVM呼び出し)
```

**実装詳細**：
```nyash
box NyashVMBox {
    mir_module: MIRModuleBox
    pc_stack: ArrayBox
    value_stack: ArrayBox
    frame_stack: ArrayBox
    
    execute(mir_json) {
        me.mir_module = MIRModuleBox.parse(mir_json)
        me.runFunction("main")
    }
    
    runFunction(name) {
        local func = me.mir_module.getFunction(name)
        local frame = new FrameBox(func)
        me.frame_stack.push(frame)
        
        loop(frame.pc < func.instructions.length()) {
            local inst = func.instructions[frame.pc]
            me.executeInstruction(inst, frame)
            frame.pc = frame.pc + 1
        }
    }
}
```

**メリット**：
- **コンパイル不要**で即実行
- VMロジックを動的に変更可能
- デバッグ・実験が容易

## 🚀 実現順序の提案

### Phase 15.2: LLVM独立化
1. LLVM層をcrateに分離
2. MIR JSONインターフェース確立
3. スタンドアロンEXE生成

### Phase 15.3: Nyashコンパイラ
1. パーサー実装（Nyashで）
2. AST→MIRブリッジ
3. ブートストラップテスト

### Phase 15.4: VM層Nyash化
1. MIR解釈エンジン（基本13命令）
2. BoxCall/ExternCallブリッジ
3. パフォーマンス最適化（JIT連携）

## 💡 ABI移行タイミング

**LLVM独立化完了後が最適**理由：
1. インターフェース確定後に最適化
2. 独立EXEならABI変更の影響限定的
3. パフォーマンス測定してから判断

## 📋 検証結果

**君の提案は正しい！**

1. **LLVM EXE独立** → MIR JSONで疎結合
2. **Nyashコンパイラ** → AST/JSONでMIR生成
3. **セルフホスト完了** → Rustコンパイラ不要
4. **VM層Nyash化** → 究極の柔軟性

この順序なら：
- 段階的に実現可能
- 各段階で動作確認
- リスク最小化
- 最終的に完全セルフホスト

## 🎯 次のアクション

1. **LLVM crateの設計開始**
2. **MIR JSONスキーマ確定**
3. **Nyパーサー拡張計画**
4. **VMプロトタイプ設計**

これが現実的で革新的なロードマップにゃ！