# Phase 5.2: Lowering for static box Main (BoxDeclaration → main body)

Summary:
- static box Main { main() { ... } } を MirBuilder で受け、main() の body を Program として lowering する経路を実装します。
- 目的は `--dump-mir` が static Main 形式のサンプルでも通り、VM 実行にも到達すること。

Scope:
- AST: BoxDeclaration(is_static=true, name=Main) を検出 → 同名 main() を探して Program 化
- Lowering: 発見した body を既存の Program lowering に渡す（関数単位でOK）
- Tests: local_tests/mir_loop_no_local.nyash（static Main）で dump/VM が通る

Tasks:
- [ ] MirBuilder: static Main → Program lowering 経路
- [ ] MirPrinter/Verifier: 必要なら修正
- [ ] サンプル/スナップショットの点検

Acceptance Criteria:
- `nyash --dump-mir ./local_tests/mir_loop_no_local.nyash` が成功
- `nyash --backend vm ./local_tests/mir_loop_no_local.nyash` が成功

References:
- #33, #35
- docs/guides/how-to-build-native/copilot_issues.txt

