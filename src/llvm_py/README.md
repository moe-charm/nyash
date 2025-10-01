# LLVM Python Backend (Experimental)

## 📝 概要
Rust/inkwellの複雑性を回避し、llvmliteを使ってシンプルに実装する実験的バックエンド。
ChatGPTが設計した`docs/development/design/legacy/LLVM_LAYER_OVERVIEW.md`の設計原則に従う。

## 🎯 目的
1. **検証ハーネス** - PHI/SSA構造の高速検証
2. **プロトタイプ** - 新機能の迅速な試作
3. **教育的価値** - シンプルで理解しやすい実装
4. **バックアップ** - Rustが詰まった時の代替案

## 📂 構造
```
llvm_py/
├── README.md                  # このファイル
├── llvm_builder.py            # メインのLLVM IR生成（パスのオーケストレーション）
├── mir_reader.py              # MIR(JSON) ローダ
├── resolver.py                # 値解決（SSA/PHIの局所化とキャッシュ）
├── utils/
│   └── values.py              # 同一ブロック優先の解決などの共通ポリシー
├── cfg/
│   └── utils.py               # CFG ビルド（pred/succ）
├── prepass/
│   ├── loops.py               # ループ検出（while 形）
│   └── if_merge.py            # if-merge（ret-merge）前処理（PHI前宣言プラン）
├── instructions/
│   ├── controlflow/
│   │   ├── branch.py          # 条件分岐
│   │   ├── jump.py            # 無条件ジャンプ
│   │   └── while_.py          # 通常の while 降下（LoopForm 失敗時のフォールバック）
│   ├── binop.py               # 2項演算
│   ├── compare.py             # 比較演算（i1生成）
│   ├── const.py               # 定数
│   ├── copy.py                # Copy（MIR13 PHI-off の合流表現）
│   ├── call.py                # Ny 関数呼び出し
│   ├── boxcall.py             # Box メソッド呼び出し
│   ├── externcall.py          # 外部呼び出し
│   ├── newbox.py              # Box 生成
│   ├── ret.py                 # return 降下（if-merge の前宣言PHIを優先）
│   ├── typeop.py              # 型変換
│   ├── safepoint.py           # safepoint
│   └── barrier.py             # メモリバリア
└── test_simple.py             # 基本テスト
```

## 🚀 使い方
```bash
# MIR JSONからオブジェクトファイル生成
python src/llvm_py/llvm_builder.py input.mir.json -o output.o

# 環境変数で切り替え（将来）
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash program.nyash
```

## 🔧 開発用フラグ（最小）
- `NYASH_LLVM_USE_HARNESS=1` … Rust 実行から llvmlite ハーネスへ委譲
- `NYASH_LLVM_TRACE_PHI=1` … PHI 配線と end-of-block 解決の詳細トレース
- `NYASH_CLI_VERBOSE=1` … 降下やスナップショットの詳細ログ
- （互換）`NYASH_MIR_NO_PHI=1` / `NYASH_VERIFY_ALLOW_NO_PHI=1` … レガシー検証用のみ

PHI 統一方針（既定）
- PHI は PhiHandler（block_head）で生成する。
- finalize_phis は“配線のみ”。PHI を新規生成しない。
- if-merge/loop のプリパスは既定OFF（必要時のみ開発者が明示ON）。

関数境界の不変（関数ごとに初期化される状態）
- `builder.vmap` / `builder.bb_map` は毎関数クリア
- `builder.block_phi_incomings` は毎関数リセット（前関数のメタデータを持ち越さない）
- `builder.phi_wired` は毎関数 `{}` に初期化（重複 incoming 防止セットのリーク防止）

レガシー finalize 経路の扱い
- finalize_phis は配線専用。`ensure_phi` を内部から呼ばない（既定）
- 互換が必要な場合のみ `NYASH_LLVM_PHI_ALLOW_CREATE=1` で「配線時にPHI作成」を許可（既定OFF）

Strict モード（段階導入）
- `NYASH_LLVM_PHI_STRICT=1`
  - PhiHandler は PHI を「生成のみ」とし、incoming の追加を行わない。
  - incoming の配線は finalize_phis に一元化。
  - 目的: 二重配線/二重生成の温床を解消し、責務を明確化するための段階的スイッチ。

## 📋 設計原則（LLVM_LAYER_OVERVIEWに準拠）
1. Resolver-only reads（原則）: 直接の cross-block vmap 参照は避け、resolver 経由で取得
2. Localize at block start: ブロック先頭で PHI を作る（if-merge は prepass で前宣言）
3. Sealed SSA: ブロック末 snapshot を用いた finalize_phis 配線
4. Builder cursor discipline: 生成位置の厳格化（terminator 後に emit しない）

## 🎨 実装状況
- [ ] 基本構造（MIR読み込み）
- [x] ControlFlow 分離（branch/jump/while_regular）
- [x] CFG/Prepass 分離（cfg/utils.py, prepass/loops.py, prepass/if_merge.py）
- [x] if-merge/loop のプリパス: 既定OFF。PhiHandler 主導で安定化。
- [ ] 追加命令/Stage-3 の持続的整備

## ✅ テスト・検証
- パリティ（llvmlite vs PyVM。既定は終了コードのみ比較）
  - `./tools/pyvm_vs_llvmlite.sh apps/tests/ternary_nested.nyash`
  - 代表例（プリパス不要）:
    - `./tools/pyvm_vs_llvmlite.sh apps/tests/ternary_nested.nyash`
    - `./tools/pyvm_vs_llvmlite.sh apps/tests/loop_if_phi.nyash`
- 厳密比較（標準出力+終了コード）
  - `CMP_STRICT=1 ./tools/pyvm_vs_llvmlite.sh <file.nyash>`
- まとまったスモーク（PHI-off 既定）
  - `tools/smokes/curated_llvm.sh`
  - PHI-on 検証（実験的）: `tools/smokes/curated_llvm.sh --phi-on`

## 📊 予想行数
- 全体: 800-1000行
- コア実装: 300-400行

「簡単最高」の精神を体現！
