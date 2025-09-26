use super::*;
use crate::box_trait::NyashBox;

pub(super) fn try_handle_map_box(
    this: &mut MirInterpreter,
    dst: Option<ValueId>,
    box_val: ValueId,
    method: &str,
    args: &[ValueId],
) -> Result<bool, VMError> {
        let recv = this.reg_load(box_val)?;
        let recv_box_any: Box<dyn NyashBox> = match recv.clone() {
            VMValue::BoxRef(b) => b.share_box(),
            other => other.to_nyash_box(),
        };
        if let Some(mb) = recv_box_any
            .as_any()
            .downcast_ref::<crate::boxes::map_box::MapBox>()
        {
            match method {
                "birth" => {
                    // No-op constructor init for MapBox
                    if let Some(d) = dst { this.regs.insert(d, VMValue::Void); }
                    return Ok(true);
                }
                "set" => {
                    if args.len() != 2 { return Err(VMError::InvalidInstruction("MapBox.set expects 2 args".into())); }
                    let k = this.reg_load(args[0])?.to_nyash_box();
                    let v = this.reg_load(args[1])?.to_nyash_box();
                    let ret = mb.set(k, v);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "get" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("MapBox.get expects 1 arg".into())); }
                    let k = this.reg_load(args[0])?.to_nyash_box();
                    let ret = mb.get(k);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "has" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("MapBox.has expects 1 arg".into())); }
                    let k = this.reg_load(args[0])?.to_nyash_box();
                    let ret = mb.has(k);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "delete" => {
                    if args.len() != 1 { return Err(VMError::InvalidInstruction("MapBox.delete expects 1 arg".into())); }
                    let k = this.reg_load(args[0])?.to_nyash_box();
                    let ret = mb.delete(k);
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "size" => {
                    let ret = mb.size();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "keys" => {
                    let ret = mb.keys();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                "values" => {
                    let ret = mb.values();
                    if let Some(d) = dst { this.regs.insert(d, VMValue::from_nyash_box(ret)); }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
}
