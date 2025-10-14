# 内蔵 vs プラグイン設計方針（Phase 15.76）

## 🎯 設計原則

凍結EXE（hako-frozen-v1.exe）に「何を内蔵し、何をプラグイン化するか」は**ブートストラップの安定性とTCB最小化のトレードオフ**。

### 基本方針
- **内蔵**: 日常開発の土台、小さく安全、外部依存なし
- **外付け**: ツールチェーン・重量物・セキュリティ影響大

---

## ✅ 凍結EXEに内蔵すべき最小セット（静的同梱）

### 📌 Core（必須・小さく安全）
- **String/Integer/Bool**: 核となる箱
- **Array/Map**: 言語データ構造の土台
- **Console/Time**: 診断と時刻（`env.console.log`, `nyrt.time.now_ms`程度）

### 📌 JSON（最小）
- **MIR JSONブリッジ**: parse/serialize最小機能
- **設定読み込み**: hako.toml等の設定解析

### 📌 File（最小・読み込み中心）
- **read_text/exists**: 読み込み中心の最小I/O
- **write**: プロファイルでON（安全重視）

### 📌 extern_c ランタイム導線（実装済み）
- **ffi.dynamic 経路**: deny-by-default + allowlist設定

### 🎯 理由
- 日常開発で必要な土台を外部依存なしで安定稼働
- 凍結EXEの再配布を容易にする（再現性・TCB最小）

---

## 🔌 外付け（プラグイン）に回すべきもの

### 🚀 AOT/ツールチェーン（C ABI出力） ⭐最重要判断

**形態**: バックエンドプラグイン（cdylib）として提供

**理由**:
- VM/言語仕様と分離、差し替え容易
- セキュリティ・配布単位が明確
- CIでも扱いやすい

**実装例**:
```hakorune
// libllvm_backend.so経由でC ABI出力
static box Compiler {
    compile_to_object(mir: StringBox, out: StringBox) -> IntegerBox {
        local result = extern_c "llvm_compile_mir_to_object" (
            mir.to_cstring(),
            out.to_cstring()
        )
        return result
    }

    // 将来拡張
    link_objects(objs: ArrayBox, out: StringBox) -> IntegerBox {
        return extern_c "llvm_link_objects"(objs, out)
    }

    compile_to_ll(mir: StringBox, out: StringBox) -> IntegerBox {
        return extern_c "llvm_compile_mir_to_ll"(mir, out)
    }
}
```

**外部ファイル**:
- `plugins/libllvm_backend.so` (Rust cdylib)
- `extern "C" { fn llvm_compile_mir_to_object(...) -> i64; }`
- allowlist: `ffi.dynamic.llvm_compile_mir_to_object = "allow"`

### 🌐 ネットワーク/HTTP
- セキュリティ影響が広い
- プロファイルでON（ENV/TOML）

### 📂 拡張FS（書込み/監視/権限）
- 安全重視（読み込みは内蔵、書き込みは外付け）

### 🎨 重量物（圧縮/暗号/画像/正規表現）
- 必要時のみ導入（サイズ/依存削減）

### ⚙️ OS/Process拡張（spawn/env/pty等）
- extern_c経由で十分
- 許可はENV/TOMLで局所的に

---

## 🔮 未作成だが内蔵を検討する候補（将来）

### 🛤️ Path/URI（軽量ユーティリティ）
- 文字列操作を補助（パス結合・正規化）程度の純関数
- 小さく安全であれば内蔵候補

### 📍 Minimal JSON Pointer/Path（読み取り専用）
- コンパイラ周辺（設定/メタ）にあると便利
- 重いJSON機能は別プラグインで

---

## 📋 プラグイン優先度リスト

### 🔥 最優先（Phase 15.76 Week 1-2）
1. **libllvm_backend** - C ABI出力（.o生成）
2. **extern_c allowlist** - 許可機構完成

### ⚡ 高優先（Week 3-4）
3. **Network/HTTP** - 外付けプラグイン化
4. **拡張FS（書き込み）** - 外付けプラグイン化

### 📦 中優先（Week 5-8）
5. **Path/URI** - 軽量ユーティリティ内蔵
6. **JSON拡張** - Pointer/Path読み取り専用

### 🎁 低優先（将来）
7. 圧縮/暗号/画像/正規表現
8. OS/Process拡張（spawn等）

---

## 🎯 凍結EXE最終構成（目標）

```
hako-frozen-v1.exe (静的同梱)
├── Core Boxes (String/Int/Bool/Array/Map)
├── Console/Time（最小）
├── JSON（最小）
├── File（読み込み専用）
└── extern_c runtime（allowlist機構）

plugins/ (動的ロード)
├── libllvm_backend.so ⭐C ABI出力
├── libnetwork.so（HTTP/Socket）
├── libfs_write.so（書き込み専用）
└── lib*.so（将来拡張）
```

### 🔒 セキュリティ設計
- **deny-by-default**: すべてのextern_cはデフォルト拒否
- **allowlist**: ENV/TOMLで明示的に許可
- **監査可能**: 許可リストは1ファイルで管理

---

## 💡 重要な洞察

### ChatGPTの指摘（2025-10-14）
> 「C ABI出力（.o 生成・リンク補助）はプラグイン（バックエンドプラグイン）として切り出すのが最適だよ。」

### 理由
1. **VM/言語仕様から独立** - Hakorune構文変更の影響なし
2. **入替え容易** - LLVM 18→19等のアップグレード簡単
3. **セキュリティ境界明確** - コンパイラとツールチェーンの責任分離
4. **CIでも扱いやすい** - プラグインのみ差し替えでテスト可能

---

## 📊 比較: 内蔵 vs 外付け

| 機能 | 内蔵 | 外付け | 理由 |
|-----|------|--------|------|
| String/Int/Bool | ✅ | ❌ | 核となる箱 |
| Array/Map | ✅ | ❌ | 言語データ構造 |
| Console/Time | ✅ | ❌ | 診断・時刻（最小） |
| JSON（最小） | ✅ | ❌ | MIR/設定 |
| File（読み込み） | ✅ | ❌ | 日常開発 |
| File（書き込み） | ❌ | ✅ | 安全重視 |
| C ABI出力 | ❌ | ✅ | ツールチェーン分離 ⭐ |
| Network/HTTP | ❌ | ✅ | セキュリティ影響大 |
| 圧縮/暗号 | ❌ | ✅ | 重量物 |

---

## 🚀 次のアクション

### Week 1-2（Phase 15.76前半）
1. libllvm_backend プラグイン化
2. extern_c allowlist機構完成
3. 凍結EXE最小構成確定

### Week 3-4（Phase 15.76後半）
4. Network/HTTP外付け化
5. 拡張FS（書き込み）外付け化
6. セキュリティ監査

---

**作成日**: 2025-10-14
**関連**: Phase 15.76, extern_c戦略, 凍結EXE設計
