# AI-Nyash Compact Notation Protocol (ANCP)
Status: Idea → Design Complete
Created: 2025-08-29
Updated: 2025-08-29 (Gemini & Codex分析統合、スモークテスト統合追加)
Priority: High
Category: AI Integration / Code Compression / Testing Infrastructure

## 概要
AIとNyashの効率的な通信のための圧縮記法プロトコル。予約語を1-2文字の記号に変換し、トークン数を大幅削減しながら、副次的にコード整形機能も提供する。さらに、既存のスモークテスト基盤との統合により、品質保証も自動化する。

## 動機
- AIのコンテキストウィンドウ制限への対応（50-90%削減目標）
- 人間が書いたコードをAIに効率的に渡す必要性
- Nyashパーサーとの100%同期による確実性
- 圧縮→展開プロセスでの自動コード整形
- 既存テスト基盤との統合による品質保証

## 設計詳細

### 基本的な変換例
```nyash
// 元のコード
box Cat from Animal {
    init { name, age }
    birth() { 
        me.name = "Tama" 
    }
}

// ANCP圧縮後
$Cat@Animal{#{name,age}b(){m.name="Tama"}}

// プロトコルヘッダー付き
;ancp:1.0 nyash:0.5;
$Cat@Animal{#{name,age}b(){m.name="Tama"}}
```

### 記号マッピング体系
```
【型・オブジェクト操作系】
box      → $   # Box定義
new      → n   # インスタンス生成
me       → m   # 自己参照

【構造・スコープ系】
from     → @   # 継承/デリゲーション
init     → #   # フィールド初期化
static   → S   # 静的定義
local    → l   # ローカル変数

【制御フロー系】
if       → ?   # 条件分岐
else     → :   # else節
loop     → L   # ループ
return   → r   # 戻り値

【メソッド系】
birth    → b   # コンストラクタ
override → O   # オーバーライド
```

## 技術的実装設計（Rust）

### 1. Lexer拡張設計
```rust
// 双方向トランスコーダー
pub struct AncpTranscoder {
    // 静的マップ（phf使用でO(1)ルックアップ）
    nyash_to_ancp: phf::Map<&'static str, &'static str>,
    ancp_to_nyash: phf::Map<&'static str, &'static str>,
    
    // 位置情報マッピング（Source Map相当）
    span_map: SpanMap,
    
    // バージョン情報
    version: AncpVersion,
}

// LosslessToken層（空白・コメント保持）
struct Token {
    kind: TokenKind,
    span: Span,
    raw: RawLexemeId,
    leading_trivia: Trivia,
    trailing_trivia: Trivia,
}

// Dialect検出
enum Dialect {
    Nyash,
    ANCP(Version),
}
```

### 2. トークン効率最適化
```python
# tiktoken実測スクリプト
import tiktoken
enc = tiktoken.get_encoding("cl100k_base")  # GPT-4等

def measure_tokens(code):
    return len(enc.encode(code))

# 実測して最適な記号を選定
original = 'box Cat from Animal { init { name } }'
ancp = '$Cat@Animal{#{name}}'

reduction = 1 - (measure_tokens(ancp) / measure_tokens(original))
print(f"Token reduction: {reduction:.1%}")
```

### 3. 双方向変換保証テスト戦略
```rust
// プロパティベーステスト
#[test]
fn roundtrip_property(code in any_valid_nyash()) {
    let transcoder = AncpTranscoder::new();
    let (ancp, _) = transcoder.encode(&code)?;
    let (restored, _) = transcoder.decode(&ancp)?;
    assert_eq!(code, restored);  // 完全一致
}

// AST同値性テスト
#[test]
fn ast_equivalence(code: &str) {
    let ast_original = parse_nyash(code);
    let ancp = transcode_to_ancp(code);
    let ast_ancp = parse_ancp(ancp);
    assert_ast_eq!(ast_original, ast_ancp);
}

// ファジングテスト
#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = transcode_roundtrip(s);
    }
});
```

### 4. デバッグ体験の維持
```rust
// ハイブリッドエラー表示
fn format_error(span: Span, ancp_src: &str, nyash_src: &str) {
    let ancp_token = get_token_at(ancp_src, span);
    let nyash_token = map_to_original(ancp_token);
    eprintln!("Error at {} ({})", ancp_token, nyash_token);
    // 例: "Error at @f (from)"
}

// IDE統合（LSP）
impl LanguageServer for NyashLsp {
    fn hover(&self, position: Position) -> Hover {
        // ANCP記号にホバーすると元の予約語を表示
        let original = self.map_to_original(position);
        Hover { contents: format!("ANCP: {} → Nyash: {}", ...) }
    }
}
```

