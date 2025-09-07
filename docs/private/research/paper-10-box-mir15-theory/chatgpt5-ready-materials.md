# ChatGPT5さんの「すぐ使える原稿集」

Date: 2025-08-31

めちゃ褒められて当然の内容だと思うけど、ここは**"褒めの勢いを成果物に変える"**のが吉だにゃ。バランスよく謙虚さも添えて、すぐ貼れる原稿を用意したよ。

## 1) arXiv 用アブスト（JP, 180–200字）

**題:** Everything is Box × MIR-15: 30日でVM/JIT/AOTまで通す最小言語設計

**概要:**
Nyash は「Everything is Box」を核に、15命令のMIRで VM/JIT/AOT/GC/非同期を追加命令なしで貫通させた。Boxにメタ情報を集約し、プラグインは `ExternCall` に一本化、Lowerer/JIT は"世界を知らない"。VM/JIT/AOT×GC on/off の I/Oトレース一致で意味論等価を検証し、4K行規模で実装を提示。結果、設計の純度を保ったまま、配布可能EXEと高い拡張性（GPU/量子/外部FFI）を両立できることを示す。

## 2) Abstract (EN, ~150 words)

**Title:** Everything is Box meets MIR-15: A Minimal, Equivalence-Preserving Path from Language to Native in 30 Days

**Abstract:**
We present Nyash, a language architecture centered on "Everything is a Box." A 15-instruction MIR suffices to implement VM, JIT, AOT, GC, and async—without extending the IR. All high-level features funnel through Box and a unified plugin boundary via `ExternCall`, while the Lowerer/JIT remain world-agnostic. We validate semantic equivalence by matching end-to-end I/O traces across `{VM,JIT,AOT} × {GC on,off}` and report a ~4 KLoC reference implementation leveraging Cranelift. Nyash shows that a minimal, consistent core can deliver native executables, strong extensibility (e.g., GPU/quantum/FFI), and practical performance, offering a short, principled route from language design to deployable binaries.

## 3) README 冒頭に貼る 4行バナー

* **Philosophy:** Everything is Box（型・所有・GC・非同期を Box で統一）
* **MIR-15:** 15命令で VM/JIT/AOT/GC/async を貫通（IR拡張なし）
* **Compiler is ignorant:** Lowerer/JIT は世界を知らない（PluginInvoke一元化）
* **Equivalence:** VM/JIT/AOT × GC on/off の I/Oトレース一致で検証

## 4) 「褒められ過ぎ」対策のバランサー（査読に強い一言）

* **制約:** まだ巨大コードベースでの最適化は限定的。O1（正規化/ピープホール/DCE）中心。
* **未解:** 高度LICM/スカラー置換/逃避解析の完全版は今後。
* **外部性:** OS/CRT/リンク手順の差分はAOTガイドで吸収（WindowsはMSVC+clang推奨）。
* **再現:** `make smoke` で `{VM,JIT,AOT}×{GC on/off}` の `trace_hash` を自動検証。

## 5) レビュアーが聞きそうな質問→想定回答（ショート）

* **Q:** 15命令で本当に足りる？  
  **A:** 高機能はプラグインへ押し出し、MIRは「What」だけ。`ExternCall` 経由で拡張し、IR拡張は不要。

* **Q:** フォールバックは？  
  **A:** 全廃。VM=仕様、JIT=高速版。未実装は即エラー＋該当VM関数への誘導で修正サイクルを短縮。

* **Q:** 最適化は弱くない？  
  **A:** O1/O2を表駆動で段階導入。等価性を崩さず、ホット箇所はプラグイン側の vtable 直結とAOT/LTOで補完。

* **Q:** 既存比較（Wasm/LLVM/Smalltalk）との差は？  
  **A:** Box でメタ情報を一元管理し、**IRを増やさず**VM/JIT/AOT等価を実証。実装規模と到達速度が新規性。

## 6) すぐ作れる図のキャプション案（文章だけ）

**Fig.1 Nyash pipeline.** Source → MIR-15 → {VM | JIT | AOT} → EXE.
Lowerer/JIT are world-agnostic; plugins bind via a unified `ExternCall`.
Equivalence is validated by I/O trace hashing across engines and GC modes.

## 7) 次アクション（超短距離）

* `docs/mir-v0.15.md` をこのまま置く → 15命令と不変条件を固定
* `core/spec/ops_map.rs` に 1:1 対応表を実装（未登録でビルド失敗）
* `make smoke` を README に記載（trace_hash 例を添付）
* 上の Abstract を arXiv/技術ブログに投下（ベンチJSON1枚だけ添える）

---

胸を張ってOK。ただし**"規格化＋再現パッケージ"**を最優先で固めよう。
これで褒めが"事実"にロックされるし、次の査読でも強いにゃ。