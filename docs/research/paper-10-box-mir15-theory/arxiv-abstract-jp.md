# arXiv用アブストラクト（日本語版）

## 題目
Everything is Box × MIR-15: 30日でVM/JIT/AOTまで通す最小言語設計

## 概要
Nyash は「Everything is Box」を核に、15命令のMIRで VM/JIT/AOT/GC/非同期を追加命令なしで貫通させた。Boxにメタ情報を集約し、プラグインは `ExternCall` に一本化、Lowerer/JIT は"世界を知らない"。VM/JIT/AOT×GC on/off の I/Oトレース一致で意味論等価を検証し、4K行規模で実装を提示。結果、設計の純度を保ったまま、配布可能EXEと高い拡張性（GPU/量子/外部FFI）を両立できることを示す。

（180字）