## スモークテストとの統合

### 統一検証フレームワーク
```nyash
// 検証用Box - スモークテストとANCP検証を統合
box TestValidatorBox {
    init { patterns, mode, transcoder }
    
    birth() {
        me.patterns = new ArrayBox()
        me.mode = "smoke"  // smoke/full/ancp/roundtrip
        me.transcoder = new AncpTranscoder()
    }
    
    // 共通のパターンチェック（既存スモークテストの仕組み）
    checkPatterns(output) {
        loop(me.patterns.hasNext()) {
            local pattern
            pattern = me.patterns.next()
            
            if (not output.contains(pattern)) {
                return false
            }
        }
        return true
    }
    
    // Nyashコード実行して検証（既存）
    validateScript(scriptPath) {
        local output
        output = me.runNyash(scriptPath)
        return me.checkPatterns(output)
    }
    
    // ANCP変換して実行・検証（新規）
    validateAncp(ancpPath) {
        // 1. ANCP → Nyash変換
        local nyashCode
        nyashCode = me.transcoder.decode(ancpPath)
        
        // 2. 実行（既存の実行パスを再利用）
        local output
        output = me.runNyashCode(nyashCode)
        
        // 3. パターン検証（既存のcheckPatternsを再利用！）
        return me.checkPatterns(output)
    }
    
    // 双方向テスト（ANCP特有）
    validateRoundtrip(scriptPath) {
        // 元の実行結果
        local originalOutput
        originalOutput = me.runNyash(scriptPath)
        
        // Nyash → ANCP → Nyash → 実行
        local ancp
        ancp = me.transcoder.encode(scriptPath)
        local restored
        restored = me.transcoder.decode(ancp)
        local roundtripOutput
        roundtripOutput = me.runNyashCode(restored)
        
        // 実行結果が同一か検証
        return originalOutput == roundtripOutput
    }
}
```

### 統合スモークテストスクリプト
```bash
#!/bin/bash
# 統一スモークテスト（Nyash & ANCP対応）

# 共通の検証関数
run_test() {
    local mode=$1    # nyash/ancp/roundtrip
    local file=$2    # ファイルパス
    local pattern=$3 # 期待パターン
    
    case "$mode" in
        "ancp")
            # ANCP → Nyash変換 → 実行
            output=$(./target/release/nyash --from-ancp "$file" 2>&1)
            ;;
        "roundtrip")
            # Nyash → ANCP → Nyash → 実行
            ancp=$(./target/release/nyash --to-ancp "$file")
            output=$(echo "$ancp" | ./target/release/nyash --from-ancp - 2>&1)
            ;;
        *)
            # 通常のNyash実行
            output=$(./target/release/nyash "$file" 2>&1)
            ;;
    esac
    
    # 共通のパターン検証（既存のスモークテストと同じ！）
    echo "$output" | grep -q "$pattern"
}

# 自動テスト生成
generate_ancp_tests() {
    for test in examples/*.nyash; do
        # Nyashテストから自動的にANCPテストを生成
        ./tools/nyash2ancp "$test" > "${test%.nyash}.ancp"
        
        # 同じ期待値パターンで両方をテスト
        pattern=$(extract_expected_pattern "$test")
        
        echo "Testing $test (Nyash)..."
        run_test nyash "$test" "$pattern"
        
        echo "Testing ${test%.nyash}.ancp (ANCP)..."
        run_test ancp "${test%.nyash}.ancp" "$pattern"
        
        echo "Testing roundtrip for $test..."
        run_test roundtrip "$test" "$pattern"
    done
}
```

### CI/CD統合
```yaml
# .github/workflows/smoke.yml への追加
jobs:
  smoke-ancp:
    runs-on: ubuntu-latest
    needs: smoke  # 通常のスモークテスト後に実行
    env:
      CARGO_TERM_COLOR: always
      NYASH_DISABLE_PLUGINS: '1'
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      
      - name: Build with ANCP support
        run: cargo build --release -j32 --features ancp
      
      - name: Run ANCP roundtrip tests
        run: |
          # すべてのサンプルでANCP往復テスト
          for f in examples/*.nyash; do
            echo "Testing ANCP roundtrip: $f"
            ./tools/test_ancp_roundtrip.sh "$f"
          done
      
      - name: Run ANCP smoke tests
        run: |
          # 既存のスモークテストをANCPモードでも実行
          ANCP_MODE=1 bash tools/smoke_phase_10_10.sh
      
      - name: Measure token reduction
        run: |
          # 実際のトークン削減率を測定・報告
          ./tools/measure_ancp_efficiency.sh examples/*.nyash
```

