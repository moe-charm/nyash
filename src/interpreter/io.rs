/*!
 * I/O Processing Module
 * 
 * Extracted from core.rs - file operations and communication
 * Handles include system, arrow operators, and I/O-related operations
 * Core philosophy: "Everything is Box" with secure I/O processing
 */

use super::*;
use crate::parser::NyashParser;

impl NyashInterpreter {
    /// Resolve include path using nyash.toml [include.roots]
    fn resolve_include_path(&self, filename: &str, caller_dir: Option<&str>) -> String {
        // If explicit relative path, resolve relative to caller when provided
        if filename.starts_with("./") || filename.starts_with("../") {
            return filename.to_string();
        }
        // Try nyash.toml roots: key/path where key is first segment before '/'
        let parts: Vec<&str> = filename.splitn(2, '/').collect();
        if parts.len() == 2 {
            let root = parts[0];
            let rest = parts[1];
            let cfg_path = "nyash.toml";
            if let Ok(toml_str) = std::fs::read_to_string(cfg_path) {
                if let Ok(toml_val) = toml::from_str::<toml::Value>(&toml_str) {
                    if let Some(include) = toml_val.get("include") {
                        if let Some(roots) = include.get("roots").and_then(|v| v.as_table()) {
                            if let Some(root_path_val) = roots.get(root).and_then(|v| v.as_str()) {
                                let mut base = root_path_val.to_string();
                                if !base.ends_with('/') && !base.ends_with('\\') { base.push('/'); }
                                let joined = format!("{}{}", base, rest);
                                return joined;
                            }
                        }
                    }
                }
            }
        }
        // Fallback: if caller_dir provided, join relative
        if let Some(dir) = caller_dir {
            if !filename.starts_with('/') && !filename.contains(":\\") && !filename.contains(":/") {
                return format!("{}/{}", dir.trim_end_matches('/'), filename);
            }
        }
        // Default to ./filename
        format!("./{}", filename)
    }
    /// include文を実行：ファイル読み込み・パース・実行 - File inclusion system
    pub(super) fn execute_include(&mut self, filename: &str) -> Result<(), RuntimeError> {
        // パス解決（nyash.toml include.roots + 相対）
        let mut canonical_path = self.resolve_include_path(filename, None);
        // 拡張子補完・index対応
        if std::path::Path::new(&canonical_path).is_dir() {
            let idx = format!("{}/index.nyash", canonical_path.trim_end_matches('/'));
            canonical_path = idx;
        } else if std::path::Path::new(&canonical_path).extension().is_none() {
            canonical_path.push_str(".nyash");
        }
        // 循環検出（ロード中スタック）
        {
            let mut stack = self.shared.include_stack.lock().unwrap();
            if let Some(pos) = stack.iter().position(|p| p == &canonical_path) {
                // 検出: A -> ... -> B -> A
                let mut chain: Vec<String> = stack[pos..].to_vec();
                chain.push(canonical_path.clone());
                let msg = format!("include cycle detected: {}",
                    chain.join(" -> "));
                return Err(RuntimeError::InvalidOperation { message: msg });
            }
            stack.push(canonical_path.clone());
        }

        // 重複読み込みチェック
        if self.shared.included_files.lock().unwrap().contains(&canonical_path) {
            // スタックから外して早期終了
            self.shared.include_stack.lock().unwrap().pop();
            return Ok(()); // 既に読み込み済み
        }
        
        // ファイル読み込み
        let content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Failed to read file '{}': {}", filename, e),
            })?;
        
        // パース
        let ast = NyashParser::parse_from_string(&content)
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Parse error in '{}': {:?}", filename, e),
            })?;
        
        // 重複防止リストに追加
        self.shared.included_files.lock().unwrap().insert(canonical_path.clone());
        
        // 現在の環境で実行
        let exec_res = self.execute(ast);
        // スタックを外す
        self.shared.include_stack.lock().unwrap().pop();
        // 実行結果を伝播
        exec_res?;
        
        Ok(())
    }
    
    /// include式を実行：ファイルを評価し、最初のstatic boxを返す
    pub(super) fn execute_include_expr(&mut self, filename: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // パス解決（nyash.toml include.roots + 相対）
        let mut canonical_path = self.resolve_include_path(filename, None);
        // 拡張子補完・index対応
        if std::path::Path::new(&canonical_path).is_dir() {
            let idx = format!("{}/index.nyash", canonical_path.trim_end_matches('/'));
            canonical_path = idx;
        } else if std::path::Path::new(&canonical_path).extension().is_none() {
            canonical_path.push_str(".nyash");
        }

        // 循環検出（ロード中スタック）
        {
            let mut stack = self.shared.include_stack.lock().unwrap();
            if let Some(pos) = stack.iter().position(|p| p == &canonical_path) {
                let mut chain: Vec<String> = stack[pos..].to_vec();
                chain.push(canonical_path.clone());
                let msg = format!("include cycle detected: {}", chain.join(" -> "));
                return Err(RuntimeError::InvalidOperation { message: msg });
            }
            stack.push(canonical_path.clone());
        }

        // ファイル読み込み（static box名検出用）
        let content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Failed to read file '{}': {}", filename, e),
            })?;

        // パースして最初のstatic box名を特定
        let ast = NyashParser::parse_from_string(&content)
            .map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Parse error in '{}': {:?}", filename, e),
            })?;

        let mut static_names: Vec<String> = Vec::new();
        if let crate::ast::ASTNode::Program { statements, .. } = &ast {
            for st in statements {
                if let crate::ast::ASTNode::BoxDeclaration { name, is_static, .. } = st {
                    if *is_static { static_names.push(name.clone()); }
                }
            }
        }

        if static_names.is_empty() {
            return Err(RuntimeError::InvalidOperation { message: format!("include target '{}' does not define a static box", filename) });
        }
        if static_names.len() > 1 {
            return Err(RuntimeError::InvalidOperation { message: format!("include target '{}' defines multiple static boxes; exactly one is required", filename) });
        }
        let box_name = static_names.remove(0);

        // まだ未読なら評価（重複読み込みはスキップ）
        let already = {
            let set = self.shared.included_files.lock().unwrap();
            set.contains(&canonical_path)
        };
        if !already {
            self.shared.included_files.lock().unwrap().insert(canonical_path.clone());
            let exec_res = self.execute(ast);
            // スタックを外す
            self.shared.include_stack.lock().unwrap().pop();
            exec_res?;
        } else {
            // スタックを外す（既に読み込み済みのため）
            self.shared.include_stack.lock().unwrap().pop();
        }

        // static boxを初期化・取得して返す
        self.ensure_static_box_initialized(&box_name)?;

        // statics名前空間からインスタンスを取り出す
        let global_box = self.shared.global_box.lock()
            .map_err(|_| RuntimeError::RuntimeFailure { message: "Failed to acquire global box lock".to_string() })?;
        let statics = global_box.get_field("statics").ok_or(RuntimeError::TypeError { message: "statics namespace not found in GlobalBox".to_string() })?;
        let statics_inst = statics.as_any().downcast_ref::<crate::instance_v2::InstanceBox>()
            .ok_or(RuntimeError::TypeError { message: "statics field is not an InstanceBox".to_string() })?;
        let value = statics_inst.get_field(&box_name)
            .ok_or(RuntimeError::InvalidOperation { message: format!("Static box '{}' not found after include", box_name) })?;

        Ok((*value).clone_or_share())
    }
    
    /// Arrow演算子を実行: sender >> receiver - Channel communication
    pub(super) fn execute_arrow(&mut self, sender: &ASTNode, receiver: &ASTNode) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        // 送信者を評価
        let sender_value = self.execute_expression(sender)?;
        
        // 受信者を評価
        let receiver_str = match receiver {
            ASTNode::Variable { name, .. } => name.clone(),
            ASTNode::Literal { value, .. } => {
                // "*" のようなリテラルの場合
                value.to_string()
            }
            _ => {
                // その他の式の場合は評価して文字列化
                let receiver_value = self.execute_expression(receiver)?;
                receiver_value.to_string_box().value
            }
        };
        
        // 送信者の名前を取得
        let sender_name = sender_value.to_string_box().value;
        
        // ChannelBoxを作成して返す
        let channel_box = Box::new(ChannelBox::new(&sender_name, &receiver_str)) as Box<dyn NyashBox>;
        // 🌍 革命的実装：Environment tracking廃止
        Ok(channel_box)
    }
    
    /// nowait文を実行 - 非同期実行（真の非同期実装） - Async execution
    pub(super) fn execute_nowait(&mut self, variable: &str, expression: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        use crate::boxes::FutureBox;
        
        // FutureBoxを作成
        let future_box = FutureBox::new();
        // 個別のクローンを用意（スケジュール経路とフォールバック経路で別々に使う）
        let future_for_sched = future_box.clone();
        let future_for_thread = future_box.clone();
        
        // 式をクローンしてスケジューラ（なければフォールバック）で実行
        // それぞれの経路で独立に所有させるためクローンを分けておく
        let expr_for_sched = expression.clone();
        let expr_for_thread = expression.clone();
        let shared_for_sched = self.shared.clone();
        let shared_for_thread = self.shared.clone();
        // Phase-2: try scheduler first (bound to current TaskGroup token); fallback to thread
        let token = crate::runtime::global_hooks::current_group_token();
        let scheduled = crate::runtime::global_hooks::spawn_task_with_token(
            "nowait",
            token,
            Box::new(move || {
                // 新しいインタープリタインスタンスを作成（SharedStateを使用）
                let mut async_interpreter = NyashInterpreter::with_shared(shared_for_sched);
                // 式を評価
                match async_interpreter.execute_expression(&expr_for_sched) {
                    Ok(result) => { future_for_sched.set_result(result); }
                    Err(e) => {
                        // エラーをErrorBoxとして設定
                        let error_box = Box::new(ErrorBox::new("RuntimeError", &format!("{:?}", e)));
                        future_for_sched.set_result(error_box);
                    }
                }
            })
        );
        if !scheduled {
            std::thread::spawn(move || {
                let mut async_interpreter = NyashInterpreter::with_shared(shared_for_thread);
                match async_interpreter.execute_expression(&expr_for_thread) {
                    Ok(result) => { future_for_thread.set_result(result); }
                    Err(e) => {
                        let error_box = Box::new(ErrorBox::new("RuntimeError", &format!("{:?}", e)));
                        future_for_thread.set_result(error_box);
                    }
                }
            });
        }
        
        // FutureBoxを現在のTaskGroupに登録（暗黙グループ best-effort）
        crate::runtime::global_hooks::register_future_to_current_group(&future_box);
        // FutureBoxを変数に保存
        let future_box_instance = Box::new(future_box) as Box<dyn NyashBox>;
        self.set_variable(variable, future_box_instance)?;

        Ok(Box::new(VoidBox::new()))
    }
}
