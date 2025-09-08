# Self Current Task — Backlog (main)

短期
- continue/break 降ろしの実装と単体/統合テストの追加。
- Verifier の支配関係/SSA の確認強化（ループ exit 合流の一意性チェック）。
- LLVM/Cranelift EXE スモーク（単純/ネスト/継続/脱出）。

中期
- VInvoke（vector）戻り型の正道化（トムル記述 or NyRT 期待フラグ）。
- ループ式として値を返す仕様が必要になった場合の設計（現状不要）。

周辺
- selfhosting-dev への取り込みと Ny ツールでの continue/break 使用解禁。
- docs 更新（言語ガイドに continue/break の記法・制約を明記）。

