# 二重セルフビルドテスト（決定性検証）

## 目的
コンパイラの決定性保証（Rust/Go業界標準）

## 方式
```
Stage 0 (凍結EXE) → Hako₁
Hako₁ → Hako₂
Hako₂ → Hako₃

検証: Hako₂ == Hako₃ (バイト同一)
```

## 実装
```bash
# tools/ci/double-selfhost-test.sh
STAGE0=./bin/hako-frozen-v1.exe

$STAGE0 apps/selfhost/full_compiler.hako -o hako-stage1.exe
./hako-stage1.exe apps/selfhost/full_compiler.hako -o hako-stage2.exe
./hako-stage2.exe apps/selfhost/full_compiler.hako -o hako-stage3.exe

cmp -s hako-stage2.exe hako-stage3.exe || exit 1
echo "✅ Deterministic!"
```

## CI統合
- GitHub Actions: 毎push/PR
- quick-selfhost プロファイル追加検討
- 失敗時 → 非決定性コンパイル警報

## タイミング
Phase 15.76完了後 → 凍結EXE生成時に設定
