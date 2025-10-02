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
- `NYASH_LLVM_SANITIZE_EMPTY_PHI=1` … 空PHI除去とPHI先頭グルーピング（開発用の保険）。既定ON（ランナー側で OFF 指定がない限り 1 を注入）
- `NYASH_NYRT_SILENT_RESULT=1` … AOT 実行時にランタイム末尾の `Result: <n>` を抑制（スモーク比較をクリーンにする目的、既定OFF）

PHI 検証・合成・作成のガード
- `NYASH_LLVM_PHI_VERIFY=1` … finalize 後に軽量 verify を実行（既定ON）。`=0` で無効化
- `NYASH_LLVM_PHI_VERIFY_STRICT=1` … 問題発見時に即失敗（Fail‑Fast）
- `NYASH_LLVM_PHI_STRICT=1` … PhiHandler は生成のみ（配線は finalize に一元化）
- `NYASH_LLVM_SYNTH_LOCAL_PHI=1` … resolver のローカル合成 PHI を許可（既定OFF/開発用）
- `NYASH_LLVM_PHI_ALLOW_CREATE=1` … finalize 中に PHI を新規作成を許可（既定OFF：wire‑only）

PHI 統一方針（既定）
- PHI は PhiHandler（block_head）で生成する。
- finalize_phis は“配線のみ”。PHI を新規生成しない（`NYASH_LLVM_PHI_ALLOW_CREATE=1` でのみ許可）。
- if-merge/loop のプリパスは既定OFF（必要時のみ開発者が明示ON）。

PHI Hardening（2025‑10‑02）
- Block先頭での占位を強化: block_lower が `block_phi_incomings` の (block,dst) すべてに対して `ensure_phi()` を呼び、PHIを必ずブロック先頭に作成してから本体を降下。
- 局所合成PHIは既定OFF: resolver のローカルPHI合成は `NYASH_LLVM_SYNTH_LOCAL_PHI=1` のときのみ許可（通常は PhiHandler/ensure_phi で先頭に作る）。
- 検証を強化: `verify_phi_cfg` に加えて `verify_phi_order` を導入し、PHIがブロック先頭にグルーピングされていることを検証。`NYASH_LLVM_PHI_VERIFY_STRICT=1` でFail‑Fast。

## 🧪 PHI スモーク（VM vs LLVM 比較）

`tools/smokes/v2/run_phi.sh` は `apps/tests/phi_*.nyash` を自動発見して、
VM と LLVM AOT 実行の `Result: <n>` 行を比較します。

使い方
```bash
# release/profile（省略可）
APP_BIN_DIR=tmp ./tools/smokes/v2/run_phi.sh release

# フィルタ（部分一致）
PHI_FILTER=nested ./tools/smokes/v2/run_phi.sh release
```

挙動
- 事前に llvmlite ハーネスで .o を生成（`NYASH_LLVM_USE_HARNESS=1`）
- リンク→実行→`Result:` 行のみ比較（NYRT は `NYASH_NYRT_SILENT_RESULT=1` 推奨）
- タイムアウトは既定 15 秒（長いケースは `TIMEOUT=30` などで調整）

## 📦 PhiDispatchPoint（計画）

compare/branch/binop の値解決フォールバックを 1 箱に統合する設計を準備しています。
当面は compare/branch から薄く利用を開始し、段階的に移行します。

### PhiRegistry（箱理論の完璧な実践例）

#### 🎯 目的
同一 `(block_id, dst_vid)` に対して PHI SSA を「1つだけ」に統一（**単一起点化**）。

#### 📦 箱理論の4原則実践

1. **「箱にする」** - PHI管理を専用箱（Registry）に完全分離
   - 責務: 登録・検索のみ（配線や検証は別箱）
   - 境界: `phi_wiring/registry.py` に閉じ込め

2. **「境界を作る」** - キーで明確な識別
   - キー: `(int(block_id), int(dst_vid))` のタプル
   - 型安全: 必ず `int()` でキャスト
   - ユニーク保証: 同一キーに対して1つのPHIのみ

3. **「戻せる」** - 既存PHI再利用で冪等性保証
   - `ensure()`: 存在すれば再利用、なければ生成
   - 副作用最小: 重複生成を完全に防止
   - トレーサビリティ: 同一キーは常に同一インスタンス

4. **「見える化」** - 単一起点による追跡容易性
   - デバッグ: PHIの出所が明確
   - 検証: `verify_phi_uniqueness` で重複検出
   - ログ: `phi_<dst>` vs `phi_<dst>.1` の区別が明確

