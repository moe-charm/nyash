# 実装戦略 - メソッド後置例外処理

## 段階的実装ロードマップ

### Phase 15.6: 段階的意思決定モデル（基盤確立）

#### 目標
メソッドレベルでの段階的意思決定を可能にする catch/cleanup 構文を追加

#### 核心概念：三段階意思決定
1. **Stage 1**: 通常処理（メソッド本体）
2. **Stage 2**: エラー処理（catch ブロック）
3. **Stage 3**: 最終調整（cleanup ブロック）

#### 構文設計
```nyash
box StageDecisionBox {
    // ✅ 既存構文（互換維持）
    method traditional(arg) {
        return computation(arg)
    }
    
    // 🆕 段階的意思決定構文（安全モード）
    method safeStaged(arg) {
        return riskyOperation(arg)
    } catch (e) {
        me.logError(e)
        return defaultValue
    } cleanup {
        // リソース管理のみ（return禁止）
        me.releaseResources()
    }
    
    // 🔥 段階的意思決定構文（表現モード）
    method expressiveStaged(arg) {
        return complexOperation(arg)
    } catch (e) {
        me.logError(e)
        return errorValue
    } cleanup returns {
        // 最終判断も可能（return許可）
        me.releaseResources()
        if me.detectSecurityThreat(arg) {
            return "SECURITY_BLOCKED"
        }
    }
}
```

#### 技術的実装

**パーサー拡張**:
```rust
// src/parser/statements.rs に追加
fn parse_method_definition(&mut self) -> Result<ASTNode, ParseError> {
    // 既存のメソッドパース
    let method = self.parse_traditional_method()?;
    
    // 後置修飾子のチェック
    if self.current_token_is("catch") || self.current_token_is("cleanup") {
        let postfix = self.parse_method_postfix_modifiers()?;
        return Ok(ASTNode::MethodWithStaging { method, postfix });
    }
    
    Ok(method)
}

fn parse_method_postfix_modifiers(&mut self) -> Result<StagingModifiers, ParseError> {
    let mut catch_clause = None;
    let mut cleanup_clause = None;
    
    if self.consume_if("catch") {
        catch_clause = Some(self.parse_catch_clause()?);
    }
    
    if self.consume_if("cleanup") {
        // cleanup returns の検出
        let allow_returns = self.consume_if("returns");
        cleanup_clause = Some(self.parse_cleanup_clause(allow_returns)?);
    }
    
    Ok(StagingModifiers { catch_clause, cleanup_clause })
}

fn parse_cleanup_clause(&mut self, allow_returns: bool) -> Result<CleanupClause, ParseError> {
    let body = self.parse_block()?;
    
    // 安全性チェック：cleanup（returnsなし）では return/throw を禁止
    if !allow_returns {
        self.validate_cleanup_safety(&body)?;
    }
    
    Ok(CleanupClause {
        body,
        allow_returns,
    })
}

fn validate_cleanup_safety(&self, block: &Block) -> Result<(), ParseError> {
    for stmt in &block.statements {
        match stmt {
            ASTNode::Return(_) => {
                return Err(ParseError::CleanupCannotReturn {
                    hint: "Use 'cleanup returns' if intentional final decision is needed"
                });
            }
            ASTNode::Throw(_) => {
                return Err(ParseError::CleanupCannotThrow {
                    hint: "cleanup blocks are for resource management only"
                });
            }
            _ => {}
        }
    }
    Ok(())
}
```

**AST変換**:
```rust
// メソッド後置をTryCatchに正規化
impl ASTNode {
    fn normalize_method_postfix(method: Method, postfix: PostfixModifiers) -> Method {
        Method {
            name: method.name,
            params: method.params,
            body: vec![ASTNode::TryCatch {
                try_body: method.body,
                catch_clauses: postfix.catch_clause.into_iter().collect(),
                finally_clause: postfix.finally_clause,
            }],
            return_type: method.return_type,
        }
    }
}
```

**Bridge互換性**:
- 既存のResult-mode lowering完全再利用
- ThrowCtx機構そのまま活用
- PHI-off (edge-copy) 合流維持

#### 仕様の明確化（Phase 15.6）

EBNF（段階的意思決定構文）
```
methodDecl := 'method' IDENT '(' params? ')' block stagingModifiers?

stagingModifiers := catchClause? cleanupClause?

catchClause := 'catch' '(' (IDENT IDENT | IDENT | ε) ')' block

cleanupClause := 'cleanup' cleanupMode block

cleanupMode := 'returns'     (* 表現モード：return/throw許可 *)
             | ε             (* 安全モード：return/throw禁止 *)
```

