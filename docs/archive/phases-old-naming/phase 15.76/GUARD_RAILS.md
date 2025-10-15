# 運用ガードレール（破ると壊れる）

## 🚨 絶対守る3ルール

### 1. new→birth一元化
- ❌ `new Constructor()` 禁止
- ✅ `Constructor.birth()` のみ
- 理由: 統一初期化、デバッグ容易

### 2. HAKO_PLUGIN_POLICY=auto
- デフォルト設定変更禁止
- plugin-on/only 切替は明示的に
- 理由: 予期しない動作防止

### 3. NoOperatorGuard
- 演算子オーバーロード保護
- == は MIRで op_eq に降ろす（VM傍受禁止）
- 理由: 決定性保証、デバッグ容易

## 🔥 赤字ルール（絶対破らない）
- 単一パーサ維持（Hakoruneのみ）
- 凍結EXE（stage0）慎重更新（1年/1回程度）
- Rust層基本凍結（緊急時以外触らない）
