use crate::instance_v2::InstanceBox;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use super::{BoxDeclaration, StaticBoxDefinition};

/// スレッド間で共有される状態
#[derive(Clone)]
pub struct SharedState {
    /// 🌍 GlobalBox - すべてのトップレベル関数とグローバル変数を管理
    pub global_box: Arc<Mutex<InstanceBox>>,

    /// Box宣言のレジストリ（読み込みが多いのでRwLock）
    pub box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>,

    /// 🔥 静的関数のレジストリ（読み込みが多いのでRwLock）
    pub static_functions: Arc<RwLock<HashMap<String, HashMap<String, crate::ast::ASTNode>>>>,

    /// 🔥 Static Box定義レジストリ（遅延初期化用）
    pub static_box_definitions: Arc<RwLock<HashMap<String, StaticBoxDefinition>>>,

    /// 読み込み済みファイル（重複防止）
    pub included_files: Arc<Mutex<HashSet<String>>>,
}

impl SharedState {
    /// 新しい共有状態を作成
    pub fn new() -> Self {
        let global_box = InstanceBox::new(
            "Global".to_string(),
            vec![],          // フィールド名（空から始める）
            HashMap::new(),  // メソッド（グローバル関数）
        );

        Self {
            global_box: Arc::new(Mutex::new(global_box)),
            box_declarations: Arc::new(RwLock::new(HashMap::new())),
            static_functions: Arc::new(RwLock::new(HashMap::new())),
            static_box_definitions: Arc::new(RwLock::new(HashMap::new())),
            included_files: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}