ゲート/フラグ
- `NYASH_STAGING_DECISION=1`（または `NYASH_PARSER_STAGE3=1` と同梱）
- `NYASH_CLEANUP_SAFETY_MODE=strict|permissive` (デフォルト: strict)

段階実行順序（時系列的意思決定）
1. **Stage 1**: メソッド本体実行
2. **Stage 2**: 例外発生時は catch ブロック実行  
3. **Stage 3**: 常に cleanup ブロック実行（安全モードまたは表現モード）

優先順位（段階的エスカレーション）
- メソッド本体内のブロック後置catchが存在する場合、そちらが優先（最も近い境界が先）
- 次にメソッドレベルcatch、最後に呼出し側（外側）のcatchへ伝播
- cleanup は常に最終段階で実行（例外の有無に関わらず）

スコープ/束縛
- `catch (e)` の `e` はcatchブロック内ローカル。メソッド本体スコープには漏れない
- cleanup ブロックはメソッド本体の変数スコープにアクセス可能

cleanup 実行順序と安全性
- **安全モード** (`cleanup`): return/throw禁止、純粋リソース管理のみ
- **表現モード** (`cleanup returns`): 最終判断可能、return/throwで段階3の意思決定
- cleanup 内の `return` は最終的な戻り値となる（Stage 3 の決定権）

オーバーライド/署名
- メソッドレベルcatch/cleanup はシグネチャに影響しない（戻り値型/引数は従来通り）
- cleanup returns の場合は事実上の意思決定権を持つが、型システム上は透明
- 将来: 段階効果ビット（staged effects）を導入し、継承階層での安全性を静的検査

エラー規約
- 複数catchはMVPでは非対応（構文受理する場合でも最初のみ使用）
- 順序は `catch` → `cleanup` のみ許可。逆順・重複は構文エラー
- `cleanup` と `cleanup returns` の混在は禁止（一意性の保証）

#### 実装コスト
- **パーサー**: 約100行追加
- **AST**: 約50行追加
- **Lowering**: 0行（既存再利用）
- **テスト**: 約200行

#### 期待効果
- メソッド呼び出し側のtry-catch不要
- リソース管理の自動化
- エラー処理の統一化

### Phase 16.1: メソッド後置定義（革新的構文）

#### 目標
メソッドの処理内容を先に書き、署名を後置する構文

#### 構文
```nyash
box ModernBox {
    // 🚀 後置メソッド定義
    {
        local result = heavyComputation(arg)
        return optimize(result)
    } method process(arg): ResultBox
    
    {
        return me.field1 + me.field2
    } method getSum(): IntegerBox catch (e) {
        return 0
    }
}
```

#### 技術的実装

**パーサー拡張**:
```rust
fn parse_box_member(&mut self) -> Result<ASTNode, ParseError> {
    if self.current_token_is("{") {
        // ブロックから開始 = 後置定義の可能性
        let block = self.parse_block()?;
        
        if self.current_token_is("method") {
            let method_sig = self.parse_method_signature()?;
            let postfix = self.parse_optional_postfix()?;
            
            return Ok(ASTNode::PostfixMethod {
                body: block,
                signature: method_sig,
                postfix,
            });
        }
        
        // 他の後置修飾子もチェック
        return self.parse_other_postfix_constructs(block);
    }
    
    // 従来構文
    self.parse_traditional_member()
}
```

EBNF（ブロック先行・メソッド後置）
```
postfixMethod := block 'method' IDENT '(' params? ')' ( 'catch' '(' (IDENT IDENT | IDENT | ε) ')' block )? ( 'finally' block )?
```

先読み/二段化方針
- `}` 後のトークンを先読みして `method|as|catch|finally` を判定。`method` の場合は署名を読み、既に取得した block を try_body として束ねる。
- ブロック内で参照する引数名は「前方宣言」扱いとし、パーサ内部で仮束縛テーブルを用意して解決する（実装は二段正規化で可）。

#### 実装コスト
- **パーサー**: 約150行追加
- **AST正規化**: 約100行
- **ドキュメント**: 大幅更新

#### 期待効果
- 思考の自然な流れ（処理→署名）
- 複雑なメソッドの可読性向上
- コードレビューの効率化

### Phase 16.2: 究極統一構文（哲学的完成）

#### 目標
Everything is Block + Modifierの完全実現

#### 構文
```nyash
box UnifiedBox {
    // 🔥 Everything is Block + Modifier
    {
        return "Hello " + me.name
    } as field greeting: StringBox
    
    {
        return me.items.count()  
    } as property size: IntegerBox
    
    {
        return complexCalculation(arg)
    } as method compute(arg): ResultBox catch (e) {
        return ErrorResult(e)
    }
    
    {
        return asyncOperation()
    } as async method fetch(): FutureBox finally {
        me.closeConnections()
    }
}
```

