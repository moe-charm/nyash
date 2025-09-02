//! Unified dispatch (WASM v2)
//!
//! - TypeRegistryのスロット表と一致させた呼び出し分岐の雛形
//! - ここではあくまで「どのスロットに行くか」の判定のみ提供

#![cfg(feature = "wasm-backend")]

use crate::box_trait::NyashBox;

/// 受信ボックス/メソッド名/アリティからスロットを解決し、識別子を返す。
pub fn resolve_slot(recv: &dyn NyashBox, method: &str, arity: usize) -> Option<u16> {
    let ty = recv.type_name();
    crate::runtime::type_registry::resolve_slot_by_name(ty, method, arity)
}

/// 実際の呼び出し分岐は、将来的にここから生成済みのstubsに委譲する予定。
pub fn dispatch_by_slot(
    _slot: u16,
    _recv: &dyn NyashBox,
    _args: &[Box<dyn NyashBox>],
) -> Option<Box<dyn NyashBox>> {
    // 未実装: wasm_v2ではJS/hostへのブリッジや、Wasm内の簡易実装に委譲
    None
}

