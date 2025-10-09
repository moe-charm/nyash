//! box_call.rs — BoxCall bridges
//!
//! Extraction target: builtin Array fast‑paths currently embedded in call flow.

use super::super::*;

impl MirInterpreter {
    /// Handle common ArrayBox fast‑paths. Returns Some(result) if handled.
    pub(crate) fn box_array_fastpath(
        &mut self,
        arr: &crate::boxes::array::ArrayBox,
        method: &str,
        args: &[ValueId],
    ) -> Option<Result<VMValue, VMError>> {
        match method {
            "birth" => Some(Ok(VMValue::Void)),
            "push" => {
                if let Some(a0) = args.get(0) {
                    let v = match self.reg_load(*a0) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let _ = arr.push(v);
                    Some(Ok(VMValue::Void))
                } else {
                    None
                }
            }
            "len" | "length" | "size" => {
                let ret = arr.length();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            "get" => {
                if let Some(a0) = args.get(0) {
                    let idx = match self.reg_load(*a0) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let ret = arr.get(idx);
                    Some(Ok(VMValue::from_nyash_box(ret)))
                } else {
                    None
                }
            }
            "set" => {
                if args.len() >= 2 {
                    let idx = match self.reg_load(args[0]) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let val = match self.reg_load(args[1]) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let _ = arr.set(idx, val);
                    Some(Ok(VMValue::Void))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle common MapBox fast‑paths. Returns Some(result) if handled.
    pub(crate) fn box_map_fastpath(
        &mut self,
        map: &crate::boxes::map_box::MapBox,
        method: &str,
        args: &[ValueId],
    ) -> Option<Result<VMValue, VMError>> {
        match method {
            "birth" => Some(Ok(VMValue::Void)),
            "set" => {
                if args.len() >= 2 {
                    let key = match self.reg_load(args[0]) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let val = match self.reg_load(args[1]) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let ret = map.set(key, val);
                    Some(Ok(VMValue::from_nyash_box(ret)))
                } else {
                    None
                }
            }
            "get" => {
                if let Some(a0) = args.get(0) {
                    let key = match self.reg_load(*a0) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let ret = map.get(key);
                    Some(Ok(VMValue::from_nyash_box(ret)))
                } else { None }
            }
            "has" => {
                if let Some(a0) = args.get(0) {
                    let key = match self.reg_load(*a0) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let ret = map.has(key);
                    Some(Ok(VMValue::from_nyash_box(ret)))
                } else { None }
            }
            "delete" => {
                if let Some(a0) = args.get(0) {
                    let key = match self.reg_load(*a0) { Ok(v) => v.to_nyash_box(), Err(e) => return Some(Err(e)) };
                    let ret = map.delete(key);
                    Some(Ok(VMValue::from_nyash_box(ret)))
                } else { None }
            }
            "size" | "length" => {
                let ret = map.size();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            "keys" => {
                let ret = map.keys();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            "values" => {
                let ret = map.values();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            "toJSON" => {
                if !args.is_empty() { return None; }
                let ret = map.toJSON();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            _ => None,
        }
    }

    /// Handle common StringBox fast‑paths. Returns Some(result) if handled.
    pub(crate) fn box_string_fastpath(
        &mut self,
        s: &crate::boxes::string_box::StringBox,
        method: &str,
        args: &[ValueId],
    ) -> Option<Result<VMValue, VMError>> {
        match method {
            "birth" => Some(Ok(VMValue::Void)),
            "upper" => {
                if !args.is_empty() { return None; }
                let ret = s.to_upper();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            "lower" => {
                if !args.is_empty() { return None; }
                let ret = s.to_lower();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            "size" | "length" => {
                if !args.is_empty() { return None; }
                let ret = s.size();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            "isEmpty" => {
                if !args.is_empty() { return None; }
                let ret = s.isEmpty();
                Some(Ok(VMValue::from_nyash_box(ret)))
            }
            _ => None,
        }
    }
}
