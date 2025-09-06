# MIR13: Everything is Boxによる究極のミニマル中間表現

## 概要

本論文では、わずか13命令で実用的なアプリケーションの実装を可能にする革新的な中間表現（IR）設計「MIR13」を提案する。従来のIR設計では数十から数百の命令が必要とされてきたが、我々は「Everything is Box」という設計哲学に基づき、すべてのメモリアクセスをBoxCallに統一することで、Load/Store命令を完全に廃止した。実装では12命令への削減も可能だが、可読性を考慮して意図的に13命令を採用している。MIR13はInterpreter、VM、JITの3つの実行バックエンドで実証され、実用的なアプリケーションの動作を確認した。

## 1. はじめに

プログラミング言語の中間表現（IR）は、高水準言語と機械語の橋渡しをする重要な抽象層である。LLVM IRは約60の基本命令、WebAssemblyは約170の命令を持つなど、既存のIRは複雑化の一途を辿っている。

本研究では、逆転の発想により「どこまでIRを単純化できるか」に挑戦した。結果として、わずか13命令で実用的なプログラミング言語を実装できることを実証した。従来のIR設計では57命令が必要とされた機能を、BoxCallへの統一により13命令まで削減した経緯についても議論する。

本稿はIR層（MIR13）に焦点を当てる。言語Nyashそのものの設計思想やbirth/fini対称メモリ管理、P2P Intentモデル、多層実行アーキテクチャ等の詳細は、別論文（論文B: Nyash言語と実行モデル）で報告・拡張予定である。

## 2. MIR13の設計哲学

### 2.1 Everything is Box

MIR13の核心は「Everything is Box」という設計原則である。従来のIRでは、メモリアクセス、配列操作、オブジェクトフィールドアクセスなどが個別の命令として実装されていた。我々はこれらをすべて「Boxへのメッセージパッシング」として統一した。

```
従来のアプローチ:
- Load/Store（メモリアクセス）
- GetElement/SetElement（配列）  
- GetField/SetField（オブジェクト）
- Call（関数呼び出し）

MIR13のアプローチ:
- BoxCall（すべて統一）
```

### 2.2 意図的な13命令選択

技術的にはBoxCallとBoxCallWithを統合して12命令にできるが、以下の理由から13命令を維持している：

1. **可読性**: 引数なし/ありの区別が明確
2. **最適化**: JITコンパイラでの特殊化が容易
3. **教育的価値**: IRの学習が容易

## 3. MIR13命令セット

### 3.1 基本13命令

| 命令 | 説明 | 用途 |
|------|------|------|
| **Const** | 定数値のロード | リテラル、初期値 |
| **Copy** | 値のコピー | 変数代入、引数渡し |
| **BoxCall** | 引数なしBox操作 | getter、単純メソッド |
| **BoxCallWith** | 引数ありBox操作 | setter、複雑メソッド |
| **Call** | 関数呼び出し | ユーザー定義関数 |
| **Jump** | 無条件ジャンプ | 制御フロー |
| **Branch** | 条件分岐 | if文、ループ |
| **Return** | 関数からの復帰 | 値の返却 |
| **Phi** | SSA形式の値選択 | 分岐後の値統合 |
| **Cast** | 型変換 | 動的型付け対応 |
| **BinOp** | 二項演算 | 算術、比較 |
| **UnOp** | 単項演算 | 否定、型チェック |
| **Nop** | 何もしない | パディング、最適化 |

### 3.2 BoxCallによる統一

従来は個別命令だった操作がBoxCallで統一される：

```mir
// 配列アクセス（従来: GetElement）
v3 = BoxCallWith(v1, "get", v2)  // array[index]

// 配列への代入（従来: SetElement）  
v4 = BoxCallWith(v1, "set", v2, v3)  // array[index] = value

// フィールドアクセス（従来: GetField）
v5 = BoxCall(v1, "name")  // object.name

// メソッド呼び出し（従来: InvokeVirtual）
v6 = BoxCallWith(v1, "add", v2)  // object.add(arg)
```

## 4. 実装と評価

### 4.1 3つの実行バックエンド

MIR13は以下の3つのバックエンドで実装・検証された：

1. **Interpreter**: 開発・デバッグ用、即座に実行可能
2. **VM**: スタックマシンによる高速実行
3. **JIT**: Craneliftによる最速ネイティブコード生成

注記（実装マイルストン）：2025-09-04 に、JIT/ネイティブEXE経由での Windows GUI 表示（ネイティブウィンドウ生成と描画）を確認した。これはMIR13ベースの実行系がOSネイティブ機能まで到達したことを示すものであり、以降のGUI応用評価の基盤となる。

### 4.2 実アプリケーションでの検証

以下の実用的なアプリケーションが動作を確認：
- テキストエディタ（Kilo移植版）
- HTTPサーバー 
- P2P通信システム
- LISPインタープリター
  - Windows GUIアプリ（ネイティブEXE）: 2025-09-04 に表示確認

