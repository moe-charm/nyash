# Phase 11.9 文法統一化提案アーカイブ

**日付**: 2025-10-12
**ステータス**: アーカイブ（未実装提案）

## 概要

Phase 11.9「文法統一化とAI連携強化 - Grammar as Single Source of Truth」の設計資料です。

## 提案内容

**ビジョン**: Hakorune/Nyashの文法知識が分散している問題を解決し、AIがコードを正しく書けるよう文法定義を一元化する。

**主要設計**:
- `nyash-grammar-v1.yaml` - 統一文法定義ファイル（YAML形式）
- 3層アーキテクチャ: Grammar Definition Layer → Grammar Runtime → Components
- AI向けエクスポート機能（トレーニングデータ生成）
- ANCP統合（文法aware圧縮）

## ファイル一覧

- `grammar-unification.txt` (15,950バイト): アーキテクチャ全体像、AIヒント、ANCP統合設計
- `implementation-plan.txt` (11,649バイト): 実装計画詳細、6ステップ実装戦略

## 実装状況

**未実装** - このアプローチは採用されませんでした。

現在のHakoruneは以下の方式を採用：
- キーワードはTokenizer/Parser内で直接定義
- 文法検証はParser/MIR Builderで分散実行
- AI向けドキュメントは手動メンテナンス

## 歴史的価値

この提案は実装されませんでしたが、以下の点で参考価値があります：
- 文法の一元管理アプローチ
- AI連携の設計思想
- YAML駆動開発の検討過程

## 不採用理由（推測）

1. 実装コストと効果のバランス
2. YAMLメンテナンスの追加負担
3. 既存のParser/Tokenizerが十分機能している
4. Phase 12以降の優先度が高かった

## 再検討の可能性

将来、以下の状況になった場合は再検討の価値あり：
- AI連携が重要課題になったとき
- 言語仕様が頻繁に変更されるとき
- 複数フロントエンド（Parser）を管理する必要が生じたとき

## 関連Phase

- Phase 11: MIR/Parser整理
- Phase 12: プラグインシステム統一
- Phase 15: セルフホスティング計画

## アーカイブ日

2025-10-12: docs/development/roadmap/phases/phase-11.9/archive/ から移動
