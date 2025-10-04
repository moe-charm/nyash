# 固定時間ベンチマーク実装ガイド

## 概要

TimerBox.now_ms()を使用した固定時間方式のベンチマークシステム実装完了（2025-10-04）。

## TimerBox実装

### Hakoソース

```hako
box TimerBox {
    birth() {
        // ExternCall経由でシステム時計使用
    }

    now_ms() {
        // コンパイラが自動的にExternCall(nyrt.time.now_ms)に変換
        return 0  // このreturnは無視される
    }
}
```

### MIR変換

コンパイラが`timer.now_ms()`呼び出しを自動的に以下のMIRに変換：

```json
{
  "args": [],
  "dst": 13,
  "func": "nyrt.time.now_ms",
  "name": "nyrt.time.now_ms",
  "op": "externcall"
}
```

## Backend実装

### VM Backend

**実装場所**: `src/backend/mir_interpreter/extern_adapter.rs`

```rust
"nyrt.time" => match method {
    "now_ms" => {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let millis = duration.as_millis() as i64;
        Ok(VMValue::Integer(millis))
    }
    _ => Err(format!("Unknown nyrt.time method: {}", method)),
},
```

**測定結果**: 166ms差分を正確に計測 ✅

### LLVM Backend

**実装場所**: `crates/hako_kernel/src/lib.rs`

```rust
#[export_name = "nyrt.time.now_ms"]
pub extern "C" fn hako_time_now_ms() -> i64 {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_millis(0));
    let millis = duration.as_millis();
    if millis > i64::MAX as u128 {
        i64::MAX
    } else {
        millis as i64
    }
}
```

**確認済み事項**:
- ✅ libhako_kernel.aにシンボル存在（121バイト）
- ✅ 実行可能ファイルに正しくリンク済み
- ✅ LLVM IRでexterncall正しく生成
- ✅ SystemTime::now()を呼び出し中

**重要な発見**:
- ループ100,000回は現代のCPUでは1ミリ秒未満で完了
- ミリ秒精度では時間差が0になる
- **解決**: ループ回数を10,000,000回に増加 → 測定可能 ✅

### WASM Backend

**実装場所**: `tools/wasm_runner.js`

```javascript
nyrt: {
  time: {
    now_ms: () => {
      return BigInt(Date.now());
    }
  }
}
```

**対応状況**: 実装済み、TimerBox動作確認済み ✅

## 固定時間方式ベンチマーク

### 設計方針

**固定回数方式**（旧方式）:
- N回実行して時間を測定
- 問題: 環境によって実行時間がばらつく

**固定時間方式**（新方式・推奨）:
- N秒間実行し続けて回数を測定
- ops/secで性能を測定
- 環境差による影響を最小化

### 実装例

```hako
static box Main {
    main() {
        local timer, start_time, end_time, current_time
        local iterations, duration_sec

        duration_sec = 5  // 5秒間測定
        timer = new TimerBox()

        start_time = timer.now_ms()
        end_time = start_time + (duration_sec * 1000)  // 秒→ミリ秒変換

        iterations = 0
        current_time = timer.now_ms()

        // 固定時間方式: end_timeまで繰り返し実行
        loop(current_time < end_time) {
            // ベンチマーク対象の処理
            iterations = iterations + 1
            current_time = timer.now_ms()
        }

        // 結果: iterations回を duration_sec秒で実行
        // ops/sec = iterations / (実際の経過秒)
        return 0
    }
}
```

### ベンチマークスクリプト

**場所**: `apps/benchmarks/harness/bench_runner.hako`

**メソッド**: `run_duration(file, duration_sec)`

**特徴**:
- end_timeまでループ継続
- MapBoxで結果構造化（iterations/duration_ms/ops_per_sec）
- DESIGN.md準拠の完全実装

## 実測結果

### VM版

