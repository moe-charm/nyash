# Phase 20.7: Collections in Hakorune

**期間**: 8週間 (2026-05-25 → 2026-07-19)
**前提Phase**: Phase 20.6 (VM Core Complete + Dispatch Unification)
**完了Phase**: Phase D (Collections - MapBox/ArrayBox in Pure Hakorune)

---

## 🎯 概要

Phase 20.7では、HakoruneコアのコレクションライブラリであるMapBoxとArrayBoxを**Pure Hakorune**で実装します。これにより、"Everything is Box"の理念が完全実現され、すべてのコレクション操作がHakorune自身で完結します。

**重要原則**:
- **決定的動作 (Deterministic Behavior)**: キー順序が予測可能（Symbol < Int < String）
- **Fail-Fast**: 不明なメソッド呼び出しは即座にエラー
- **ValueBox/DataBox境界**: ValueBoxは一時的（パイプライン境界のみ）、DataBoxは永続的

---

## 📊 実装範囲

### Week 1-4: MapBox実装

MapBoxは、キー・バリューペアを格納するハッシュマップです。

**実装メソッド**:
- `set(key, value)`: キー・バリューペアを設定
- `get(key)`: キーから値を取得
- `has(key)`: キーの存在確認
- `remove(key)`: キーを削除
- `size()`: 要素数を返す
- `keys()`: すべてのキーを返す（決定的順序）
- `values()`: すべての値を返す（決定的順序）

**技術要件**:
1. **ハッシュマップ実装**: チェイン法またはオープンアドレス法
2. **決定的ハッシュ/等価性**: 同じキーは常に同じバケットに配置
3. **キー正規化**: Symbol/Int/String型を統一的に扱う
4. **Provenance Tracking**: plugin_id, versionを記録

### Week 5-8: ArrayBox実装

ArrayBoxは、動的配列（リスト）です。

**実装メソッド**:
- `push(value)`: 末尾に要素を追加
- `pop()`: 末尾から要素を削除して返す
- `get(index)`: インデックスで要素を取得
- `set(index, value)`: インデックスに値を設定
- `size()`: 要素数を返す
- `slice(start, end)`: 部分配列を返す
- `concat(other)`: 別の配列と結合

**技術要件**:
1. **動的配列実装**: 容量拡張戦略（2倍拡張など）
2. **境界チェック**: インデックス範囲外は即座にエラー
3. **参照カウント**: GC準備（Phase 20.8）のための追跡
4. **Fail-Fast**: 不正な操作は即座にpanicではなくエラー返却

---

## 🔑 キー比較順序（決定的動作の核心）

MapBoxのキー順序は以下のルールで決定されます:

```
Symbol < Int < String
```

**例**:
```hakorune
local map = new MapBox()
map.set(:foo, 1)      // Symbol
map.set(42, 2)        // Int
map.set("bar", 3)     // String

// keys() の返り値は必ず [:foo, 42, "bar"] の順序
local keys = map.keys()
// Output: [:foo, 42, "bar"]
```

**実装詳細**:
- 同じ型内では、自然順序（Symbol: 辞書順、Int: 数値順、String: 辞書順）
- 異なる型間では、上記の型優先順位に従う
- ハッシュ衝突時も、この順序で解決

---

## 🏗️ ValueBox/DataBox境界

**設計原則** (Phase 20.5で確立):
- **ValueBox**: 一時的な値（パイプライン境界のみ）
- **DataBox**: 永続的な値（長期保存）
- **Fail-Fast**: Entry/ExitポイントでValueBoxをUnpack

**Collections適用**:
```hakorune
// ❌ BAD: ValueBoxをMapに直接格納
map.set(key, ValueBox.wrap(data))

// ✅ GOOD: DataBoxに変換してから格納
local databox = ValueBox.unpack(data)
map.set(key, databox)
```

**実装要件**:
1. MapBox/ArrayBoxは**DataBoxのみ**を受け入れる
2. ValueBoxが渡された場合、即座にエラー
3. Pipeline境界でUnpackを強制

---

## 🧪 成功基準

### 必須条件 (✅ すべて達成必須)

1. **MapBox/ArrayBox完全実装**:
   - すべてのメソッドがPure Hakoruneで動作
   - Rustコードへの依存なし（HostBridge以外）

2. **Golden Tests通過**:
   - Rust-Collections vs Hako-Collections パリティ
   - 100+テストケースで完全一致
   - 決定的動作の検証（同じ入力→同じ出力）

3. **決定的動作**:
   - キー順序が予測可能
   - イテレーション順序が安定
   - Provenance Tracking動作

4. **パフォーマンス**:
   - Hako-Collections ≥ 70% of Rust-Collections速度
   - 基本操作（set/get/push/pop）はO(1)平均時間
   - 大規模データ（10,000要素）でも実用的速度

### 除外項目 (⚠️ Phase 20.7では**扱わない**)

- **GC (Garbage Collection)**: Phase 20.8で実装
- **最適化**: 基本実装のみ（プロファイリングは後回し）
- **並行処理**: シングルスレッドのみ
- **永続化**: メモリ内のみ（ディスク保存なし）

---

## 📚 関連ドキュメント

- **Phase 20.6**: [VM Core Complete + Dispatch Unification](../phase-20.6/)
- **Phase 20.8**: [GC v0 + Rust Deprecation](../phase-20.8/)
- **Pure Hakorune Roadmap**: [PURE_HAKORUNE_ROADMAP.md](../phase-20.5/PURE_HAKORUNE_ROADMAP.md)
- **Box-First原則**: [CLAUDE.md](../../../../CLAUDE.md#🧱-先頭原則-箱理論box-first)
- **Golden Testing Strategy**: [PURE_HAKORUNE_ROADMAP.md#🧪-golden-testing-strategy](../phase-20.5/PURE_HAKORUNE_ROADMAP.md#-golden-testing-strategy)

---

## 📝 ドキュメント構造

- **[README.md](README.md)** (このファイル): Phase 20.7概要（日本語）
- **[PLAN.md](PLAN.md)**: 詳細実装計画（英語）
- **[CHECKLIST.md](CHECKLIST.md)**: 週次チェックリスト
- **[INDEX.md](INDEX.md)**: ドキュメント索引

---

**Status**: PLANNED
**Dependencies**: Phase 20.6 (VM Core + Dispatch)
**Delivers**: Phase D (Collections in Pure Hakorune)
**Timeline**: 8 weeks (2026-05-25 → 2026-07-19)
