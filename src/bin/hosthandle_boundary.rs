//! hosthandle_boundary.rs — Boundary checks for HostHandleRouter
//!
//! Purpose: exercise error return codes without going through high-level routers.
//!
//! Checks
//! - UNKNOWN_HANDLE: pass an invalid handle → expect rc = -1
//! - WRONG_TYPE: create StringBox handle, call MAP_HAS(202) → expect rc = -11
//! - TLV_DECODE: create ArrayBox handle, call ARRAY_SET(101) with 1 arg only → expect rc = -13

use nyash_rust::runtime::host_api_box;
use nyash_rust::runtime::host_api_box::slots as S;

fn main() {
    let mut ok = true;

    // UNKNOWN_HANDLE (-1)
    let bad_handle: u64 = 0xFFFF_FFFF_FFFF_FFF0;
    match host_api_box::call_slot_grow(bad_handle, S::MAP_SIZE, &[]) {
        Ok(_) => {
            println!("UNKNOWN_HANDLE rc=0 (unexpected)");
            ok = false;
        }
        Err(rc) => {
            println!("UNKNOWN_HANDLE rc={}", rc);
            if rc != -1 {
                ok = false;
            }
        }
    }

    // WRONG_TYPE (-11): StringBox handle with Map.has selector
    let sb = nyash_rust::box_trait::StringBox::new("hello".to_string());
    let sb_h = nyash_rust::runtime::host_handles::to_handle_box(
        Box::new(sb) as Box<dyn nyash_rust::box_trait::NyashBox>
    );
    let mut tlv_key = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(1);
    nyash_rust::runtime::plugin_ffi_common::encode::string(&mut tlv_key, "k");
    match host_api_box::call_slot_grow(sb_h, S::MAP_HAS, &tlv_key) {
        Ok(_) => {
            println!("WRONG_TYPE rc=0 (unexpected)");
            ok = false;
        }
        Err(rc) => {
            println!("WRONG_TYPE rc={}", rc);
            if rc != -11 {
                ok = false;
            }
        }
    }

    // TLV_DECODE (-13): Array.set requires 2 args; provide only 1
    #[cfg(feature = "legacy-boxes")]
    {
        let arr = nyash_rust::boxes::array::ArrayBox::new();
        let arr_h = nyash_rust::runtime::host_handles::to_handle_box(Box::new(arr));
        let mut tlv_one = nyash_rust::runtime::plugin_ffi_common::encode_tlv_header(1);
        nyash_rust::runtime::plugin_ffi_common::encode::i64(&mut tlv_one, 0);
        match host_api_box::call_slot_grow(arr_h, S::ARRAY_SET, &tlv_one) {
            Ok(_) => {
                println!("TLV_DECODE rc=0 (unexpected)");
                ok = false;
            }
            Err(rc) => {
                println!("TLV_DECODE rc={}", rc);
                if rc != -13 {
                    ok = false;
                }
            }
        }
    }
    #[cfg(not(feature = "legacy-boxes"))]
    {
        // In plugin-only builds, skip TLV_DECODE check (no legacy Array slot)
        println!("TLV_DECODE rc=SKIP (plugin-only)");
    }

    if ok {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
