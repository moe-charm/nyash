/*!
 * Type Registry (Tier-0 雛形)
 *
 * 目的:
 * - TypeId → TypeBox 参照の最小インターフェースを用意（現時点では未実装・常に未登録）。
 * - VM/JIT 実装が存在を前提に呼び出しても no-op/fallback できる状態にする。
 *
 * スロット番号の方針（注釈）
 * - ここで定義する `slot` は「VTable 用の仮想メソッドID」です。VM/JIT の内部ディスパッチ最適化
 *   と、Builtin Box の高速経路（fast path）に使われます。
 * - HostAPI（プラグインのネイティブ関数呼び出し）で用いるメソッド番号空間とは独立です。
 *   HostAPI 側は TLV で型付き引数を渡し、プラグイン実装側の関数テーブルにマップされます。
 *   そのため重複しても問題ありません（互いに衝突しない設計）。
 * - 慣例として以下の帯域を利用します（将来の整理用の目安）：
 *   - 0..=3: ユニバーサルスロット（toString/type/equal/copy 相当）
 *   - 100..: Array 系（get/set/len ほか拡張）
 *   - 200..: Map 系（size/len/has/get/set/delete ほか拡張）
 *   - 300..: String 系（len/substring/concat/indexOf/replace/trim/toUpper/toLower）
 *   - 400..: Console 系（log/warn/error/clear）
 */

use super::type_box_abi::{MethodEntry, TypeBox};

// 最小サンプル: MapBox の TypeBox を事前登録（Tier-1 PoC 用）
// --- ArrayBox ---
const ARRAY_METHODS: &[MethodEntry] = &[
    MethodEntry {
        name: "get",
        arity: 1,
        slot: 100,
    },
    MethodEntry {
        name: "set",
        arity: 2,
        slot: 101,
    },
    MethodEntry {
        name: "len",
        arity: 0,
        slot: 102,
    },
    MethodEntry {
        name: "length",
        arity: 0,
        slot: 102,
    },
    // P0: vtable coverage extension
    MethodEntry {
        name: "push",
        arity: 1,
        slot: 103,
    },
    MethodEntry {
        name: "pop",
        arity: 0,
        slot: 104,
    },
    MethodEntry {
        name: "clear",
        arity: 0,
        slot: 105,
    },
    // P1: contains/indexOf/join
    MethodEntry {
        name: "contains",
        arity: 1,
        slot: 106,
    },
    MethodEntry {
        name: "indexOf",
        arity: 1,
        slot: 107,
    },
    MethodEntry {
        name: "join",
        arity: 1,
        slot: 108,
    },
    // P2: sort/reverse/slice
    MethodEntry {
        name: "sort",
        arity: 0,
        slot: 109,
    },
    MethodEntry {
        name: "reverse",
        arity: 0,
        slot: 110,
    },
    MethodEntry {
        name: "slice",
        arity: 2,
        slot: 111,
    },
    MethodEntry {
        name: "toJSON",
        arity: 0,
        slot: 112,
    },
];
static ARRAYBOX_TB: TypeBox = TypeBox::new_with("ArrayBox", ARRAY_METHODS);

// --- MapBox ---
const MAP_METHODS: &[MethodEntry] = &[
    MethodEntry {
        name: "size",
        arity: 0,
        slot: 200,
    },
    MethodEntry {
        name: "len",
        arity: 0,
        slot: 201,
    },
    MethodEntry {
        name: "has",
        arity: 1,
        slot: 202,
    },
    MethodEntry {
        name: "get",
        arity: 1,
        slot: 203,
    },
    MethodEntry {
        name: "set",
        arity: 2,
        slot: 204,
    },
    // Extended
    MethodEntry {
        name: "delete",
        arity: 1,
        slot: 205,
    }, // alias: remove (同一スロット)
    MethodEntry {
        name: "remove",
        arity: 1,
        slot: 205,
    },
    MethodEntry {
        name: "keys",
        arity: 0,
        slot: 206,
    },
    MethodEntry {
        name: "values",
        arity: 0,
        slot: 207,
    },
    MethodEntry {
        name: "clear",
        arity: 0,
        slot: 208,
    },
    MethodEntry {
        name: "toJSON",
        arity: 0,
        slot: 209,
    },
];
static MAPBOX_TB: TypeBox = TypeBox::new_with("MapBox", MAP_METHODS);

