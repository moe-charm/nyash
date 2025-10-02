# PHI処理設計ドキュメント（箱理論実装）

## 📖 概要

このドキュメントは、LLVM Python バックエンドにおけるPHI命令処理の設計決定、現状の問題点、改善方針を記載します。

**更新日**: 2025-10-01
**担当**: Claude Code + ChatGPT Pro 協働分析

---

## 🎯 設計原則: 箱理論（Box-First）

### 箱理論の実践
PHI命令処理は「箱理論」に基づいて実装されています：

1. **「箱にする」**: PHI処理を専用モジュール（PhiHandler）に分離
2. **「境界を作る」**: block layer（block_lower.py）とinstruction layer（instruction_lower.py）の責任を明確化
3. **「戻せる」**: 従来の処理フローも維持可能（デバッグ・検証用）
4. **「見える化」**: PHI処理の流れが明確（verbose mode対応）

---

## 🏗️ アーキテクチャ

### 現在の実装（wasm-development）

```
block_lower.py (Block Layer)
  ├─ PhiHandler.collect_phi_instructions()  # PHI命令分離
  ├─ PhiHandler.process_phi_instructions()  # PHI命令処理（ブロック先頭）
  └─ builder.lower_instruction()            # 非PHI命令処理

instruction_lower.py (Instruction Layer)
  ├─ lower_phi()  ← ⚠️ デッドコード（到達不能）
  ├─ lower_binop(), lower_compare(), ...
  └─ InstructionContext ← 🔧 PHI専用、未使用
```

### PHI処理の流れ

```python
# 1. block_lower.py で命令を分離
phi_ops, non_phi_insts = phi_handler.collect_phi_instructions(insts)

# 2. PHI命令を先頭で処理
phi_handler.process_phi_instructions(phi_ops, bb, func)

# 3. 非PHI命令を処理
for inst in body_ops:
    builder.lower_instruction(ib, inst, func)
    # ← PHI命令はここに到達しない（既に分離済み）
```

---

## 🚨 現状の問題点（ChatGPT Pro 分析結果）

### 1. PHI処理の二重経路問題

**問題**:
- `block_lower.py` でPHI命令を処理しているため、`instruction_lower.py:81-100` のPHI処理コードは**到達不能（デッドコード）**

**影響**:
- コード重複によるメンテナンス負荷
- 将来的なバグの温床（二重処理の可能性）

**対策**:
- instruction_lower.pyのPHI処理コードを削除（PhiHandlerに一本化）

---

### 2. Silent Failure（エラー握りつぶし）

**問題**:
```python
# block_lower.py:166-169
except Exception as e:
    trace_debug(f"[llvm-py] PHI processing error: {e}")
    import traceback
    traceback.print_exc()
    # ← ここで処理続行（エラーを握りつぶし）
```

**影響**:
- PHI処理失敗時にビルドが継続してしまう
- 不完全なLLVM IRが生成され、後続でクラッシュ

**対策**:
- フェイルファスト化：PHI処理失敗時は明確に例外を投げてビルド停止
- 環境変数でフォールバックモードを明示的に選択可能にする

---

### 3. vmap二重登録

**問題**:
```python
# phi_handler.py: 両方に登録
self.builder.vmap[dst] = phi
if hasattr(self.builder, '_current_vmap'):
    self.builder._current_vmap[dst] = phi
```

**検討事項**:
- `_current_vmap` はブロックローカルなSSA値追跡用
- `vmap` はグローバルな値マップ
- 両方への登録が本当に必要かを再検証

**対策**:
- ブロック終了時のスナップショット解決で十分か検証
- 不要な場合は片方への登録に統一

---

### 4. InstructionContext の活用不足

**問題**:
```python
# instruction_lower.py:42-44
inst_ctx = InstructionContext.from_owner(owner, builder, builder.block)
# ← 生成後、使われていない（lower_phi に渡していない）
```

**現状**:
- InstructionContext はPHI専用に設計されているが、実際には未使用

**方針**:
- 現状はPhiHandler方式で十分
- 将来的に複雑な命令（binop/compare）に箱化を拡張する際に活用

---

## ✅ 実装済み機能

### 1. ループPHI対応
- ✅ 自己参照を含むループヘッダのPHI命令に対応
- ✅ predecessorスナップショット解決による正確なincoming接続

### 2. 箱理論の実践
- ✅ PhiHandler による責任分離
- ✅ verbose mode によるデバッグ可視化
- ✅ ブロックレイヤでのPHI先頭処理

### 3. SSA値管理
- ✅ ブロックローカルvmap（`_current_vmap`）
- ✅ グローバルvmap（`vmap`）
- ✅ block_end_values スナップショット