### 共通化のメリット

1. **コード再利用**: 
   - 既存のパターン検証ロジックをそのまま活用
   - `grep`ベースの柔軟な検証方法を継承

2. **テスト網羅性**:
   - すべてのNyashテストが自動的にANCPテストにもなる
   - 双方向変換の正確性を継続的に検証

3. **開発効率**:
   - 新機能追加時、Nyashテストを書けばANCPテストも自動生成
   - スモークテストの期待値パターンを共有

4. **品質保証**:
   - ANCP変換によるセマンティクスの保持を自動検証
   - AIが生成したANCPコードも同じ基準で検証可能

5. **将来の拡張性**:
   - WebAssembly出力との統合も同じフレームワークで可能
   - 他の中間表現（MIR等）も同様に統合可能

## 実装ロードマップ（更新版）

### Phase 1: 基礎実装（2週間）
- [ ] tiktoken実測による最適記号選定
- [ ] 固定辞書（20語）でTranscoder実装
- [ ] 基本的な往復テスト
- [ ] **既存スモークテストとの基本統合**

### Phase 2: パーサー統合（3週間）
- [ ] Lexer拡張（Dialect検出）
- [ ] TokenKind統一化
- [ ] SpanMapによるエラー位置マッピング
- [ ] **TestValidatorBox実装**

### Phase 3: 品質保証（2週間）
- [ ] プロパティベーステスト
- [ ] ファジングテスト
- [ ] ベンチマーク（性能測定）
- [ ] **自動ANCP テスト生成スクリプト**

### Phase 4: ツール統合（3週間）
- [ ] CLI: --view=ancp|nyash|hybrid
- [ ] VSCode拡張（ホバー変換）
- [ ] プロトコルヘッダー処理
- [ ] **CI/CDパイプライン統合**

### Phase 5: AI統合（4週間）
- [ ] 並列コーパス生成
- [ ] ファインチューニング実験
- [ ] 実運用評価
- [ ] **AIコード検証の自動化**

## リスクと対策

### 技術的リスク
1. **記号衝突**: 既存演算子との衝突 → 慎重な記号選定
2. **デバッグ困難**: 圧縮コードの可読性 → ハイブリッド表示
3. **バージョン互換**: 将来の拡張 → ヘッダーによるバージョニング
4. **テスト複雑性**: ANCP特有のバグ → スモークテスト統合で早期発見

### 運用リスク
1. **AIの誤解釈**: 記号の誤認識 → 明示的なヘッダー
2. **人間の混乱**: 2つの記法混在 → 明確な使い分け
3. **ツール対応**: エコシステム分断 → 段階的移行
4. **テスト負荷**: 2倍のテスト実行 → 並列実行とキャッシュ活用

## 成功指標（更新版）
- トークン削減率: 50%以上（目標70%）
- 往復変換成功率: 100%
- パース時間増加: 10%以内
- AIエラー率改善: 30%以上
- **スモークテスト合格率: 100%（Nyash/ANCP両方）**
- **CI実行時間増加: 20%以内**

## 関連技術・参考
- JavaScript Minification (Terser, UglifyJS)
- WebAssembly (.wat ⇔ .wasm)
- Protocol Buffers (スキーマベース圧縮)
- Source Maps (位置情報マッピング)
- **Nyashスモークテスト基盤**（パターン検証方式）

## 専門家レビュー
- Gemini: トークナイザ最適化、記号体系化の重要性を指摘
- Codex: Rust実装設計、テスト戦略の詳細を提供
- Claude: スモークテストとの統合可能性を発見

## 次のアクション
1. tiktoken実測実験の実施
2. 最小プロトタイプの作成
3. ChatGPTさんとの実装協議
4. **既存スモークテストへのANCPモード追加**

---
*このアイデアは2025-08-29にGeminiさん、Codexさんとの深い技術討論を経て、実装可能な設計として成熟し、さらにスモークテストとの統合により品質保証も含めた完全なソリューションとなりました。*