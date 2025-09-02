//! WASM Backend v2 (Phase 12 scaffolding)
//!
//! 目的:
//! - vtable/スロット解決に基づく統一ディスパッチ経路の雛形
//! - 既存ビルドに影響を与えない最小構成（feature/target gate）

#![cfg(feature = "wasm-backend")]

pub mod unified_dispatch;
pub mod vtable_codegen;

use crate::box_trait::{NyashBox, StringBox};
use crate::boxes::ConsoleBox;

/// WASM v2エントリポイント: 統一vtableディスパッチで実行
pub fn compile_and_execute_v2(module: &crate::mir::MirModule, _temp_name: &str) -> Result<Box<dyn crate::box_trait::NyashBox>, String> {
    // 最小実装: env.console.log の動作確認
    
    // 1. ConsoleBoxをenvにバインド（簡易版）
    let console = Box::new(ConsoleBox::new());
    
    // 2. 統一ディスパッチでconsole.log呼び出しテスト
    let slot = unified_dispatch::resolve_slot(console.as_ref(), "log", 1);
    if let Some(slot_id) = slot {
        let test_msg = vec![Box::new(StringBox::new("🎉 WASM v2 console.log working!")) as Box<dyn NyashBox>];
        let _result = unified_dispatch::dispatch_by_slot(slot_id, console.as_ref(), &test_msg);
    }
    
    // 3. 成功メッセージを返す
    Ok(Box::new(StringBox::new("WASM v2 unified dispatch test completed")))
}