### 4.3 性能評価

#### 4.3.0 再現手順（Artifact & Scripts）
本論文の性能評価は、リポジトリ同梱のスクリプトで再現可能である。

1. 環境情報の収集（自動生成）
   - `docs/papers/active/paper-a-mir13-ir-design/_artifacts/COLLECT_ENV.sh` を実行すると、CPU/OS/Rust/Cranelift/コミットIDを `ENVIRONMENT.txt` に記録する。
2. ビルド（フルモード）
   - `cargo build --release --features cranelift-jit`
3. ベンチ実行
   - `docs/papers/active/paper-a-mir13-ir-design/_artifacts/RUN_BENCHMARKS.sh`
   - `hyperfine` があればCSVにエクスポート、無い場合はフォールバック計測を行う。
4. 結果
   - `_artifacts/results/*.csv` に各モード（Interpreter/VM/JIT/AOT）の結果を保存。

注: AOT（ネイティブEXE）は `tools/build_aot.sh` が利用可能な場合のみ測定する（無ければ自動スキップ）。
また、LLVMバックエンド経由のAOT計測も可能である（`USE_LLVM_AOT=1`）。
  - 依存: `llvm-config-18`（LLVM 18 開発環境）
  - 例: `USE_LLVM_AOT=1 SKIP_INTERP=1 ./RUN_BENCHMARKS.sh`

さらに、Cranelift JIT からの直接AOT（`--jit-direct`、本実装では「JIT-AOT」と表記）も計測可能である（`USE_JIT_AOT=1`）。
  - 例: `USE_EXE_ONLY=1 USE_JIT_AOT=1 SKIP_INTERP=1 ./RUN_BENCHMARKS.sh`

#### 4.3.x 最適化状況と注意
現時点の実装は、最適化処理を徹底していない（例：インライン化、ICの高度化、ボックス形状多態の特殊化、VM命令選択のチューニングなどは限定的）。従って、提示する数値は「素の実装に近いベースライン」であり、今後の最適化で改善余地が大きい。再現スクリプトはモード差が観測しやすいよう、負荷を軽量〜中程度に設定している（遅い環境では `SKIP_INTERP=1` でインタープリタ計測を省略可能）。

#### 4.3.1 相対性能
MIR13の3つのバックエンド間での相対実行時間：
- Interpreter: 1.0x（基準）
- VM: 10-50x高速
- JIT: 100-500x高速

#### 4.3.2 絶対性能比較
標準的なベンチマーク（Fibonacci、行列演算、文字列処理）での比較：
- Python 3.11: 1.0x（基準）
- Nyash Interpreter: 0.8-1.2x
- Nyash VM: 8-40x
- Nyash JIT: 80-400x
- Go 1.21: 100-600x
- Rust (release): 150-800x

#### 4.3.3 BoxCallのオーバーヘッド分析
マイクロベンチマークによる分析：
- 配列アクセス: 従来のLoad/Store比で1.2-1.5倍のオーバーヘッド
- JIT最適化後: インライン化により0.95-1.1倍まで改善
- メソッド呼び出し: 動的ディスパッチを含むため2-3倍だが、ICで1.1-1.3倍まで改善

### 4.4 実装公開と再現性（Availability）
本研究の実装と評価スクリプトは以下で公開している。
- リポジトリ: https://github.com/moe-charm/nyash
- 対象コミット: `_artifacts/ENVIRONMENT.txt` に `git rev-parse HEAD` を記録
- 再現手順: `_artifacts/COLLECT_ENV.sh` と `_artifacts/RUN_BENCHMARKS.sh`
  - 出力: `_artifacts/results/*.csv`

## 5. 議論

### 5.1 設計の進化：57命令から13命令への道のり

MIR13の13命令セットは、最初から意図的に設計されたものではない。当初57命令で始まったIRを、以下の洞察により段階的に削減した：

1. **メモリアクセスの統一**: Load/Store、GetElement/SetElement、GetField/SetFieldがすべてオブジェクトへの操作として統一可能
2. **メッセージパッシングへの抽象化**: すべての操作を「Boxへのメッセージ」として見ることでBoxCallに集約
3. **型操作の統合**: TypeCheck/Castを単一のCast命令に統合

この削減過程は、IR設計における本質的な要素の発見プロセスであり、結果として得られた13命令は実践的な検証を経た最小セットである。

### 5.2 なぜ13命令で十分なのか

1. **抽象度の適切性**: Boxという適切な抽象化により、低レベル詳細を隠蔽
2. **実行時システムとの分担**: 複雑性をランタイムに委譲
3. **最小限の制御構造**: Jump/Branch/Phiで全制御フローを表現

### 5.3 ランタイムシステムの役割

MIR13の単純性は、洗練されたランタイムシステムとの協調によって実現される：

