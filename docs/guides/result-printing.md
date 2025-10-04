# Result Printing Policy (VM/LLVM/WASM)

目的
- 実行結果の表示を“最深部（葉）”に集約し、どの経路でも確実に見えるようにする。
- 取りこぼし（標準出力のバッファや分散した表示責務）を防ぐ。

設計原則
- 出力責務は葉に置く（Non‑Ambiguous）：
  - VM（Rust VM ライン）: VM エンジン（FallbackVmEngine）が `Result: <n>` を出力（quiet 時は抑制）し、flush してから終了コードへマップ。
  - LLVM（ハーネス）: Runner の LLVM モードが既存どおり出力。
  - LLVM（AOT/スタンドアロン）: nyrt 側 main スタブが `ny_main()` の戻り値を `Result: <n>` として出力し、flush の上で exit(n & 0xFF)。
  - WASM: Node ランナー（tools/wasm_runner.js）が `returned: <n>` を出力。

Quiet/抑制（既定）
- `NYASH_JSON_ONLY=1`: 子パイプライン静音（CI/emit-only）。Result 行は出さない。
- （提案）`NYASH_NYRT_SILENT_RESULT=1`: AOT 実行時の Result 行を抑制（ハーネス比較のための将来トグル）。

実装メモ
- VM パス: 標準出力に `Result: <n>`、さらに標準エラーにもミラーしてパイプ時の取りこぼしを防ぐ（flush 必須）。
- LLVM ハーネス: 既存の `📊 Result: ...` 出力を維持（比較ツールは body のみ抽出）。
- AOT スタブ: C/Rust どちらでも可。例（C）：
```c
int64_t ny_main(void);
int main(void) {
  int64_t r = ny_main();
  printf("Result: %lld\n", (long long)r);
  fflush(stdout);
  return (int)(r & 0xFF);
}
```

テスト/ベンチ
- WASM ベンチは Node ランナーの `returned:` 行で比較。
- VM/LLVM は `Result:` 行（または exit code）で比較（ランナー/スクリプトが整形）。