#### 🔧 インターフェース

```python
# 基本操作
PhiRegistry.ensure(builder, block_id, dst_vid, bb)  # 取得or生成
PhiRegistry.get(builder, block_id, dst_vid)         # 取得のみ
PhiRegistry.register(builder, block_id, dst_vid, phi)  # 登録

# 互換性シム
from phi_wiring.registry import ensure_phi
ensure_phi(builder, block_id, dst_vid, bb)  # 従来のインターフェース
```

#### 📋 契約（Contract）

**前提条件**:
- PHI は常にブロック先頭に作成される（`position_at_start`）
- `block_id` と `dst_vid` は正の整数

**事後条件**:
- `builder.vmap[dst]` に登録
- `builder._current_vmap[dst]` に登録（存在する場合）
- `builder.phi_registry[(block_id, dst_vid)]` に登録
- 同一インスタンスが3箇所に存在（一貫性保証）

**不変条件**:
- 同一 `(block_id, dst_vid)` に対してPHIは必ず1つ
- 重複SSA名（`phi_X.1`）は発生しない
- vmap と _current_vmap と registry は常に同期

#### 🔍 検証

```bash
# PHI重複検出（Fail-Fast）
NYASH_LLVM_PHI_VERIFY_STRICT=1 ./target/release/hakorune --backend llvm test.hkr

# 検証内容
verify_phi_uniqueness()   # 重複PHI検出（phi_<dst> と phi_<dst>.1）
verify_phi_order()        # ブロック先頭配置検証
verify_phi_cfg()          # CFG整合性検証
```

#### 🚀 さらなる箱化の可能性（UltraThink考察）

##### 1. **PhiLifecycle Box** - ライフサイクル統一管理
現状は3箇所に分散：
- `PhiRegistry`: 生成・登録
- `PhiHandler`: 収集・初期配線
- `finalize_phis`: 最終配線

**提案**: 統一的なライフサイクル管理箱
```python
class PhiLifecycle:
    """PHI生成から検証までの完全なライフサイクル管理"""

    def create_phase(self):   # 生成フェーズ（Registry）
    def wire_phase(self):     # 配線フェーズ（Handler + finalize）
    def verify_phase(self):   # 検証フェーズ（Verifier）
```

**利点**:
- 状態遷移の明確化: `CREATED` → `PARTIAL_WIRED` → `FULLY_WIRED` → `VERIFIED`
- エラー追跡の容易化: どのフェーズで失敗したか一目瞭然
- テスト容易性: 各フェーズを独立テスト可能

##### 2. **PhiSnapshot Box** - 状態スナップショット
**用途**:
- デバッグ: 任意時点のPHI状態を保存・比較
- テスト: 状態再現によるリグレッション検出
- トレース: PHI配線履歴の完全記録

```python
class PhiSnapshot:
    """PHI状態のイミュータブルスナップショット"""

    def capture(self, registry):  # 現在の状態を保存
    def diff(self, other):        # 2つの状態の差分
    def replay(self):             # 保存した状態を再現
```

##### 3. **BlockVMap統合深化**
`BlockVMap` と `PhiRegistry` は密接に関連。

**提案**: 統一的な「ブロックローカル状態管理箱」
```python
class BlockState:
    """ブロック内の値とPHIの統一管理"""

    def __init__(self, block_id):
        self.vmap = BlockVMap(block_id)
        self.phi_registry = {}
        self.snapshots = []
```

**利点**:
- 値とPHIの一体管理
- ブロック境界での状態遷移が明確
- デバッグ時の状態確認が容易

#### 📊 設計洞察

**単一責任の徹底**:
- PhiRegistry: 登録と検索**のみ**
- 配線: `phi_wiring` モジュール
- 検証: `verify_*` 関数群
→ 責務が明確でデバッグが容易

**失敗の局所化**:
- 各操作を `try/except` で隔離
- 一部の失敗が全体に波及しない
- エラーハンドリングが細粒度

**不変条件の保証**:
- キーの整数化で型安全
- 同期更新でデータ一貫性
- 冪等操作で副作用最小

#### 🎓 学習価値

PhiRegistryは「箱理論」の教科書的実装：
- 小さく（102行）
- 明確な責務
- 完全な境界
- 検証可能
- 拡張容易

**参考実装**: `src/llvm_py/phi_wiring/registry.py`

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