#### 技術的実装

**統一パーサー**:
```rust
enum BlockModifier {
    AsField { name: String, type_hint: Option<Type> },
    AsProperty { name: String, type_hint: Option<Type> },
    AsMethod { 
        name: String, 
        params: Vec<Parameter>,
        return_type: Option<Type>,
        is_async: bool,
    },
    WithCatch { param: Option<String>, body: Block },
    WithFinally { body: Block },
}

fn parse_block_with_modifiers(&mut self) -> Result<ASTNode, ParseError> {
    let block = self.parse_block()?;
    let mut modifiers = Vec::new();
    
    while let Some(modifier) = self.parse_optional_modifier()? {
        modifiers.push(modifier);
    }
    
    Ok(ASTNode::BlockWithModifiers { block, modifiers })
}
```

**意味解析**:
```rust
impl BlockWithModifiers {
    fn into_traditional_ast(self) -> Result<ASTNode, SemanticError> {
        match self.modifiers.primary_modifier() {
            BlockModifier::AsField { .. } => self.to_field(),
            BlockModifier::AsMethod { .. } => self.to_method(),
            BlockModifier::AsProperty { .. } => self.to_property(),
        }
    }
}
```

#### 実装コスト
- **パーサー**: 約300行（大幅改造）
- **意味解析**: 約200行
- **エラーメッセージ**: 約100行
- **移行ツール**: 約500行

#### 期待効果
- データと振る舞いの完全統一
- コンパイラ最適化の新機会
- 言語学習コストの削減

## 互換性戦略

### 後方互換性の確保
```nyash
box MixedBox {
    // ✅ 従来構文（永続サポート）
    field oldField: StringBox
    
    method oldMethod() {
        return traditional()
    }
    
    // ✅ 新構文（段階的導入）
    {
        return computed()
    } as field newField: StringBox
    
    method newMethod() {
        return modern()
    } catch (e) {
        return fallback()
    }
}
```

### 移行支援ツール
```bash
# 自動変換ツール
nyash-migrate --from traditional --to postfix src/

# 段階的移行
nyash-migrate --phase 15.6 src/  # メソッドレベルcatch/finally
nyash-migrate --phase 16.1 src/  # 後置定義
nyash-migrate --phase 16.2 src/  # 統一構文
```

## 性能への影響

### 実行時性能
- **変化なし**: AST正規化により従来と同じMIR生成
- **最適化機会**: 統一構文による新しい最適化パス

### コンパイル時間
- **Phase 15.6**: 影響なし（既存パス再利用）
- **Phase 16.1**: +5-10%（パーサー複雑化）
- **Phase 16.2**: +10-15%（意味解析追加）

### メモリ使用量
- **AST**: +10-15%（中間表現の追加）
- **実行時**: 変化なし（同じMIR生成）

## テスト戦略

### 段階的テスト
```bash
# Phase 15.6 テスト
./test/method-postfix-catch/
├── basic-catch.nyash
├── basic-finally.nyash  
├── catch-and-finally.nyash
├── nested-methods.nyash
└── resource-management.nyash

# Phase 16.1 テスト  
./test/postfix-method-definition/
├── simple-postfix.nyash
├── complex-signature.nyash
├── with-catch.nyash
└── mixed-styles.nyash

# Phase 16.2 テスト
./test/unified-syntax/
├── field-method-property.nyash
├── async-methods.nyash
├── performance-comparison.nyash
└── migration-compatibility.nyash
```

### 回帰テストの確保
- 全既存テストがPhase 15.6で継続PASS
- 新構文と旧構文の等価性検証
- パフォーマンス回帰の監視

## リスク評価と対策

### 技術的リスク
1. **パーサー複雑化** → 段階的実装で複雑性管理
2. **エラーメッセージ** → 専用のエラー報告機構
3. **IDE対応** → Language Server Protocol拡張

### 採用リスク  
1. **学習コスト** → 段階的導入と豊富なドキュメント
2. **コミュニティ分裂** → 互換性維持と移行支援
3. **ツール対応** → 既存ツールとの互換性確保

### 対策
- **保守的ロールアウト**: 各フェーズで十分な検証期間
- **フィードバック収集**: ベータユーザーからの積極的意見収集
- **フォールバック計画**: 問題発生時の迅速な巻き戻し手順

## 成功指標

