# CAX (C-ABI Explorer) - Revolutionary Debugging Tool

**Status**: Post‑Bootstrap Implementation (Core Idea Complete)  
**Priority**: High (World-First Tool)  
**Origin**: 1-minute inspiration (C ABI dynamic → C ABI Debugger)  
**Date**: 2025-09-21  

## 🌟 Core Concept

C-ABI境界デバッグのGUIツール。**「ぽいっと付け外し」「視覚的ログ監視」「マクロ自動化」「ホットスワップ」**を実現。

### Revolutionary Aspects
- **Nyash箱理論**でC境界を完全トレース
- **Record/Replay**で回帰テスト・CI再現性
- **GUI Explorer**でプラグイン管理
- **Type Safety**境界での型検証・所有権チェック

## 📁 Files Structure

- `gemini-ipc-implementation.nyash` - Geminiの172行実装コード
- `chatgpt-design-spec.md` - ChatGPTの設計仕様
- `inspiration-process.md` - 1分発想プロセスの記録
- `technical-roadmap.md` - 実装ロードマップ（2週間MVP）

## 🎯 Implementation Priority

**Phase 1** (Post Mini-VM): IPC層 + Timeline GUI  
**Phase 2**: Record/Replay + Hot-swap  
**Phase 3**: Advanced Analytics + 可視化

## 💡 Technical Innovation

- **境界フック**: PluginHost.Invoke 層で完全インターセプト
- **統一観測**: すべてのBoxで統一されたイベントログ
- **型安全**: TypeBox境界での実時間検証
- **構造化**: RoutineBox/ChannelBox での並行デバッグ

---

**Note**: このアイデアは、C ABI動的呼び出しからわずか1分で到達した革新的発想の記録です。
