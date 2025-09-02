# Phase 12 Task Board (v2 - セルフホスティング対応)

目的: C ABI を壊さず、TypeBox + 統一ディスパッチで Nyash ABI を段階導入。MIR→VM→JIT を「綺麗な箱」で統一。**最終的にRust依存を排除し、セルフホスティングを実現。**

## Tier-0（直近・安全に積める）
- [x] MapBoxの実用拡張（stringキー/便利API）
- [x] `keys()/values()` ランタイムシム（現状は改行区切りString返却）
- [ ] TypeBoxレジストリ（雛形）
  - Box名/FQN、type_id、メソッド表、returns_result を登録
  - 既存 `nyash.toml` → TypeBoxInfo への変換層
- [ ] 統一ディスパッチ層（VM）
  - Nyash ABI vtable優先 → 無ければ C ABI（TLV）へフォールバック
  - 所有権・セーフポイントのガード（MAY_BLOCKのみ初期対応）
  - [x] プラグインテスター更新（v2ローダに対応）: `src/bin/test_plugin_loader_v2.rs`

## Tier-1（実証）
- [ ] Nyash ABI vtable の最小サンプル（1プラグイン・1メソッド）
  - 例: MapBox.getS(name) を Nyash ABI で直接返却
  - 単体テスト（VM/JIT）
- [ ] JIT側：統一ディスパッチthunkを呼ぶ経路を追加（フォールバックでも可）
- [ ] 互換テスト: C ABI と Nyash ABI が同一結果になる差分テスト

## Tier-2（強化）
- [ ] NyashValueインライン（i64/bool）の高速化
- [ ] 例外/エラーの完全変換（panic→nyrt_err）
- [ ] 所有権契約の遵守（TRANSFER/BORROW/CLONE）
- [ ] `keys()/values()` の正式実装（ArrayBox返却）
  - 選択肢A: ランタイムで ArrayBox を構築
  - 選択肢B: Mapプラグインに KeysArrayBox を同梱（要設定追加）

## Tier-3（セルフホスティング）🔥新規
- [ ] Nyash ABI C実装の開始
  - [ ] nyash_abi_provider.h定義（16バイトアライメント）
  - [ ] C Shim実装（Rust FFI経由）
  - [ ] 基本型実装（Tagged Pointers対応）
  - [ ] アトミック参照カウント実装
  - [ ] 弱参照による循環参照対策
- [ ] セレクターキャッシング実装
  - [ ] lookup_selector API
  - [ ] JIT統合（vtable_slot直接呼び出し）
- [ ] 適合性テストスイート構築
  - [ ] Rust/C実装の差分テスト
  - [ ] パフォーマンス測定（1.5x以内）

## ドキュメント/管理
- [ ] UNIFIED-ABI-DESIGN.md の「最小導入プロファイル」明記
- [ ] VM/JIT実装メモ（統一ディスパッチの呼出し順）
- [ ] リファクタリング計画（>1000行ファイルの分割方針）

## 既知のやり残し（Phase 12 関連）
- TypeBoxレジストリ/統一ディスパッチのコード未導入
- Nyash ABI vtableの実装サンプル未着手
- 既存プラグインの対応（TypeBox vtable移行 or 互換レイヤ継続）
- GCセーフポイントのMAY_BLOCK以外の一般化
- keys()/values() の正式ArrayBox返却（現状はシム）
- AOT(LLVM)のbuild失敗（nyrt借用修正、後回し方針）
- Nyash ABI C実装（セルフホスティングの要）🔥新規

## Doneの定義（Phase 12 - 更新版）
1) TypeBoxレジストリと統一ディスパッチがVMに入り、C ABI互換で全プラグインが動作
2) 1プラグインでNyash ABIの成功パスが通る（VM/JIT）
3) keys()/values() が ArrayBox 返却で安定
4) 基本の所有権・セーフポイントルールが守られる
5) **Nyash ABI C実装の基礎が動作し、セルフホスティングへの道筋が明確**🔥新規
