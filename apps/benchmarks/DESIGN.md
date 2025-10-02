# HakoRune Benchmark Design - 商用レベルのベンチマーク体系

**設計者**: ChatGPT Pro
**日付**: 2025-10-02
**ステータス**: 設計フェーズ（Phase 15.8+）

このドキュメントは、HakoRune（Nyash）の**商用アプリケーション級ベンチマークシステム**の設計を記述します。

---

## 📋 目次

1. [全体方針](#全体方針商用アプリ級のベンチ設計)
2. [推奨ディレクトリ構成](#推奨ディレクトリ)
3. [ベンチ姿勢](#ベンチ姿勢-これを満たすと商用レベル)
4. [ターゲット比較マトリクス](#ターゲット比較マトリクス例キーワードのみ)
5. [代表ベンチ案](#代表ベンチ案52本から厳選)
6. [ハーネス設計](#ハーネスcli仕様と-json-出力)
7. [ベンチランナ雛形](#ベンチランナ雛形harnessbench_runnernyash)
8. [WASM/PHI/Effects/Contracts](#wasmphieffectscontracts-の特別考慮)
9. [CI統合](#ci-で市販アプリっぽくするポイント)
10. [実装ロードマップ](#実装ロードマップ)

---

## 全体方針（商用アプリ級のベンチ設計）

### 🎯 コンセプト

**"ライン（VM / LLVM / WASM）が動き始めた今が商用レベルのベンチ体系を一気に設計するベストタイミング"**

### 🏗️ 二層構造

1. **ベンチ本体（workload）**: 小さな Hakorune プログラム
   - Everything is Box 原則準拠
   - `flow Main.main` または `static box Main` エントリーポイント
   - 明確な期待値・計測可能な出力

2. **ハーネス（harness）**: 実行・計測・集計
   - 実行時間計測（ns精度）
   - メモリ使用量（RSS、Box割り当て数）
   - JSON形式での結果出力
   - CI比較・可視化対応

### 🔄 スイッチで比較

以下の軸で性能を比較：

- **バックエンド**: VM / LLVM（O0/O2/O3） / WASM（AOT/JIT）
- **決定性**: ON / OFF
- **Effects**: warn / enforce
- **Contracts**: warn / enforce
- **PHI検証**: ON / OFF

### 📚 目的別スイート

| スイート | 目的 | サイズ |
|---------|------|--------|
| **Micro** | 命令/構造要素を測る | 小（<100行） |
| **Runtime** | メモリ/GC/所有権 | 中（100-500行） |
| **Interop** | FFI/Plugin/Python連携 | 中 |
| **Macro** | アプリケーション寄り | 大（500行+） |

---

## 推奨ディレクトリ

```
apps/benchmarks/
├── README.md                  # 現状の使い方（既存）
├── DESIGN.md                  # 本ドキュメント（設計）
│
├── harness/                   # ランナ & 集計
│   ├── bench_runner.nyash     # メインランナ
│   ├── stats.nyash            # 統計計算
│   └── json_builder.nyash     # JSON出力
│
├── config/
│   ├── matrix.toml            # ターゲット & フラグの組み合わせ
│   └── thresholds.toml        # 回帰判定しきい値
│
├── micro/                     # 純アルゴ・構文・IRストレス
│   ├── 01_counter.nyash       # ✅ 既存
│   ├── 02_fibonacci.nyash     # ✅ 既存
│   ├── 03_prime_check.nyash   # ✅ 既存
│   ├── 04_branch_mispredict.nyash
│   ├── 05_call_indirect.nyash
│   ├── 06_struct_field.nyash
│   ├── 07_map_ops.nyash
│   ├── 08_string_concat.nyash
│   ├── 09_memory_alloc.nyash
│   ├── 10_phi_stress_if.nyash
│   └── 11_phi_stress_loop.nyash
│
├── runtime/                   # ランタイム機能
│   ├── 20_json_parse.nyash
│   ├── 21_json_stringify.nyash
│   ├── 22_regex_match.nyash
│   ├── 23_sort_large.nyash
│   └── 24_matrix_mul_simd.nyash
│
├── interop/                   # 相互運用
│   ├── 30_plugin_call_overhead.nyash
│   ├── 31_fs_read_write.nyash
│   ├── 32_http_stub.nyash
│   ├── 33_python_call_pulse.nyash
│   ├── 34_capability_enforce_overhead.nyash
│   └── 35_contract_check_overhead.nyash
│
├── macro/                     # アプリケーション
│   ├── 40_markdown_render_like.nyash
│   ├── 41_kv_store_workload.nyash
│   └── 42_pipeline_channel.nyash
│
└── results/                   # 結果保存
    ├── latest.json            # 最新結果
    └── history.jsonl          # CIで1行1JSON追記
```

**注**: `micro/01-03` は既存ファイルを活用。`harness/`以降は将来実装。

---

## ベンチ「姿勢」— これを満たすと"商用レベル"

### 1. 🔥 温身（Warmup）

JIT/キャッシュの助走時間を捨てる。

- **N回ウォームアップ** + **M回計測**
- 例: `--warmup=10 --repeat=50`

### 2. 📊 統計

単純な平均だけでなく、分布を把握。

- 平均（mean）
- 中央値（median）
- 標準偏差（σ）
- パーセンタイル（95p, 99p）

### 3. 🔄 再現性

`--deterministic` 時に完全再現可能。

- Seed記録
- 環境変数記録
- コミットハッシュ記録
- **スナップショット再現**

### 4. 🚨 回帰判定

前回比の劣化で自動失敗。

- 例: +10%劣化で **Fail**
- 閾値は `thresholds.toml` で設定
- 黄色（warning）と赤（fail）の2段階

### 5. 📝 構成ログ

すべての設定をJSONに同梱。

- ターゲット（vm/llvm/wasm）
- 最適化レベル（O0/O2/O3）
- Effects/Contracts モード
- PHI検証 ON/OFF

---

## ターゲット比較マトリクス例（キーワードのみ）

| target | opt      | jit     | det    | effects      | contracts    |
|--------|----------|---------|--------|--------------|--------------|
| vm     | -        | -       | on/off | warn/enforce | warn/enforce |
| llvm   | O0/O2/O3 | AOT     | on/off | warn/enforce | warn/enforce |
| wasm   | O2       | JIT/AOT | on/off | warn/enforce | warn/enforce |

**組み合わせ例**:
- `vm:det=on:effects=warn:contracts=enforce`
- `llvm:O2:det=off:effects=enforce:contracts=warn`
- `wasm:JIT:det=on:effects=enforce:contracts=enforce`

---

## 代表ベンチ案（52本から厳選）

### Micro（言語/IRの素性を測る）

| # | 名前 | 説明 | ステータス |
|---|------|------|-----------|
| 01 | counter | シンプルカウンター | ✅ 実装済 |
| 02 | fibonacci | フィボナッチ数列 | ✅ 実装済 |
| 03 | prime_check | 素数判定 | ✅ 実装済 |
| 04 | branch_mispredict | if分岐の偏りパターン差 | 📋 計画中 |
| 05 | call_indirect | 仮想呼び出し/多態ディスパッチ | 📋 計画中 |
| 06 | struct_field | Boxフィールドアクセス | 📋 計画中 |
| 07 | map_ops | MapBox操作 | 📋 計画中 |
| 08 | string_concat | 文字列連結（rope化なし） | 📋 計画中 |
| 09 | memory_alloc | 小/中/大アロケ混在 | 📋 計画中 |
| 10 | phi_stress_if | incoming=2/8/32段階 | 📋 計画中 |
| 11 | phi_stress_loop | 自己参照・深いループ巣 | 📋 計画中 |

### Runtime（ランタイム機能）

| # | 名前 | 説明 | サイズ |
|---|------|------|--------|
| 20 | json_parse | JSON解析 | 50KB/1MB/10MB |
| 21 | json_stringify | JSON文字列化 | 同上 |
| 22 | regex_match | 正規表現マッチ | 短/中/長テキスト |
| 23 | sort_large | 大規模ソート | 10^5/10^6要素 |
| 24 | matrix_mul_simd | 行列乗算（SIMD） | 64×64/256×256 |

### Interop / Effects（相互運用）

| # | 名前 | 説明 |
|---|------|------|
| 30 | plugin_call_overhead | Nullプラグイン呼び出し往復 |
| 31 | fs_read_write | ファイルI/O（1MB×100回） |
| 32 | http_stub | HTTPスタブ（no network） |
| 33 | python_call_pulse | PyFunctionBox.exec 1/100回 |
| 34 | capability_enforce_overhead | warn→enforce切替 |
| 35 | contract_check_overhead | pre/post 無→軽→重 |

### Macro（アプリケーション寄り）

| # | 名前 | 説明 |
|---|------|------|
| 40 | markdown_render_like | パーサ＋レンダリング |
| 41 | kv_store_workload | put/get混在 90/10 |
| 42 | pipeline_channel | 3段パイプ、並行度1/4/8 |

---

## ハーネス：CLI仕様と JSON 出力

### CLI仕様（例）

```bash
hako bench run micro/11_phi_stress_loop.nyash \
  --target=vm,llvm:O2,wasm:JIT \
  --repeat=50 --warmup=10 \
  --deterministic=on \
  --effects=enforce --contracts=warn \
  --out=apps/benchmarks/results/latest.json
```

### JSON スキーマ

```json
{
  "suite": "micro/11_phi_stress_loop",
  "commit": "abc1234",
  "timestamp": "2025-10-01T12:34:56+09:00",
  "env": {
    "cpu": "M3",
    "os": "macOS 15",
    "hakorune": "0.10.5"
  },
  "config": {
    "targets": [
      {"name": "vm", "opt": null, "jit": false},
      {"name": "llvm", "opt": "O2", "jit": false},
      {"name": "wasm", "opt": "O2", "jit": true}
    ],
    "warmup": 10,
    "repeat": 50,
    "deterministic": true,
    "effects": "enforce",
    "contracts": "warn",
    "phi_verify": true
  },
  "results": [
    {
      "target": "vm",
      "metrics": {
        "ns_avg": 1234567,
        "ns_median": 1200000,
        "ns_p95": 1400000,
        "ns_p99": 1500000,
        "rss_mb": 32.1,
        "alloc_boxes": 12000,
        "effect_calls": 0
      }
    },
    {
      "target": "llvm:O2",
      "metrics": {
        "ns_avg": 890000,
        "ns_median": 880000,
        "ns_p95": 950000,
        "ns_p99": 1000000,
        "rss_mb": 28.5,
        "alloc_boxes": 12000,
        "effect_calls": 0
      }
    }
  ],
  "regression": {
    "vm_vs_baseline": "+2.3%",
    "llvm_vs_baseline": "-5.1%",
    "status": "PASS"
  }
}
```

### history.jsonl

上記JSONを1行ずつ追記（CI比較用）。

```jsonl
{"suite":"micro/01_counter","commit":"abc1234",...}
{"suite":"micro/01_counter","commit":"def5678",...}
{"suite":"micro/02_fibonacci","commit":"abc1234",...}
```

---

## ベンチランナ（雛形：`harness/bench_runner.nyash`）

### 擬似コード（Hakorune 風）

```nyash
// Timer Box - Monotonic clock
box Timer {
    birth() {
        // 初期化
    }

    now_ns() -> IntegerBox {
        // 現在時刻（ナノ秒）を返す
        // ExternCall("clock_gettime", CLOCK_MONOTONIC)
    }
}

// Stats Box - 統計計算
box Stats {
    samples: ArrayBox

    birth() {
        me.samples = new ArrayBox()
    }

    push(ns: IntegerBox) {
        me.samples.push(ns)
    }

    mean() -> IntegerBox {
        local sum, count, avg
        sum = 0
        count = me.samples.length()

        local i
        i = 0
        loop(i < count) {
            sum = sum + me.samples.at(i)
            i = i + 1
        }

        avg = sum / count
        return avg
    }

    p95() -> IntegerBox {
        // ソートして95パーセンタイル取得
        local sorted
        sorted = me.samples.sort()
        local idx
        idx = (sorted.length() * 95) / 100
        return sorted.at(idx)
    }
}

// Runner Box - ベンチマーク実行
box Runner {
    target: StringBox

    birth(target: StringBox) {
        me.target = target
    }

    set_flags(det: IntegerBox, effects: StringBox, contracts: StringBox) {
        // 環境変数を設定
        // ENV["NYASH_DETERMINISTIC"] = det
        // ENV["NYASH_EFFECTS"] = effects
        // ENV["NYASH_CONTRACTS"] = contracts
    }

    run_once(file: StringBox) -> IntegerBox {
        local timer, t0, t1, rc
        timer = new Timer()

        t0 = timer.now_ns()
        // ExecBox::run(file) - VM/LLVM/WASM切替実行
        t1 = timer.now_ns()

        return t1 - t0
    }

    run(file: StringBox, warmup: IntegerBox, repeat: IntegerBox) -> Stats {
        // Warmup
        local i
        i = 0
        loop(i < warmup) {
            me.run_once(file)
            i = i + 1
        }

        // 計測
        local stats
        stats = new Stats()
        i = 0
        loop(i < repeat) {
            local ns
            ns = me.run_once(file)
            stats.push(ns)
            i = i + 1
        }

        return stats
    }
}

// Main Entry
static box Main {
    main() {
        local cfg
        // cfg = parse_args(args) - 実装予定

        local out
        // out = JsonBuilder::new() - 実装予定

        // 各ターゲットでベンチ実行
        local runner
        runner = new Runner("vm")

        local stats
        stats = runner.run("micro/01_counter.nyash", 10, 50)

        print("Mean: " + stats.mean())
        print("P95: " + stats.p95())

        // out.write(cfg.out_path)
    }
}
```

### 計測補助フック

以下をVM/ランタイムに実装：

- `mem_rss_mb()`: RSS（Resident Set Size）取得
- `alloc_boxes()`: Box割り当て総数
- `effect_calls()`: Effect呼び出し回数

---

## WASM/PHI/Effects/Contracts の特別考慮

### WASM ラインの注意

#### 起動オーバーヘッドとホットループ分離

- **起動オーバーヘッド**: ランタイム起動/コンパイル時間
- **ホットループ**: 実際の実行時間

#### ホスト境界測定

`interop/30_plugin_call_overhead.nyash` で往復N回計測。

#### SIMD

`runtime/24_matrix_mul_simd.nyash` で WASM SIMD 拡張 ON/OFF 比較。

### PHI/SSA を「見える化」

#### phi_stress_if.nyash

incoming=2/4/8/32 のマージを段階的に追加。

```nyash
static box Main {
    main() {
        local x, i
        x = 0
        i = 0

        loop(i < 10000) {
            if (i & 7 == 0) { x = x + 1 }
            else if (i & 7 == 1) { x = x + 2 }
            else if (i & 7 == 2) { x = x + 3 }
            else if (i & 7 == 3) { x = x + 4 }
            else if (i & 7 == 4) { x = x + 5 }
            else if (i & 7 == 5) { x = x + 6 }
            else if (i & 7 == 6) { x = x + 7 }
            else { x = x + 8 }

            i = i + 1
        }

        print("Result: " + x)
    }
}
```

#### phi_stress_loop.nyash

自己参照・多重ループネスト（深さ3まで）。

### 計測指標

- **配線時間**: PHI命令生成時間
- **Verifier時間**: PHI検証時間
- **実行時間**: 実際の実行時間

モード:
- `--phi_verify=on/off`
- `--deterministic=on`

### Effects / Contracts のオーバーヘッド

#### capability_enforce_overhead.nyash

`warn` と `enforce` 切替で差分計測。1万回の no-op effect 通過。

#### contract_check_overhead.nyash

`pre/post` 無し→軽量式→重め式 の3段階。

---

## CI で"市販アプリっぽく"するポイント

### 1. Fail fast ではなく「退色」

- 小さな劣化: **黄色（warning）**
- しきい値超え: **赤（fail）**

### 2. トレンド表示

`history.jsonl` を可視化（簡易SVG生成）。

### 3. バリアント一括

`matrix.toml` の組合せを全回し→`latest.json` にマージ。

### 4. artifact

各ターゲットのIR/objサイズも記録（IR増で遅くなる兆候を早期検知）。

---

## 既存3本の活用方針

### counter（01_counter.nyash）

**現状**: 短時間すぎる（3-4ms）

**改善案**:
- ループ回数自動調整
- ターゲット別に100msを目安に繰り返し
- ウォームアップ追加

### fibonacci（02_fibonacci.nyash）

**現状**: ループ版のみ

**拡張案**:
- 再帰版追加 → **call/branch 比率**比較
- 入力サイズ変動（n=10/20/30）

### prime（03_prime_check.nyash）

**現状**: 1つの入力のみ

**拡張案**:
- 入力セット複数プリセット（1e5 / 1e6 / 1e7）
- **計測時間スケール**を揃える

---

## 実装ロードマップ

### Phase 1: 基盤構築（最優先）✅ 進行中

- [x] 既存3ベンチマーク整備（counter/fibonacci/prime）
- [x] `apps/benchmarks/` ディレクトリ作成
- [x] README.md作成（使い方）
- [x] DESIGN.md作成（本ドキュメント）
- [ ] `harness/bench_runner.nyash` 最小実装
- [ ] `config/matrix.toml` テンプレート

### Phase 2: PHI検証強化

- [ ] `micro/10_phi_stress_if.nyash`
- [ ] `micro/11_phi_stress_loop.nyash`
- [ ] PHI計測フック実装

### Phase 3: ランタイム拡張

- [ ] `runtime/20_json_parse.nyash`
- [ ] `runtime/21_json_stringify.nyash`
- [ ] メモリ計測フック実装

### Phase 4: Interop/Effects

- [ ] `interop/30_plugin_call_overhead.nyash`
- [ ] `interop/34_capability_enforce_overhead.nyash`
- [ ] `interop/35_contract_check_overhead.nyash`

### Phase 5: CI統合

- [ ] `results/history.jsonl` 自動記録
- [ ] 回帰判定ロジック
- [ ] トレンド可視化

### Phase 6: WASM完全対応

- [ ] WASM JIT/AOT分離計測
- [ ] SIMD ON/OFF比較
- [ ] ホスト境界オーバーヘッド計測

---

## まず入れる最小TODO（すぐ楽になる順）

### 優先度1（即座に着手）

1. ✅ **既存3本の整備**（完了）
2. **`harness/bench_runner.nyash`**（上の雛形）
3. **`config/matrix.toml`**（vm/llvm:O2/wasm:JIT + det ON/OFF）
4. **`results/history.jsonl` ログ化**（CIの毎回追記）

### 優先度2（PHI品質保証）

5. **`micro/10_phi_stress_if.nyash`**
6. **`micro/11_phi_stress_loop.nyash`**
7. PHI計測フック

### 優先度3（実用ベンチ）

8. **`runtime/20_json_parse.nyash`**
9. **`interop/30_plugin_call_overhead.nyash`**

---

## 参考リンク

- **現状のREADME**: [README.md](./README.md)
- **LLVM Build Quickstart**: [CLAUDE.md](../../CLAUDE.md)
- **Phase 15.8計画**: [docs/development/roadmap/phases/phase-15.8/](../../docs/development/roadmap/phases/phase-15.8/)

---

## 変更履歴

| 日付 | 変更内容 |
|------|----------|
| 2025-10-02 | 初版作成（ChatGPT Pro設計統合） |

---

🌿 **wasm-development branch**: 商用レベルのベンチマーク設計完了！