### 技術指標
- [ ] 全既存テストがPASS（100%）
- [ ] 新構文テストカバレッジ（95%+）
- [ ] パフォーマンス劣化なし（±3%以内）
- [ ] コンパイル時間増加最小（+15%以下）

### 利用指標
- [ ] メソッドレベル例外処理の採用率（50%+）
- [ ] 後置定義構文の採用率（30%+）
- [ ] 統一構文の採用率（20%+）
- [ ] 開発者満足度向上（Survey）

### 影響指標
- [ ] 例外関連バグの削減（30%+）
- [ ] コードレビュー時間短縮（20%+）
- [ ] 新規開発者の学習時間短縮（25%+）
- [ ] 他言語からの影響検証（学術追跡）

## 🔥 Property System革命 - 最終決定事項（9月18日 4AM）

### ChatGPT5との最終合意

**採用決定**: Header-first構文を正式採用、Block-first構文との両立戦略確立

### 最終実装戦略

#### Phase 1: Header-first構文（最優先）
```nyash
box Example {
    // stored: 読み書き可能
    name: StringBox = "default"
    counter: IntegerBox = 0
    
    // computed: 毎回計算（読み専用）
    size: IntegerBox { me.items.count() }
    greeting: StringBox => "Hello " + me.name  // ショートハンド
    
    // once: 遅延評価+キャッシュ（読み専用）
    once cache: CacheBox { buildExpensiveCache() }
    once data: DataBox => loadData()  // ショートハンド
    
    // birth_once: 即座評価+保存（読み専用）
    birth_once config: ConfigBox { loadConfiguration() }
    birth_once token: StringBox => readEnv("TOKEN")  // ショートハンド
}
```

#### Phase 2: Block-first構文（共存）
```nyash
box Example {
    // 哲学的統一：まず計算→役割付与
    { return "Hello " + me.baseName } as greeting: StringBox
    { return buildCache() } as once cache: CacheBox
    { return loadConfig() } as birth_once config: ConfigBox
}
```

### 技術決定事項

#### 視覚ルール確立
- **`=` が見える** → stored（読み書き可能）
- **`{}` または `=>` が見える** → 読み専用（computed系）

#### パーサー実装戦略
```rust
// メンバ領域での二分岐（曖昧性なし）
if current_token == '{' {
    parse_block_first_member()  // 共存時
} else {
    parse_header_first_member() // 既定
}
```

#### 例外処理統合
```nyash
// stored以外でcatch/cleanup許可
once cache: CacheBox {
    return buildCache()
} catch (e) {
    return EmptyCache()  // フォールバック（キャッシュ）
} cleanup {
    log("Cache init finished")
}
```

#### Poison-on-throw戦略
- **Success Path**: 計算成功 → 値キャッシュ
- **Fallback Path**: 例外発生、catch成功 → フォールバックキャッシュ  
- **Poison Path**: 例外発生、catchなし → 永続poison、再アクセス時即再throw

### フォーマッタ戦略

#### 正規化方針
```bash
# nyfmt.toml設定
style = "header-first"  # 既定
# または
style = "block-first"   # 哲学重視

# 個別制御
# nyfmt: keep-block-first  # コメントで保持指示
```

#### 往復変換
- Header-first ⇄ Block-first 完全等価変換
- プロジェクト設定で統一スタイル選択可能

### 実装順序確定

#### ステップ1（最小実装）
- Header-first全対応（stored/computed/once/birth_once）
- `=>` ショートハンド砂糖
- Block-first未実装（丁寧なエラーメッセージ）

#### ステップ2（軽量拡張）
- Block-first受理追加（メンバ領域限定）
- 内部ASTはheader-firstと同一形に正規化
- handlersなし

#### ステップ3（完全対応）
- Block-first にもcatch/cleanup対応
- フォーマッタ実装

### 成功指標追加

#### Property System採用率
- [ ] Header-first構文採用率（90%+）
- [ ] Block-first構文採用率（10%+、哲学重視場面）
- [ ] フォーマッタ正規化成功率（100%）

#### 開発効率向上
- [ ] プロパティ実装時間短縮（40%+）
- [ ] 型推論支援効果測定
- [ ] IDE補完改善効果

### 革新的意義

この実装戦略により：

1. **実用性**: Header-first でツール対応最優先
2. **革新性**: Block-first で哲学的統一保持
3. **柔軟性**: フォーマッタで両者統一
4. **段階性**: 最小リスクで段階的展開

**結論**: 現実的革命の完璧な実現戦略確立

---

**最終更新**: 2025年9月18日 4AM
**実装責任者**: ChatGPT5 (最終設計) + 段階的実装チーム  
**状態**: 全設計完了、実装準備100%完了、学術論文準備完了
