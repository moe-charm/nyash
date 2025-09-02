/*!
 * Type Registry (Tier-0 雛形)
 *
 * 目的:
 * - TypeId → TypeBox 参照の最小インターフェースを用意（現時点では未実装・常に未登録）。
 * - VM/JIT 実装が存在を前提に呼び出しても no-op/fallback できる状態にする。
 */

use super::type_box_abi::{TypeBox, MethodEntry};

// 最小サンプル: MapBox の TypeBox を事前登録（Tier-1 PoC 用）
// --- ArrayBox ---
const ARRAY_METHODS: &[MethodEntry] = &[
    MethodEntry { name: "get", arity: 1, slot: 100 },
    MethodEntry { name: "set", arity: 2, slot: 101 },
    MethodEntry { name: "len", arity: 0, slot: 102 },
    MethodEntry { name: "length", arity: 0, slot: 102 },
];
static ARRAYBOX_TB: TypeBox = TypeBox::new_with("ArrayBox", ARRAY_METHODS);

// --- MapBox ---
const MAP_METHODS: &[MethodEntry] = &[
    MethodEntry { name: "size", arity: 0, slot: 200 },
    MethodEntry { name: "len", arity: 0, slot: 201 },
    MethodEntry { name: "has", arity: 1, slot: 202 },
    MethodEntry { name: "get", arity: 1, slot: 203 },
    MethodEntry { name: "set", arity: 2, slot: 204 },
];
static MAPBOX_TB: TypeBox = TypeBox::new_with("MapBox", MAP_METHODS);

// --- StringBox ---
const STRING_METHODS: &[MethodEntry] = &[
    MethodEntry { name: "len", arity: 0, slot: 300 },
];
static STRINGBOX_TB: TypeBox = TypeBox::new_with("StringBox", STRING_METHODS);

// --- ConsoleBox ---
const CONSOLE_METHODS: &[MethodEntry] = &[
    MethodEntry { name: "log", arity: 1, slot: 400 },
    MethodEntry { name: "warn", arity: 1, slot: 401 },
    MethodEntry { name: "error", arity: 1, slot: 402 },
    MethodEntry { name: "clear", arity: 0, slot: 403 },
];
static CONSOLEBOX_TB: TypeBox = TypeBox::new_with("ConsoleBox", CONSOLE_METHODS);

/// 型名から TypeBox を解決（雛形）。現在は常に None。
pub fn resolve_typebox_by_name(type_name: &str) -> Option<&'static TypeBox> {
    match type_name {
        "MapBox" => Some(&MAPBOX_TB),
        "ArrayBox" => Some(&ARRAYBOX_TB),
        "StringBox" => Some(&STRINGBOX_TB),
        "ConsoleBox" => Some(&CONSOLEBOX_TB),
        _ => None,
    }
}

/// 型名・メソッド名・アリティからスロットを解決（雛形）
pub fn resolve_slot_by_name(type_name: &str, method: &str, arity: usize) -> Option<u16> {
    let tb = resolve_typebox_by_name(type_name)?;
    let ar = arity as u8;
    for m in tb.methods {
        if m.name == method && m.arity == ar { return Some(m.slot); }
    }
    None
}
