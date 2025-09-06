#[cfg(all(test, not(feature = "jit-direct-only")))]
mod tests {
    use std::env;
    use crate::box_trait::{NyashBox, StringBox, IntegerBox};
    use crate::boxes::math_box::FloatBox;
    use crate::boxes::array::ArrayBox;
    use std::fs;
    use std::path::PathBuf;

    fn ensure_host() {
        let _ = crate::runtime::init_global_plugin_host("nyash.toml");
    }

    fn create_plugin_instance(box_type: &str) -> (String, u32, Box<dyn NyashBox>) {
        let host = crate::runtime::get_global_plugin_host();
        let host = host.read().unwrap();
        let bx = host.create_box(box_type, &[]).expect("create_box");
        // Downcast to PluginBoxV2 to get instance_id
        if let Some(p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            (box_type.to_string(), p.instance_id(), bx)
        } else {
            panic!("not a plugin box: {}", bx.type_name());
        }
    }

    #[test]
    #[ignore = "MIR13 parity: MapBox TLV vs TypeBox under unified BoxCall/TypeOp pending"]
    fn mapbox_get_set_size_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();

        // TLV path: disable typebox
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("MapBox");
        let out_tlv = {
            let h = host.read().unwrap();
            // set("k", 42)
            let _ = h.invoke_instance_method(&bt1, "set", id1, &[Box::new(StringBox::new("k")), Box::new(IntegerBox::new(42))]).expect("set tlv");
            // size()
            let sz = h.invoke_instance_method(&bt1, "size", id1, &[]).expect("size tlv").unwrap();
            // get("k")
            let gv = h.invoke_instance_method(&bt1, "get", id1, &[Box::new(StringBox::new("k"))]).expect("get tlv").unwrap();
            (sz.to_string_box().value, gv.to_string_box().value)
        };