---

## 🔧 改善方針（Phase 3.1+）

### 優先度A（緊急）

#### A1. デッドコード削除
- **対象**: `instruction_lower.py:81-100` のPHI処理コード
- **理由**: 到達不能コード、メンテナンス負荷
- **作業**: PhiHandler方式への統一

#### A2. フェイルファスト化
- **対象**: `block_lower.py:166-169` のエラーハンドリング
- **変更前**:
  ```python
  except Exception as e:
      trace_debug(f"[llvm-py] PHI processing error: {e}")
      # 処理続行
  ```
- **変更後**:
  ```python
  except Exception as e:
      if os.environ.get('NYASH_LLVM_PHI_LENIENT') == '1':
          trace_debug(f"[llvm-py] PHI processing error (lenient): {e}")
      else:
          raise RuntimeError(f"PHI processing failed: {e}") from e
  ```

### 優先度B（重要）

#### B1. vmap二重登録の検証
- **検証項目**:
  1. `_current_vmap` のみで十分か？
  2. グローバル `vmap` への登録タイミングは適切か？
  3. スナップショット解決との整合性

#### B2. InstructionContext の適用範囲見極め
- **現状**: PHI専用、未使用
- **検討**: binop/compare等への拡張が本当に必要か？
- **方針**: パフォーマンスと複雑さのバランスを取る

---

## 📊 テスト状況

### 実装済みテスト
- ✅ `test_phi_if.json` - if文PHI合流
- ✅ `test_phi_simple.json` - 基本PHI
- ✅ `test_control_flow_smoke.json` - 制御フロー統合

### 今後追加予定
- [ ] ループPHI専用テスト（自己参照確認）
- [ ] 複雑制御フロー（ネストif、多段ループ）
- [ ] エラーハンドリングテスト（フェイルファスト確認）

---

## 🔍 デバッグ環境変数

### 現在サポート
```bash
# PHI処理詳細ログ
NYASH_PHI_VERBOSE=1

# 厳格モード（synthesized zero検出時にエラー）
NYASH_LLVM_PHI_STRICT=1
```

### 追加予定
```bash
# フォールバックモード（エラー時に処理継続）
NYASH_LLVM_PHI_LENIENT=1
```

---

## 📚 参考リソース

- **PhiHandler実装**: `src/llvm_py/builders/phi_handler.py` (197行)
- **InstructionContext**: `src/llvm_py/builders/instruction_context.py` (98行)
- **箱理論ガイド**: `CLAUDE.md` - "箱理論（Box-First）"セクション
- **Phase 3.1完了報告**: `CURRENT_TASK_WASM.md`

---

## 🎯 まとめ

**現状**: PhiHandler方式で箱理論を実践、基本機能は完全動作
**問題**: 二重経路、silent failure、vmap二重登録
**方針**: デッドコード削除→フェイルファスト化→vmap最適化

ChatGPT Proの分析により、設計の強みと改善点が明確になりました。段階的に改善を進めます。

---

## 追補（2025‑10‑02）— PHI Hardening 方針と実装

目的
- PHI は常に「ブロック先頭にグルーピング」されるLLVM不変条件を、構造で担保する。

変更点（要旨）
- 占位の強化: block_lower が body 降下前に `block_phi_incomings[bid]` の全 dst について `ensure_phi(builder, bid, dst, bb)` を呼び、PHIプレースホルダを先頭に作成。
- 局所合成の既定OFF: resolver によるローカルPHI合成は `NYASH_LLVM_SYNTH_LOCAL_PHI=1` の opt‑in のみ許可。通常は PhiHandler/ensure_phi で先頭に用意。
- 検証強化: `verify_phi_cfg` に加えて `verify_phi_order` を導入し、PHIが非PHIより後に現れないことを検証。`NYASH_LLVM_PHI_VERIFY_STRICT=1` でFail‑Fast。

実装参照
- `src/llvm_py/builders/block_lower.py` — ensure_phi 呼び出しの追加（body 前占位）
- `src/llvm_py/resolver.py` — ローカル合成PHIの既定OFF（環境変数でのみ許可）
- `src/llvm_py/phi_wiring/verify.py` — `verify_phi_order` 追加
- `src/llvm_py/builders/function_lower.py` — PHI順序検証の導入

運用
- bring‑up 時は `NYASH_LLVM_PHI_VERIFY=1`（既定ON）で軽量検証、`NYASH_LLVM_PHI_VERIFY_STRICT=1` でFail‑Fastに。
- どうしても必要な実験時のみ `NYASH_LLVM_SYNTH_LOCAL_PHI=1` を使用（既定はOFF）。