// --- StringBox ---
const STRING_METHODS: &[MethodEntry] = &[
    MethodEntry {
        name: "len",
        arity: 0,
        slot: 300,
    },
    // P1: extend String vtable
    MethodEntry {
        name: "substring",
        arity: 2,
        slot: 301,
    },
    MethodEntry {
        name: "concat",
        arity: 1,
        slot: 302,
    },
    MethodEntry {
        name: "indexOf",
        arity: 1,
        slot: 303,
    },
    MethodEntry {
        name: "replace",
        arity: 2,
        slot: 304,
    },
    MethodEntry {
        name: "trim",
        arity: 0,
        slot: 305,
    },
    MethodEntry {
        name: "toUpper",
        arity: 0,
        slot: 306,
    },
    MethodEntry {
        name: "toLower",
        arity: 0,
        slot: 307,
    },
    // P2: toString/stringify/startsWith/endsWith
    MethodEntry {
        name: "toString",
        arity: 0,
        slot: 308,
    },
    MethodEntry {
        name: "stringify",
        arity: 0,
        slot: 309,
    },
    MethodEntry {
        name: "startsWith",
        arity: 1,
        slot: 310,
    },
    MethodEntry {
        name: "endsWith",
        arity: 1,
        slot: 311,
    },
];
static STRINGBOX_TB: TypeBox = TypeBox::new_with("StringBox", STRING_METHODS);

// --- ConsoleBox --- (WASM v2 unified dispatch 用の雛形)
// 400: log(..), 401: warn(..), 402: error(..), 403: clear()
const CONSOLE_METHODS: &[MethodEntry] = &[
    MethodEntry {
        name: "log",
        arity: 1,
        slot: 400,
    },
    MethodEntry {
        name: "warn",
        arity: 1,
        slot: 401,
    },
    MethodEntry {
        name: "error",
        arity: 1,
        slot: 402,
    },
    MethodEntry {
        name: "clear",
        arity: 0,
        slot: 403,
    },
];
static CONSOLEBOX_TB: TypeBox = TypeBox::new_with("ConsoleBox", CONSOLE_METHODS);

// --- InstanceBox ---
// Representative methods exposed via unified slots for field access and diagnostics.
// 1: getField(name)
// 2: setField(name, value)
// 3: has(name)
// 4: size()
const INSTANCE_METHODS: &[MethodEntry] = &[
    MethodEntry {
        name: "getField",
        arity: 1,
        slot: 1,
    },
    MethodEntry {
        name: "setField",
        arity: 2,
        slot: 2,
    },
    MethodEntry {
        name: "has",
        arity: 1,
        slot: 3,
    },
    MethodEntry {
        name: "size",
        arity: 0,
        slot: 4,
    },
];
static INSTANCEBOX_TB: TypeBox = TypeBox::new_with("InstanceBox", INSTANCE_METHODS);

/// 型名から TypeBox を解決（雛形）。現在は常に None。
pub fn resolve_typebox_by_name(type_name: &str) -> Option<&'static TypeBox> {
    match type_name {
        "MapBox" => Some(&MAPBOX_TB),
        "ArrayBox" => Some(&ARRAYBOX_TB),
        "StringBox" => Some(&STRINGBOX_TB),
        "ConsoleBox" => Some(&CONSOLEBOX_TB),
        "InstanceBox" => Some(&INSTANCEBOX_TB),
        _ => None,
    }
}

/// 型名・メソッド名・アリティからスロットを解決（雛形）
pub fn resolve_slot_by_name(type_name: &str, method: &str, arity: usize) -> Option<u16> {
    let tb = resolve_typebox_by_name(type_name)?;
    let ar = arity as u8;
    for m in tb.methods {
        if m.name == method && m.arity == ar {
            return Some(m.slot);
        }
    }
    None
}

/// Return list of known methods for a type (names only) for diagnostics.
pub fn known_methods_for(type_name: &str) -> Option<Vec<&'static str>> {
    let tb = resolve_typebox_by_name(type_name)?;
    let mut v: Vec<&'static str> = tb.methods.iter().map(|m| m.name).collect();
    v.sort();
    v.dedup();
    Some(v)
}

/// Return list of known arities for a specific method name on a type.
pub fn known_arities_for(type_name: &str, method: &str) -> Option<Vec<u8>> {
    let tb = resolve_typebox_by_name(type_name)?;
    let mut v: Vec<u8> = tb
        .methods
        .iter()
        .filter(|m| m.name == method)
        .map(|m| m.arity)
        .collect();
    if v.is_empty() {
        return Some(vec![]);
    }
    v.sort();
    v.dedup();
    Some(v)
}
