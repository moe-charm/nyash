# 簡単マン流シェル革命 - Everything is Box Shell

作成日: 2025-08-27
Status: 構想段階
Priority: High
Related: ChatGPT5との共同構想

## 🎯 ビジョン

**Windows/Ubuntu差異を感じさせない、Boxベースの最小シェル**

すべてをBoxで統一することで：
- クォート地獄からの解放
- OS差異の完全吸収
- 暗黙展開による事故の根絶
- 思考速度に追いつくUI

## 🏗️ コア設計（全部Box）

### 基本Box群
```nyash
// 環境変数は Map<str,str>（常にUTF-8）
EnvBox: 環境変数管理、OS差異はアダプタで吸収

// 引数は常に配列（トークナイズ済）
ArgvBox: スペース/クォート解釈は一切しない

// プロセス表現
ProcBox: spawn/exec/wait/kill メソッド、I/OはPipeBox/FileBox

// パスは正規化（/区切りで統一）
PathBox: Windows/Unix差はアダプタで解決

// パイプはBoxで明示
PipelineBox: lhs | rhs も等価メソッドで表現可
```

## 💎 使用例（クォート地獄からの解放）

### 従来のシェル（地獄）
```bash
# スペースとクォートの悪夢
git commit -m "fix: handle 'quotes' and \"escapes\" properly"
tar czf archive.tar.gz $(find . -name "*.md" -o -name "*.txt")
```

### 簡単マン流シェル（天国）
```nyash
// クォート不要！配列で書く
Cmd(["git", "commit", "-m", "fix: handle 'quotes' and \"escapes\" properly"])
  .run(cwd, env).wait()

// 変数展開も明示的
let files = Glob("*.{md,txt}").expand(cwd)
Cmd(["tar", "czf", "archive.tar.gz"].concat(files))
  .run(cwd, env).wait()
```

## 🚀 思考速度UI設計

### 反応速度の約束
- **20ms以内**: タイプ開始〜候補提示
- **80ms以内**: 実行プレビュー表示
- **モードレス**: 学習不要・覚えるコマンド最小

### UI構成要素（全部Box）
```nyash
InputLineBox: 1行入力、文脈で自動パース
SuggestBox: リアルタイム3レベル候補（コマンド/アクション/引数）
ArgChipsBox: 引数を"チップ"で視覚化（Tab/矢印で移動）
PreviewBox: 実行前プレビュー（結果サマリ/影響件数/危険度）
HistoryStackBox: 直前操作の即戻し（Ctrl+Z/Y）
```

### 入力体験
```
git → [commit, status, log] → commit → [message:"", add:[]]
→ Preview: "3 files staged, commit message: fix"
→ Enter実行
```

## 🐳 軽量コンテナ統合（Docker級をシンプルに）

### コンテナもBox
```nyash
// イメージからコンテナ作成
let img = ImageBox.from_tar("ubuntu-min.tar")
let fs = FSBox.overlay(img, rw_dir="/tmp/ctr1")
let net = NetBox.bridge("ny0").expose(8080->80)

// コンテナ構成
let ctr = ContainerBox {
  fs, net,
  env: EnvBox.default().set("RUST_LOG", "info"),
  ns: NamespaceBox.default().isolate_pid().isolate_mount(),
  caps: CapsBox.minimal().seccomp_profile("default"),
  cmd: Cmd(["/usr/bin/nginx", "-g", "daemon off;"])
}

// 実行・操作
ctr.start()
let logs = ctr.attach_stdout()
ctr.exec(Cmd(["ls", "-la", "/"]))
ctr.stop()
ctr.commit("myimg:latest")
```

### fini伝播による確定的クリーンアップ
- プロセスkill → アンマウント → veth破棄 → temp削除
- GCオン/オフでも観測等価

## 📺 ハイブリッドシェル（テキスト＋最小GUI）

### 設計原則
1. **真実はテキスト**: 常にテキスト実行可能（CLI互換）
2. **GUIは注釈**: テキスト結果に薄いメタを添えるだけ
3. **レンダはクライアント責務**: サーバは重いUIを送らない

### メッセージ設計（超軽量）
```bash
git status
```
```json
{"ui":"table","cols":["file","status"],"data":[["src/app.rs","M"],["README.md","U"]]}
```

### 3テンプレートで統一
- `table`: ファイル一覧、git status等
- `list`: 検索結果、ファイル列挙等
- `log`: 実行ログ、HTTPレスポンス等

## 🔒 セキュリティ・安全性

### 構造化による根本解決
- **インジェクション不可能**: すべて配列引数
- **暗黙展開禁止**: $VAR、*, ? の自動展開なし
- **AllowList**: 実行可能コマンド限定
- **Timeout/Limit**: リソース制限明示

### Lintルール
- L1: スペース区切り文字列禁止 → 配列で書け
- L2: 環境変数の暗黙展開禁止 → env.get()使用
- L3: 相対パスのみ警告 → cwd明示
- L4: 未捕捉stderr警告

## 📅 実装ロードマップ

### Phase 1: 構造シェル基盤（1週間）
- [ ] CmdBox + EnvBox（配列引数/UTF-8）
- [ ] PipelineBox + PipeBox（パイプ実装）
- [ ] File redirect + GlobBox
- [ ] 基本Lintルール実装

### Phase 2: 思考速度UI（1週間）
- [ ] SuggestBox（20ms補完）
- [ ] ArgChipsBox（引数可視化）
- [ ] PreviewBox（80msプレビュー）
- [ ] HistoryStackBox（Undo/Redo）

### Phase 3: コンテナ統合（2週間）
- [ ] ContainerBox基盤
- [ ] FSBox（overlay/bind）
- [ ] NetBox（bridge/port）
- [ ] NamespaceBox（隔離）

### Phase 4: ハイブリッドGUI（1週間）
- [ ] 3テンプレート実装
- [ ] UiMetaBox設計
- [ ] テキスト↔GUI等価性
- [ ] WebSocket/stdio両対応

## 💡 期待される革命

1. **開発効率10倍**: クォート地獄・OS差異から解放
2. **学習コスト1/10**: Everything is Boxで統一
3. **事故率1/100**: 構造化・明示化で根本安全
4. **思考速度実現**: 20ms反応でストレスフリー

---

*「簡単マン」の究極形態 - すべてが箱になった世界*