- **空ベンチ**: 109,543 ops/sec（約10万回/秒）
- **sum_loop(100k)**: 5 ops/sec
  - 妥当性確認: 前回測定200ms/回 × 5 = 1000ms ✅

### LLVM版

- **実装**: `local_tests/bench_timer_llvm.hako`（5秒間測定方式）
- **動作確認**: exit=0（成功）✅

### WASM版

- 🔧 実装予定（次のステップ）

## トラブルシューティング

### LLVM版で時間差が0になる

**症状**: test返り値が1（diff <= 0）

**原因**: ループ回数が少なく、1ミリ秒未満で完了

**解決**:
1. ループ回数を10,000,000回以上に増加
2. または固定時間方式（5秒間ループ）を使用

### straceで時間関連システムコールが見えない

**原因**: SystemTime::now()はvDSO経由で高速化されている

**確認方法**:
- `nm`でシンボル確認
- `objdump -d`で逆アセンブル確認
- ループ回数増加で動作確認

## リファレンス

- **ExternCall Registry**: `tmp/externs_registry.json`
- **MIR JSON**: `tmp/nyash_cli_emit.json`
- **LLVM IR**: `/tmp/debug_ir.ll`
- **実装**: `local_tests/bench_timer_llvm.hako`

## 📊 言語対決ベンチマーク結果

### sum_loop言語比較（固定5秒測定, 2025-10-04）

| 言語        | Backend       | Ops/sec      | 相対速度       | 対C比 | 備考 |
|-------------|---------------|--------------|----------------|-------|------|
| C           | gcc -O3       | 58,012,004   | 1.00x (基準)   | 100% | ネイティブコンパイル最適化 |
| Python      | CPython 3.x   | 17,915,223   | 0.31x          | 31%  | C層委譲戦略 |
| **Ruby**    | **YARV 3.2**  | **11,178,680** | **0.19x**    | **19%** | すべてオブジェクト思想 |
| Nyash       | Rust VM       | 351,263      | 0.006x         | 0.6% | インタープリター妥当 |
| Nyash       | LLVM (harness)| N/A          | **失敗***      | -    | シンボル不足 |

**Ruby vs Python**: Rubyの方が62%の速度（命令数は少ないが1命令が重い）

**\*LLVM失敗原因**: `libhako_kernel.a`に`nyash.console.log`/`nyash.string.concat_si`シンボルが存在しないため、リンク失敗。

### 重要な発見

**性能比較**:
- ✅ C言語: 5,801万 ops/sec（基準、ネイティブコンパイル）
- ✅ Python: 1,792万 ops/sec（C の 31%）
  - **速度の秘密**: `time.time()`と整数演算(`+=`, `++`)はすべてC実装
  - このベンチマークは「PythonのVM層」ではなく「PythonのC実装層」を測定
  - つまり、Pythonインタープリターのオーバーヘッドはほぼ測定されていない
- ✅ Nyash VM: 35万 ops/sec（C の 0.6%、Python の 1/165）
  - インタープリターとして妥当なオーバーヘッド
  - ループごとに`timer.now_ms()`呼び出しコストを含む

**LLVM実行問題**:
- **現象**: `print()` + 文字列連結ありの`sum_loop_bench.hako`はリンク失敗
- **原因**: `libhako_kernel.a`に`nyash.console.log`/`nyash.string.concat_si`が未実装
- **回避策**: `print()`なし版（`sum_loop_bench_noprint.hako`）なら実行可能 ✅
  - ただし結果表示できないため、速度測定には使えない
  - `exit=0`で正常終了のみ確認可能

### ベンチマーク仕様

**テストコード**: `benchmarks/sum_loop_bench.{hako,py,c}`

**アルゴリズム** (全言語共通):
```
iterations = 0
sum = 0
start_time = now()
end_time = start_time + 5000ms

while (now() < end_time) {
    sum += iterations
    iterations++
    now()  // 毎回時刻確認
}

print("Iterations: " + iterations)
print("Ops/sec: " + iterations * 1000 / elapsed_ms)
```

