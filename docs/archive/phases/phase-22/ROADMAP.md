# Phase 22 実装ロードマップ

## 前提条件
- [ ] Phase 15 LLVM Rust実装の完成（ChatGPT5作業中）
- [ ] LLVMバックエンドでの基本的なEXE生成確認
- [ ] MIR 13命令セットの安定

## Phase 0: MVP実装（2週間）

### Week 1: C++グルー層
- [ ] 最小C++ラッパー作成（10関数以内）
  ```cpp
  llvm_init()
  llvm_context_create/free()
  llvm_module_from_ir()
  llvm_verify_module()
  llvm_write_object()
  llvm_get_error()
  ```
- [ ] ビルドシステム整備（CMake/Makefile）
- [ ] 基本的なエラーハンドリング

### Week 2: Nyash実装とテスト
- [ ] LLVMTextGeneratorBox実装
  - MIR → LLVM IR テキスト変換
  - 最小限：main関数、return文のみ
- [ ] エンドツーエンドテスト
  ```bash
  echo 'print(42)' > test.nyash
  ./nyash phase22-compiler.nyash test.nyash
  ```
- [ ] Rust版との出力比較

## Phase 1: 基本機能実装（1ヶ月）

### Week 3-4: MIR命令カバレッジ
- [ ] 算術演算（BinOp, UnaryOp）
- [ ] 制御フロー（Branch, Jump）
- [ ] 関数呼び出し（Call）
- [ ] Box操作（BoxCall基本）

### Week 5-6: バッチBuilder化
- [ ] バイナリエンコーディング設計
- [ ] `llvm_build_batch()` API実装
- [ ] Nyash側エンコーダー実装
- [ ] パフォーマンス測定

## Phase 2: 完全移行（1ヶ月）

### Week 7-8: 高度な機能
- [ ] Phi命令サポート
- [ ] ExternCall完全実装
- [ ] 文字列・配列操作
- [ ] プラグインサポート

### Week 9-10: 最適化と検証
- [ ] 全テストスイート通過
- [ ] パフォーマンスチューニング
- [ ] メモリ使用量最適化
- [ ] ドキュメント整備

## 成功指標

### 必須要件
- [ ] `dep_tree_min_string.nyash` のコンパイル成功
- [ ] 基本的なプラグインテスト通過
- [ ] Rust版と同一のオブジェクトファイル生成

### パフォーマンス目標
- [ ] コンパイル時間: Rust版の2倍以内
- [ ] メモリ使用量: 100MB以内
- [ ] コード行数: 200行以内

### 品質目標
- [ ] エラーメッセージの明確性
- [ ] デバッグ情報の充実
- [ ] 拡張性の確保

## リスクと対策

### 技術的リスク
1. **FFI境界のオーバーヘッド**
   - 対策: バッチ化で呼び出し回数削減

2. **LLVM APIの複雑性**
   - 対策: テキストIRから段階的に移行

3. **デバッグの困難さ**
   - 対策: 充実したロギングとIRダンプ

### スケジュールリスク
- Phase 15完了の遅延 → 並行して設計・プロトタイプ作成

## 長期ビジョン

### Phase 22.5: 自己コンパイル
```nyash
// NyashコンパイラでNyashコンパイラをコンパイル！
local compiler = new NyashCompiler()
compiler.compile("phase22-compiler.nyash", "nyash-compiler.exe")
```

### Phase 23: 完全セルフホスティング
- Rust依存の完全排除
- NyashだけでNyash開発環境構築
- 究極の「Everything is Box」実現

---

> 「難しいけど、夢があるにゃ！」