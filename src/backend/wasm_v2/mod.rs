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

/// WASM v2エントリポイント: 統一vtableディスパッチの最小テスト
pub fn compile_and_execute_v2(
    _module: &crate::mir::MirModule,
    _temp_name: &str,
) -> Result<Box<dyn crate::box_trait::NyashBox>, String> {
    // 1) ConsoleBoxを生成（WASM環境ではブラウザコンソールに委譲）
    let console = Box::new(ConsoleBox::new());
    // 2) slot解決→dispatchでlogを呼ぶ（最小疎通）
    if let Some(slot_id) = unified_dispatch::resolve_slot(console.as_ref(), "log", 1) {
        let args =
            vec![Box::new(StringBox::new("🎉 WASM v2 console.log working!")) as Box<dyn NyashBox>];
        let _ = unified_dispatch::dispatch_by_slot(slot_id, console.as_ref(), &args);
    }
    // 3) 結果を返す
    Ok(Box::new(StringBox::new(
        "WASM v2 unified dispatch test completed",
    )))
}