        // TypeBox path: enable typebox
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("MapBox");
        let out_tb = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt2, "set", id2, &[Box::new(StringBox::new("k")), Box::new(IntegerBox::new(42))]).expect("set tb");
            let sz = h.invoke_instance_method(&bt2, "size", id2, &[]).expect("size tb").unwrap();
            let gv = h.invoke_instance_method(&bt2, "get", id2, &[Box::new(StringBox::new("k"))]).expect("get tb").unwrap();
            (sz.to_string_box().value, gv.to_string_box().value)
        };

        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match");
    }

    #[test]
    #[ignore = "MIR13 parity: ArrayBox len/get under unified ops pending"]
    fn arraybox_set_get_len_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();
        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("ArrayBox");
        let out_tlv = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt1, "set", id1, &[Box::new(IntegerBox::new(0)), Box::new(IntegerBox::new(7))]).expect("set tlv");
            let ln = h.invoke_instance_method(&bt1, "len", id1, &[]).expect("len tlv").unwrap();
            let gv = h.invoke_instance_method(&bt1, "get", id1, &[Box::new(IntegerBox::new(0))]).expect("get tlv").unwrap();
            (ln.to_string_box().value, gv.to_string_box().value)
        };
        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("ArrayBox");
        let out_tb = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt2, "set", id2, &[Box::new(IntegerBox::new(0)), Box::new(IntegerBox::new(7))]).expect("set tb");
            let ln = h.invoke_instance_method(&bt2, "length", id2, &[]).expect("len tb").unwrap();
            let gv = h.invoke_instance_method(&bt2, "get", id2, &[Box::new(IntegerBox::new(0))]).expect("get tb").unwrap();
            (ln.to_string_box().value, gv.to_string_box().value)
        };
        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (ArrayBox)");
    }

    #[test]
    #[ignore = "MIR13 parity: StringBox length/concat under unified ops pending"]
    fn stringbox_len_concat_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();
        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("StringBox");
        let out_tlv = {
            let h = host.read().unwrap();
            // birth with init string: use fromUtf8 via set of arg in create? Current loader birth() no-arg, so concat
            let _ = h.invoke_instance_method(&bt1, "concat", id1, &[Box::new(StringBox::new("ab"))]).expect("concat tlv").unwrap();
            let ln = h.invoke_instance_method(&bt1, "length", id1, &[]).expect("len tlv").unwrap();
            (ln.to_string_box().value)
        };
        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("StringBox");
        let out_tb = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt2, "concat", id2, &[Box::new(StringBox::new("ab"))]).expect("concat tb").unwrap();
            let ln = h.invoke_instance_method(&bt2, "length", id2, &[]).expect("len tb").unwrap();
            (ln.to_string_box().value)
        };
        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (StringBox)");
    }

    #[test]
    #[ignore = "MIR13 parity: IntegerBox get/set under unified ops pending"]
    fn integerbox_get_set_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();
        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("IntegerBox");
        let out_tlv = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt1, "set", id1, &[Box::new(IntegerBox::new(123))]).expect("set tlv").unwrap();
            let gv = h.invoke_instance_method(&bt1, "get", id1, &[]).expect("get tlv").unwrap();
            gv.to_string_box().value
        };
        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("IntegerBox");
        let out_tb = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt2, "set", id2, &[Box::new(IntegerBox::new(123))]).expect("set tb").unwrap();
            let gv = h.invoke_instance_method(&bt2, "get", id2, &[]).expect("get tb").unwrap();
            gv.to_string_box().value
        };
        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (IntegerBox)");
    }

    #[test]
    fn consolebox_println_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();
        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("ConsoleBox");
        let out_tlv_is_none = {
            let h = host.read().unwrap();
            let rv = h.invoke_instance_method(&bt1, "println", id1, &[Box::new(StringBox::new("hello"))]).expect("println tlv");
            rv.is_none()
        };
        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("ConsoleBox");
        let out_tb_is_none = {
            let h = host.read().unwrap();
            let rv = h.invoke_instance_method(&bt2, "println", id2, &[Box::new(StringBox::new("hello"))]).expect("println tb");
            rv.is_none()
        };
        assert!(out_tlv_is_none && out_tb_is_none, "println should return void/None in both modes");
    }

    #[test]
    #[ignore = "MIR13 parity: MathBox basic ops under unified ops pending"]
    fn mathbox_basic_ops_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();

        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("MathBox");
        let out_tlv = {
            let h = host.read().unwrap();
            let s1 = h.invoke_instance_method(&bt1, "sqrt", id1, &[Box::new(IntegerBox::new(9))]).expect("sqrt tlv").unwrap();
            let s2 = h.invoke_instance_method(&bt1, "sin", id1, &[Box::new(IntegerBox::new(0))]).expect("sin tlv").unwrap();
            let s3 = h.invoke_instance_method(&bt1, "cos", id1, &[Box::new(IntegerBox::new(0))]).expect("cos tlv").unwrap();
            let s4 = h.invoke_instance_method(&bt1, "round", id1, &[Box::new(IntegerBox::new(26))]).expect("round tlv").unwrap();
            (
                s1.to_string_box().value,
                s2.to_string_box().value,
                s3.to_string_box().value,
                s4.to_string_box().value,
            )
        };

        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("MathBox");
        let out_tb = {
            let h = host.read().unwrap();
            let s1 = h.invoke_instance_method(&bt2, "sqrt", id2, &[Box::new(IntegerBox::new(9))]).expect("sqrt tb").unwrap();
            let s2 = h.invoke_instance_method(&bt2, "sin", id2, &[Box::new(IntegerBox::new(0))]).expect("sin tb").unwrap();
            let s3 = h.invoke_instance_method(&bt2, "cos", id2, &[Box::new(IntegerBox::new(0))]).expect("cos tb").unwrap();
            let s4 = h.invoke_instance_method(&bt2, "round", id2, &[Box::new(IntegerBox::new(26))]).expect("round tb").unwrap();
            (
                s1.to_string_box().value,
                s2.to_string_box().value,
                s3.to_string_box().value,
                s4.to_string_box().value,
            )
        };
        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (MathBox)");
    }

    #[test]
    #[ignore = "MIR13 parity: EncodingBox base64/hex under unified ops pending"]
    fn encodingbox_base64_hex_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();

        // Prepare bytes ["hi"] as Array<uint8>
        let bytes_array = {
            let arr = ArrayBox::new();
            let _ = arr.push(Box::new(IntegerBox::new(104))); // 'h'
            let _ = arr.push(Box::new(IntegerBox::new(105))); // 'i'
            Box::new(arr) as Box<dyn NyashBox>
        };

        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("EncodingBox");
        let out_tlv = {
            let h = host.read().unwrap();
            let b64 = h.invoke_instance_method(&bt1, "base64Encode", id1, &[Box::new(StringBox::new("hi"))]).expect("b64 tlv").unwrap();
            let hex = h.invoke_instance_method(&bt1, "hexEncode", id1, &[Box::new(StringBox::new("hi"))]).expect("hex tlv").unwrap();
            (b64.to_string_box().value, hex.to_string_box().value)
        };

        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("EncodingBox");
        let out_tb = {
            let h = host.read().unwrap();
            let b64 = h.invoke_instance_method(&bt2, "base64Encode", id2, &[Box::new(StringBox::new("hi"))]).expect("b64 tb").unwrap();
            let hex = h.invoke_instance_method(&bt2, "hexEncode", id2, &[Box::new(StringBox::new("hi"))]).expect("hex tb").unwrap();
            (b64.to_string_box().value, hex.to_string_box().value)
        };
        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (EncodingBox)");
    }

    #[test]
    #[ignore = "MIR13 parity: RegexBox match/find under unified ops pending"]
    fn regexbox_is_match_find_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();

        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("RegexBox");
        let out_tlv = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt1, "compile", id1, &[Box::new(StringBox::new("h.+o"))]).expect("compile tlv");
            let m = h.invoke_instance_method(&bt1, "isMatch", id1, &[Box::new(StringBox::new("hello"))]).expect("isMatch tlv").unwrap();
            let f = h.invoke_instance_method(&bt1, "find", id1, &[Box::new(StringBox::new("hello"))]).expect("find tlv").unwrap();
            (m.to_string_box().value, f.to_string_box().value)
        };

        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("RegexBox");
        let out_tb = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt2, "compile", id2, &[Box::new(StringBox::new("h.+o"))]).expect("compile tb");
            let m = h.invoke_instance_method(&bt2, "isMatch", id2, &[Box::new(StringBox::new("hello"))]).expect("isMatch tb").unwrap();
            let f = h.invoke_instance_method(&bt2, "find", id2, &[Box::new(StringBox::new("hello"))]).expect("find tb").unwrap();
            (m.to_string_box().value, f.to_string_box().value)
        };
        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (RegexBox)");
    }

    #[test]
    #[ignore = "MIR13 parity: PathBox ops under unified ops pending"]
    fn pathbox_ops_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();

        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("PathBox");
        let out_tlv = {
            let h = host.read().unwrap();
            let j = h.invoke_instance_method(&bt1, "join", id1, &[Box::new(StringBox::new("/a/b")), Box::new(StringBox::new("c.txt"))]).expect("join tlv").unwrap();
            let d = h.invoke_instance_method(&bt1, "dirname", id1, &[Box::new(StringBox::new("/a/b/c.txt"))]).expect("dirname tlv").unwrap();
            let b = h.invoke_instance_method(&bt1, "basename", id1, &[Box::new(StringBox::new("/a/b/c.txt"))]).expect("basename tlv").unwrap();
            let n = h.invoke_instance_method(&bt1, "normalize", id1, &[Box::new(StringBox::new("/a/./b/../b/c"))]).expect("normalize tlv").unwrap();
            (j.to_string_box().value, d.to_string_box().value, b.to_string_box().value, n.to_string_box().value)
        };

        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("PathBox");
        let out_tb = {
            let h = host.read().unwrap();
            let j = h.invoke_instance_method(&bt2, "join", id2, &[Box::new(StringBox::new("/a/b")), Box::new(StringBox::new("c.txt"))]).expect("join tb").unwrap();
            let d = h.invoke_instance_method(&bt2, "dirname", id2, &[Box::new(StringBox::new("/a/b/c.txt"))]).expect("dirname tb").unwrap();
            let b = h.invoke_instance_method(&bt2, "basename", id2, &[Box::new(StringBox::new("/a/b/c.txt"))]).expect("basename tb").unwrap();
            let n = h.invoke_instance_method(&bt2, "normalize", id2, &[Box::new(StringBox::new("/a/./b/../b/c"))]).expect("normalize tb").unwrap();
            (j.to_string_box().value, d.to_string_box().value, b.to_string_box().value, n.to_string_box().value)
        };
        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (PathBox)");
    }

    #[test]
    #[ignore = "MIR13 parity: TOMLBox parse/get/toJson under unified ops pending"]
    fn tomlbox_parse_get_tojson_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();
        let toml_text = "[package]\nname=\"nyash\"\n[deps]\nregex=\"1\"\n";

        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("TOMLBox");
        let out_tlv = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt1, "parse", id1, &[Box::new(StringBox::new(toml_text))]).expect("parse tlv").unwrap();
            let name = h.invoke_instance_method(&bt1, "get", id1, &[Box::new(StringBox::new("package.name"))]).expect("get tlv").unwrap();
            let json = h.invoke_instance_method(&bt1, "toJson", id1, &[]).expect("toJson tlv").unwrap();
            (name.to_string_box().value, json.to_string_box().value)
        };

        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("TOMLBox");
        let out_tb = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt2, "parse", id2, &[Box::new(StringBox::new(toml_text))]).expect("parse tb").unwrap();
            let name = h.invoke_instance_method(&bt2, "get", id2, &[Box::new(StringBox::new("package.name"))]).expect("get tb").unwrap();
            let json = h.invoke_instance_method(&bt2, "toJson", id2, &[]).expect("toJson tb").unwrap();
            (name.to_string_box().value, json.to_string_box().value)
        };
        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (TOMLBox)");
    }

    #[test]
    #[ignore = "MIR13 parity: TimeBox now tolerance under unified ops pending"]
    fn timebox_now_tlv_vs_typebox_with_tolerance() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();

        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("TimeBox");
        let t_tlv = {
            let h = host.read().unwrap();
            let v = h.invoke_instance_method(&bt1, "now", id1, &[]).expect("now tlv").unwrap();
            v.to_string_box().value.parse::<i64>().unwrap_or(0)
        };

        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("TimeBox");
        let t_tb = {
            let h = host.read().unwrap();
            let v = h.invoke_instance_method(&bt2, "now", id2, &[]).expect("now tb").unwrap();
            v.to_string_box().value.parse::<i64>().unwrap_or(0)
        };

        let diff = (t_tb - t_tlv).abs();
        assert!(diff < 5_000, "TimeBox.now difference too large: {}ms", diff);
    }

    #[test]
    #[ignore = "MIR13 parity: CounterBox singleton delta under unified ops pending"]
    fn counterbox_singleton_delta_increments() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();

        // TLV path: verify get->inc->get increases by 1
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("CounterBox");
        let (a1, b1) = {
            let h = host.read().unwrap();
            let a = h.invoke_instance_method(&bt1, "get", id1, &[]).expect("get tlv").unwrap();
            let _ = h.invoke_instance_method(&bt1, "inc", id1, &[]).expect("inc tlv");
            let b = h.invoke_instance_method(&bt1, "get", id1, &[]).expect("get2 tlv").unwrap();
            (a.to_string_box().value.parse::<i64>().unwrap_or(0), b.to_string_box().value.parse::<i64>().unwrap_or(0))
        };
        assert_eq!(b1 - a1, 1, "CounterBox TLV should increment by 1");

        // TypeBox path: verify same delta behavior (not comparing absolute values due to singleton)
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("CounterBox");
        let (a2, b2) = {
            let h = host.read().unwrap();
            let a = h.invoke_instance_method(&bt2, "get", id2, &[]).expect("get tb").unwrap();
            let _ = h.invoke_instance_method(&bt2, "inc", id2, &[]).expect("inc tb");
            let b = h.invoke_instance_method(&bt2, "get", id2, &[]).expect("get2 tb").unwrap();
            (a.to_string_box().value.parse::<i64>().unwrap_or(0), b.to_string_box().value.parse::<i64>().unwrap_or(0))
        };
        assert_eq!(b2 - a2, 1, "CounterBox TypeBox should increment by 1");
    }

    #[test]
    #[ignore = "MIR13 parity: FileBox RW/close under unified ops pending"]
    fn filebox_rw_close_tmpdir_tlv_vs_typebox() {
        ensure_host();
        let host = crate::runtime::get_global_plugin_host();

        // Prepare temp file path
        let mut p = std::env::temp_dir();
        p.push(format!("nyash_test_{}_{}.txt", std::process::id(), rand_id())) ;
        let path_str = p.to_string_lossy().to_string();

        // TLV path
        env::set_var("NYASH_DISABLE_TYPEBOX", "1");
        let (bt1, id1, _hold1) = create_plugin_instance("FileBox");
        let out_tlv = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt1, "open", id1, &[Box::new(StringBox::new(&path_str)), Box::new(StringBox::new("w"))]).expect("open tlv");
            let _ = h.invoke_instance_method(&bt1, "write", id1, &[Box::new(StringBox::new("hello"))]).expect("write tlv");
            let _ = h.invoke_instance_method(&bt1, "close", id1, &[]).expect("close tlv");
            // reopen and read
            let _ = h.invoke_instance_method(&bt1, "open", id1, &[Box::new(StringBox::new(&path_str)), Box::new(StringBox::new("r"))]).expect("open2 tlv");
            let rd = h.invoke_instance_method(&bt1, "read", id1, &[]).expect("read tlv").unwrap();
            let _ = h.invoke_instance_method(&bt1, "close", id1, &[]).expect("close2 tlv");
            rd.to_string_box().value
        };

        // TypeBox path
        env::remove_var("NYASH_DISABLE_TYPEBOX");
        let (bt2, id2, _hold2) = create_plugin_instance("FileBox");
        let out_tb = {
            let h = host.read().unwrap();
            let _ = h.invoke_instance_method(&bt2, "open", id2, &[Box::new(StringBox::new(&path_str)), Box::new(StringBox::new("w"))]).expect("open tb");
            let _ = h.invoke_instance_method(&bt2, "write", id2, &[Box::new(StringBox::new("hello"))]).expect("write tb");
            let _ = h.invoke_instance_method(&bt2, "close", id2, &[]).expect("close tb");
            let _ = h.invoke_instance_method(&bt2, "open", id2, &[Box::new(StringBox::new(&path_str)), Box::new(StringBox::new("r"))]).expect("open2 tb");
            let rd = h.invoke_instance_method(&bt2, "read", id2, &[]).expect("read tb").unwrap();
            let _ = h.invoke_instance_method(&bt2, "close", id2, &[]).expect("close2 tb");
            rd.to_string_box().value
        };

        // Cleanup best-effort
        let _ = fs::remove_file(&path_str);

        assert_eq!(out_tlv, out_tb, "TLV vs TypeBox results should match (FileBox)");
    }

    fn rand_id() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        now.as_micros() as u64
    }
}
