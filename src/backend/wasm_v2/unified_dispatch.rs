//! Unified dispatch (WASM v2)
//!
//! - TypeRegistryのスロット表と一致させた呼び出し分岐の雛形
//! - env.console.log とArray/Map統一ディスパッチの最小実装

#![cfg(feature = "wasm-backend")]

use crate::box_trait::{NyashBox, StringBox, VoidBox, BoolBox};
use crate::boxes::{ConsoleBox, ArrayBox, MapBox};

/// 受信ボックス/メソッド名/アリティからスロットを解決し、識別子を返す。
pub fn resolve_slot(recv: &dyn NyashBox, method: &str, arity: usize) -> Option<u16> {
    let ty = recv.type_name();
    crate::runtime::type_registry::resolve_slot_by_name(ty, method, arity)
}

/// WASM v2統一ディスパッチ: console.log とArray/Map基本操作を実装
pub fn dispatch_by_slot(
    slot: u16,
    recv: &dyn NyashBox,
    args: &[Box<dyn NyashBox>],
) -> Option<Box<dyn NyashBox>> {
    match slot {
        // ConsoleBox slots (400番台予約)
        400 => {
            // console.log(message)
            if let Some(console) = recv.as_any().downcast_ref::<ConsoleBox>() {
                if args.len() == 1 {
                    let message = args[0].to_string_box().value;
                    console.log(&message);
                    return Some(Box::new(VoidBox::new()));
                }
            }
            None
        }
        
        // ArrayBox slots (100番台)
        100 => {
            // array.get(index)
            if let Some(array) = recv.as_any().downcast_ref::<ArrayBox>() {
                if args.len() == 1 {
                    if let Ok(index) = args[0].to_string_box().value.parse::<usize>() {
                        return array.get(index);
                    }
                }
            }
            None
        }
        102 => {
            // array.length()
            if let Some(array) = recv.as_any().downcast_ref::<ArrayBox>() {
                return Some(Box::new(StringBox::new(&array.len().to_string())));
            }
            None
        }
        
        // MapBox slots (200番台)  
        200 => {
            // map.size()
            if let Some(map) = recv.as_any().downcast_ref::<MapBox>() {
                return Some(Box::new(StringBox::new(&map.size().to_string())));
            }
            None
        }
        202 => {
            // map.has(key)
            if let Some(map) = recv.as_any().downcast_ref::<MapBox>() {
                if args.len() == 1 {
                    let key_box = args[0].clone_box();
                    return Some(map.has(key_box));
                }
            }
            None
        }
        203 => {
            // map.get(key)
            if let Some(map) = recv.as_any().downcast_ref::<MapBox>() {
                if args.len() == 1 {
                    let key_box = args[0].clone_box();
                    return Some(map.get(key_box));
                }
            }
            None
        }
        
        _ => None
    }
}

