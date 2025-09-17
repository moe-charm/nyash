# TODO (Phase 10.7 Workbench)

短期（C1〜C3に向けた小粒タスク）
- [ ] C1: Parser plugin 雛形スケルトンを作る（pyo3, parse(code)->AstBox/to_json）
- [ ] C1: Telemetry最小（node種別カウント, 未対応ノード列挙）
- [ ] C2: CorePy IR最小スキーマ（JSON）を commit（with/async系は予約）
- [ ] C2: IR→Nyash ASTの最小変換（def/if/for/while/return/算術/比較/呼出し）
- [ ] C3: CLI隠しフラグ prototyping（--pyc/--pyc-native）
- [ ] Docs: PLANとimplementationの差分同期（週次）

メモ
- All-or-Nothing原則：未対応は即Err（自動フォールバックなし）
- 生成Nyashは現行AOT導線で配布可能（Strict）
