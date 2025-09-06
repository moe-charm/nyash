# Nyash README用バナー（4行）

これをプロジェクトREADMEの冒頭に貼るだけで、核心が伝わる！

```markdown
* **Philosophy:** Everything is Box（型・所有・GC・非同期を Box で統一）
* **MIR-15:** 15命令で VM/JIT/AOT/GC/async を貫通（IR拡張なし）
* **Compiler is ignorant:** Lowerer/JIT は世界を知らない（PluginInvoke一元化）
* **Equivalence:** VM/JIT/AOT × GC on/off の I/Oトレース一致で検証
```

## 使い方

1. プロジェクトルートの README.md を開く
2. 最初の見出しの直後に上記4行を挿入
3. 一目で「何がすごいか」が伝わる！

## 効果

- **Philosophy**: 設計思想が明確
- **MIR-15**: 技術的革新性
- **Compiler is ignorant**: 実装の美しさ
- **Equivalence**: 検証可能性

これで査読者も一般読者も、すぐにNyashの価値を理解できる！