**測定環境**:
- 固定5秒間実行
- 毎イテレーション時刻確認（公平性確保）
- 結果: iterations/秒 (ops/sec)

### 実行方法

```bash
# 全言語一括実行
bash benchmarks/run_language_shootout.sh

# 個別実行
gcc -O3 -o benchmarks/sum_loop_bench_c benchmarks/sum_loop_bench.c
./benchmarks/sum_loop_bench_c

python3 benchmarks/sum_loop_bench.py

./target/release/hako --backend vm benchmarks/sum_loop_bench.hako

# LLVM版（print()なし、結果表示なし、exit=0のみ確認）
NYASH_DISABLE_PLUGINS=1 NYASH_LLVM_USE_HARNESS=1 \
  ./target/release/hako --backend llvm benchmarks/sum_loop_bench_noprint.hako
```

### ベンチマークの限界と教訓

**1. Pythonベンチマークの注意点**:
- ❌ **誤解**: 「PythonがC言語の31%の速度！Pythonインタープリター速い！」
- ✅ **真実**: このベンチマークは`time.time()`（C実装）と整数演算（C実装）しか測定していない
- 📝 **教訓**: 言語ベンチマークは「何を測定しているか」を正確に理解すべき

**🧠 Pythonの賢い二層戦略**:
```
【Python層】制御フローのみ（軽量バイトコードVM）
  while/if/for → バイトコード（18命令/ループ）
  変数アクセス → 配列操作（2-3 CPU命令）
  ↓ (オーバーヘッド: 約9 CPU命令/バイトコード)
【C層】実際の処理（超高速ネイティブ実装）
  time.time() → システムコール
  整数演算(+=) → ネイティブCPU命令
  リスト/辞書 → 最適化C構造体
```

**実測データ** (バイトコード解析):

**Python**:
- ループ1回 = 18バイトコード命令
- 1,792万イテレーション/秒 = **3.2億バイトコード命令/秒**
- 1バイトコード ≈ 9 CPU命令（3GHz CPUの場合）
- Computed goto最適化（超高速ディスパッチ）

**Ruby**:
- ループ1回 = 14バイトコード命令（Pythonより少ない！）
- 1,118万イテレーション/秒 = **1.6億バイトコード命令/秒**
- 1バイトコード ≈ 20 CPU命令（推測）
- Switch文ベースVM（Pythonより重い）

**重要な発見**:
- ❌ 「Pythonは遅いインタープリター」← 誤解
- ✅ 「Pythonは軽量VM + 重い処理はC層に委譲」← Pythonの設計思想
- ✅ 「Rubyはすべてオブジェクト思想」← 純粋性 vs 速度のトレードオフ
- 📝 このベンチマークは**C層の速度**を測定（Python VM層はほぼ測定されていない）
- 📝 RubyはTime.now（メソッド呼び出し+オブジェクト生成）が重い

**2. 公平なベンチマークにするには**:
- Pythonリスト操作、辞書操作、文字列処理など、VM層が関わる処理を含める
- 関数呼び出し、属性アクセスなど、インタープリターのオーバーヘッドが大きい処理を含める
- 現状は「tight loop + integer arithmetic」のみ = C層ベンチマーク

**3. Nyash VMが遅い理由**:
- ループごとに`BoxCall(timer.now_ms())`を実行（メソッド解決コスト）
- MIR命令解釈オーバーヘッド
- Python版はC実装の`time.time()`を直接呼び出し（メソッド解決なし）

**4. LLVM版が使えない理由**:
- `libhako_kernel.a`は最小限の実装（`nyrt.time.now_ms`のみ）
- Box操作（StringBox等）やコンソール出力は未実装
- プラグインシステムが必要だが、LLVM実行時は未サポート

---

**更新日**: 2025-10-04
**Phase**: 3.5完了（言語対決ベンチ追加）
