# GPU Box 技術仕様（案）

## 🔧 技術的詳細

### GPU Box の要件

1. **純粋性**
   - 副作用を持たない
   - 外部状態に依存しない
   - 決定的な実行結果

2. **型制約**
   - GPU互換型のみ使用可能
   - ポインタ/参照の制限
   - 再帰呼び出し禁止

3. **メモリモデル**
   ```nyash
   // GPU メモリレイアウト
   gpu box ParticleArray {
       // Structure of Arrays (SoA) で自動配置
       positions: GPUBuffer<Float3>
       velocities: GPUBuffer<Float3>
       masses: GPUBuffer<Float>
   }
   ```

### MIR → GPU IR 変換

```
// Nyash MIR
BoxCall { 
    box_val: %particles,
    method: "update",
    args: [%deltaTime]
}

// ↓ 変換

// GPU IR（擬似コード）
kernel particle_update {
    params: [particles_ptr, deltaTime]
    threads: particles.count
    
    thread_body: {
        idx = thread_id()
        pos = load(particles.positions[idx])
        vel = load(particles.velocities[idx]) 
        new_pos = pos + vel * deltaTime
        store(particles.positions[idx], new_pos)
    }
}
```

### バックエンド対応

1. **CUDA** (NVIDIA GPU)
   - PTX生成
   - cuBLAS/cuDNN統合

2. **OpenCL** (クロスプラットフォーム)
   - SPIR-V生成
   - 各種GPU対応

3. **Vulkan Compute** (モダンAPI)
   - SPIR-V生成
   - モバイルGPU対応

4. **Metal** (Apple GPU)
   - Metal Shading Language
   - Apple Silicon最適化

### 最適化技術

1. **Box融合**
   ```nyash
   // これらの操作を1つのGPUカーネルに融合
   data.map(x => x * 2)
       .filter(x => x > 10)
       .reduce(+)
   ```

2. **メモリ合体アクセス**
   - Boxフィールドの最適配置
   - キャッシュ効率の最大化

3. **占有率最適化**
   - スレッドブロックサイズ自動調整
   - レジスタ使用量の制御

### エラー処理

```nyash
gpu box SafeDiv {
    @gpu
    divide(a, b) {
        if b == 0 {
            // GPU例外はCPU側で処理
            gpu.raise(DivisionByZeroError)
        }
        return a / b
    }
}
```

## 🔍 課題と解決策

### 課題1: デバッグの困難さ
**解決**: GPU実行トレース機能
```nyash
// デバッグモードでGPU実行を記録
local result = particles.gpuExecute("update", 0.016, debug: true)
print(result.trace)  // 各スレッドの実行履歴
```

### 課題2: CPU/GPU同期オーバーヘッド
**解決**: 非同期実行とパイプライン
```nyash
// GPU実行を非同期化
local future = particles.gpuExecuteAsync("update", 0.016)
// CPU側で他の処理を継続
doOtherWork()
// 必要な時に結果を取得
local result = await future
```

### 課題3: メモリ制限
**解決**: ストリーミング処理
```nyash
// 大規模データを分割処理
largeData.gpuStream(chunkSize: 1_000_000)
    .map(process)
    .collect()
```

## 🎓 学習曲線を下げる工夫

1. **自動GPU化**
   - コンパイラが自動的にGPU実行可能性を判定
   - ヒント表示: 「このBoxはGPU実行可能です」

2. **段階的移行**
   - 既存コードはCPUで動作保証
   - `@gpu`を追加するだけでGPU化

3. **プロファイリング支援**
   ```nyash
   // GPU実行の効果を可視化
   local profile = Profiler.compare(
       cpu: => particles.update(0.016),
       gpu: => particles.gpuExecute("update", 0.016)
   )
   print(profile.speedup)  // "GPU: 156.3x faster"
   ```