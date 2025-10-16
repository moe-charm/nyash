# LegacyCallBridgeBox

この箱は過渡的な `emit_legacy_call` 経路を集約して、旧来の呼び出し発行ロジックを閉じ込める役割を持つよ。

- **責務**: レガシーな CallTarget を評価し、最終的な `MirInstruction::Call` を `emit_call_with_guard` 経由で発行すること。
- **境界**: 新規コードはここを通さず `emit_unified_call` を使う。レガシー互換のために残すが、外からは `LegacyCallBridgeBox::emit` だけを使う。
- **禁止事項**: 直接 `emit_instruction` を叩いて Call を発行しない。必ず guard を通した素材化を行う。
- **将来計画**: レガシー経路が消滅し次第、この箱ごと削除できるよう小さく保つ。