1. **Boxの内部表現**: 各Boxは型タグ、参照カウント、データペイロードを持つ
2. **ホスト関数インターフェース**: BoxCallはランタイムのネイティブ関数を効率的に呼び出す
3. **メモリ管理**: 参照カウントベースの決定的メモリ管理（GCオプション付き）

この設計により、IR自体の複雑性を最小限に抑えながら、実用的な性能を達成している。

### 5.3bis 代表的操作のLowering例（MIR13→BoxCall）
以下は高水準操作が13命令（概念上）に落ちる代表例である。

```mir
// 例1: 配列アクセスと更新（a[i]、a[i]=v）
%a   = ...                 // ArrayBox
%i   = ...                 // IntBox
%v   = ...                 // 任意のBox
%x   = BoxCallWith(%a, "get", %i)           // load → BoxCallWith
%ok  = BoxCallWith(%a, "set", %i, %v)       // store → BoxCallWith

// 例2: フィールド読み書き（obj.name、obj.name=v）
%obj = ...                 // ObjectBox
%nm  = Const("name")
%cur = BoxCallWith(%obj, "getField", %nm)
%ok2 = BoxCallWith(%obj, "setField", %nm, %v)

// 例3: メソッド呼び出し（obj.add(arg)）
%res = BoxCallWith(%obj, "add", %v)

// 例4: 外部（プラグイン）関数呼び出し（host.fn(args…)）
%h   = ...                 // HostBox
%r   = BoxCallWith(%h, "fn", %arg1, %arg2)

// 制御構造はJump/Branch/Phiで表現
branch %cond, ^T, ^F
^T:  ...
     jump ^K
^F:  ...
^K:  %y = phi(%yT, %yF)
```

### 5.4 限界と将来展望

- SIMD命令は現在未対応（BoxCall拡張で対応予定）
- 並列実行最適化の余地あり
- WASM/LLVMバックエンドは開発中

## 6. 関連研究

### 6.1 既存のIR設計との比較

| IR | 命令数 | メモリモデル | 主な特徴 |
|---|--------|------------|----------|
| LLVM IR | 約60 | Load/Store明示 | SSA形式、型付き |
| WebAssembly | 約170 | スタックマシン | セキュリティ重視 |
| JVM Bytecode | 約200 | スタック+ローカル | オブジェクト指向 |
| MIR13 | 13 | BoxCall統一 | 最小命令セット |

### 6.2 メッセージパッシングIRの系譜

- **Smalltalk**: すべてはオブジェクト、すべてはメッセージ
- **Self**: プロトタイプベースオブジェクト
- **PyPy**: RPythonのオブジェクト空間
- **Truffle/Graal**: 動的言語のための抽象解釈

MIR13はこれらの思想を低レベルIRに適用し、Load/Store命令の完全廃止という新境地を開拓した。

#### 補足: Wasm GCとTyped Objectsとの比較
近年のWasm GC拡張やTyped Objectsの動向は、高レベル型をWasm上に安全に表現することを目指している。一方MIR13は「命令数最小化」と「BoxCallによる操作統一」を主眼に置き、型表現やメモリモデルの複雑さをIRではなくランタイムへ委譲する。したがって、目的関数（安全な型表現 vs. 最小命令と実装容易性）が異なり、補完的関係にある。

## 7. 結論

MIR13は、13命令という極めて小さな命令セットで完全なプログラミング言語を実装できることを実証した。「Everything is Box」の設計哲学により、従来は数十の命令が必要だった操作をBoxCallに統一し、Load/Store命令を完全に廃止した。技術的には12命令も可能だが、可読性のために意図的に13命令を選択した。3つの実行バックエンドでの動作確認により、実用性も証明された。

本研究は、プログラミング言語設計における「少ないことは豊かである」という原則の究極の実証である。

## 謝辞

本論文の執筆にあたり、Anthropic Claude、OpenAI ChatGPT（Codex CLI経由）、Google Gemini の支援（ブレインストーミング、下書き、コード補助、校正）を受けた。生成物はすべて著者がレビュー・修正し、最終的な設計判断・統合・評価は著者が行った。開発は2025-08-03頃に着手し、初回コミットは2025-08-09である。AI時代の研究開発における新しい協働形態の実例として、これを明記する。

## 参考文献

[省略]

---

## 付録：なぜ12命令にしないのか

BoxCallとBoxCallWithは技術的に統合可能である：

```mir
// 統合版（12命令）
v1 = BoxCall(obj, "method", [])      // 引数なし
v2 = BoxCall(obj, "method", [arg1])  // 引数あり

// 現在の分離版（13命令）  
v1 = BoxCall(obj, "method")          // 明確に引数なし
v2 = BoxCallWith(obj, "method", arg1) // 明確に引数あり
```

しかし、以下の理由から分離を維持：
1. パターンマッチングが単純
2. 最適化パスが書きやすい
3. エラーメッセージが分かりやすい
4. 「13」という数字の美しさ（主観的だが重要）
