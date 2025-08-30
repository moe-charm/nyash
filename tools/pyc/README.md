# Nyash Python Compiler (Phase 10.7 Workbench)

目的: Parser(プラグイン) → Nyash側コンパイラ → Nyashソース → 既存AOT までの最短ルートを、Nyashだけで段階実装する作業場。

## 構成
- `pyc.nyash` — エントリ（最小パイプライン実行）
- `PyCompiler.nyash` — Nyash側コンパイラ本体（C2で拡張）
- `PyIR.nyash` — IR生成/整形のヘルパ（最小）

## 使い方（最小）
```bash
# 1) NYASH_PY_CODE に Python コードを入れる（Parserプラグインが拾う）
NYASH_PY_CODE=$'def main():\n    return 0' \
  ./target/release/nyash --backend vm tools/pyc/pyc.nyash
```

出力
- Parser JSON（dump/counts/unsupported）
- 生成された Nyash ソース（現状は最小: return 0）

## 次の拡張
- Parser JSON → IR(JSON) への変換（def/return最小）
- IR → Nyash 生成（If/Return/Assign へ拡張）
- All-or-Nothing 運用（unsupported_nodes を見て Strict に弾くスイッチ）
