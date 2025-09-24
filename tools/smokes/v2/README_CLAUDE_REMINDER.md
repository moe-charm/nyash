# 🚨 Claude用リマインダー：ここが正しい場所！

## スモークテストは必ずここ（v2構造）に作る！

### ❌ やってはいけないこと
```bash
# 旧場所に作らない！
tools/new_smoke.sh              # ❌ ダメ
tools/test_something_smoke.sh   # ❌ ダメ
```

### ✅ 正しい作成場所
```bash
# プロファイル別に配置
tools/smokes/v2/profiles/quick/feature_name/test.sh      # 1-2分テスト
tools/smokes/v2/profiles/integration/feature_name/test.sh # 5-10分テスト
tools/smokes/v2/profiles/full/feature_name/test.sh       # 完全テスト
```

### 📁 現在の構造
```
v2/
├── profiles/
│   ├── quick/
│   │   ├── using/        # using systemテスト
│   │   │   ├── named_packages.sh
│   │   │   └── minimal_test.nyash
│   │   ├── boxes/        # Box関連テスト
│   │   └── core/         # コア機能テスト
│   ├── integration/
│   └── full/
├── configs/              # テスト設定
│   └── using_tests.conf
└── run.sh               # 統一エントリポイント
```

### 🎯 新しいテスト追加時の手順
1. まず適切なprofile/ディレクトリを選ぶ（quick/integration/full）
2. 機能名のサブディレクトリを作る
3. テストスクリプトまたは.nyashファイルを配置
4. configs/に設定ファイルを追加（オプション）

### 📝 例：新機能「foo」のテスト追加
```bash
# Step 1: ディレクトリ作成
mkdir -p tools/smokes/v2/profiles/quick/foo/

# Step 2: テスト作成
cat > tools/smokes/v2/profiles/quick/foo/basic.sh << 'EOF'
#!/usr/bin/env bash
# Foo feature smoke test
echo "Testing foo feature..."
EOF

# Step 3: 実行権限
chmod +x tools/smokes/v2/profiles/quick/foo/basic.sh

# Step 4: 実行
./tools/smokes/v2/run.sh --profile quick --filter "foo:*"
```

---
**覚え方**：「スモークはv2！プロファイル別！」🚀