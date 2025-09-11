# Phase 15 自己ホスティング実装戦略 - MIR→Cranelift→lld

Author: ChatGPT5 + Claude協議
Date: 2025-09-03
Version: 1.0

## 📋 概要

Nyash完全自己ホスティングを実現するための具体的実装戦略。
**「MIR→Craneliftで.o/.objを作る→lldでEXEを組む」**をNyashツールチェーンに内蔵する。

## 🎯 最終形（自己ホスト時の一発ボタン）

```bash
nyash build main.ny \
  --backend=cranelift \
  --target=x86_64-pc-windows-msvc   # or x86_64-unknown-linux-gnu
```

内部処理フロー:
1. **frontend**: AST→MIR13
2. **codegen**: MIR→Cranelift→`.obj/.o`
3. **link**: `lld-link`(Win) / `ld.lld`(Linux)でEXE生成
4. 依存ランタイム`nyashrt`を自動リンク（静的/動的選択）

## 🏗️ 実装の芯（最小で美しいやつ）

### 1. コード生成ライブラリ（C ABIファサード）

```c
// 最小限の美しいインターフェース
ny_mir_to_obj(mir_bin, target_triple) -> obj_bytes
ny_mir_jit_entry(mir_bin) -> exit_code  // 開発用
ny_free_buf(buffer)  // メモリ解放

// エラーハンドリング
// 例外は戻り値＋NyErr（unwind禁止）
```

実装のポイント:
- 返却メモリは`ny_free_buf`で解放
- 例外は戻り値＋NyErrで統一（unwind禁止）
- C ABIで安定した境界を作る

### 2. リンカー・ラッパ（プラットフォーム別）

#### Windows
- 既定: `lld-link`
- 主要フラグ:
```bash
lld-link <objs...> nyashrt.lib /SUBSYSTEM:CONSOLE \
  /OUT:a.exe /ENTRY:nyash_entry \
  /LIBPATH:<sdk/lib> /MACHINE:X64
```
- MSVC互換が要る配布向けに`/fallback:link.exe`オプションも用意可

#### Linux
- 既定: `ld.lld`（開発で`mold`併用可）
```bash
ld.lld -o a.out main.o -L. -lnyashrt -lc -lm -pthread \
       --gc-sections --icf=all
```

#### macOS（将来）
- 日常は`ld64.lld`、配布はXcodeの`ld64` + コード署名（要Entitlements）

### 3. 同梱/検出戦略

**優先順**: 埋め込み`lld` → システム`lld` → 代替（mold/link.exe/ld64）

```bash
nyash toolchain doctor  # 検出＆パス設定
--linker=lld|mold|link.exe|ld64  # 明示上書き
```

### 4. ランタイム同梱

- `nyashrt`を**static（.a/.lib）**と**shared（.so/.dll）**両用意
- 既定は**static**（配布が楽）、`--shared-rt`で動的リンクに切替
- Windowsは**PDB生成**、Linuxは`-g`/`-Wl,--build-id`でデバッグ容易に

## 🔧 エラー整合（ビルド失敗をわかりやすく）

| エラー種別 | 戻り値 | 説明・対処 |
|----------|-------|-----------|
| `ny_mir_to_obj`失敗 | `NYCG_ERR_*` | ターゲット不一致/CLIF生成失敗など |
| リンク失敗 | リンカ標準出力 | ファイル名/未解決シンボルを整形表示 |

診断オプション:
```bash
--emit=clif,asm,obj,link-cmd  # 診断をファイル出力（再現しやすい）
```

## 💾 キャッシュ＆クロスコンパイル

### オブジェクトキャッシュ
`hash(MIR, target, codegen_ver)` → `.obj/.o`を再利用

### クロスコンパイル
```bash
--target=<triple>  # .obj/.oとリンク器/SDKを切替
```
- Win用: `x86_64-pc-windows-msvc`（`lld-link` + MSVCライブラリ）
- Linux: `x86_64-unknown-linux-gnu`（`ld.lld` + glibc）

**Zig toolchain**を併用するとクロス用のCRT/SDKが楽（内部はlld）

## 🎨 使いやすいCLI例

```bash
nyash build main.ny --backend=cranelift --release
nyash build main.ny --emit=obj,asm,clif       # 解析用
nyash run   main.ny --backend=cranelift       # JITで即実行
nyash toolchain doctor                        # lld/SDK検出
```

## ⚡ 地味に効く最適化スイッチ

### リンカ最適化
- `ld.lld`: `--gc-sections --icf=all`（不要コード除去＆同一関数折りたたみ）

### Cranelift最適化
- `opt_level=speed`
- TypedArrayの**bounds-check併合**をLowerで実装

### 実行時最適化
- 起動時CPUIDで**関数ポインタ切替**（AVX2/512の専用小関数）

## ✅ 最初の"動くまで"チェックリスト

- [ ] `ny_mir_to_obj`（C ABI）で`.o/.obj`を返せる
- [ ] `nyash link <obj> --target=<triple>`が`lld`でEXEを作れる
- [ ] Windows/Linuxそれぞれ"Hello, Box!"実行成功
- [ ] `--emit=clif,asm`でダンプが落ちる
- [ ] 失敗時のエラーメッセージが**ファイル名＋未解決シンボル**まで出る
- [ ] `nyash toolchain doctor`でlld/SDK検出

## 📐 実装設計詳細

### LinkerBox設計
```nyash
box LinkerBox {
    init { platform, linker_path, libraries, flags }
    
    link(objects, output_path) {
        local cmd = me.build_link_command(objects, output_path)
        local result = me.execute_linker(cmd)
        
        if result.exit_code != 0 {
            me.format_link_error(result.stderr)
        }
        
        return result
    }
    
    detect_linker() {
        // 優先順: 内蔵lld → システムlld → 代替
        if me.has_embedded_lld() {
            return me.extract_embedded_lld()
        }
        return me.find_system_linker()
    }
}
```

### CraneliftBox統合
```nyash
box CraneliftBox {
    init { target_triple, opt_level }
    
    compile(mir) {
        // MIR13 → Cranelift IR → Object
        local module = me.create_module()
        
        for inst in mir.instructions {
            me.lower_instruction(module, inst)
        }
        
        return module.compile()
    }
}
```

## 🌟 まとめ

- **Yes**: CraneliftでEXEにするには**内部でlldを叩く機能を埋め込む**のが正攻法
- 仕組みは**MIR→Cranelift .o/.obj → lld**
- C ABIファサード経由でcodegenを呼び、リンカは**内蔵ドライバ**で統一
- これで**自己ホスト→即EXE生成**の"気持ちいい体験"が完成！

## 🔗 関連ドキュメント

- [Phase 15メインドキュメント](README.md)
- [C ABI境界設計](../phase-12/c-abi-spec.md)
- [MIR 13命令セット](../../reference/mir/INSTRUCTION_SET.md)
- [Cranelift統合](../phase-10/)